use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn::{FnArg::Typed, Ident, ImplItemConst, ImplItemMethod, ItemImpl, ReturnType::Type};
use std::sync::RwLock;
use syn::spanned::Spanned;

use crate::EXPORTNUM;
lazy_static::lazy_static! {
    static ref WRAPPER_NUM: RwLock<usize> = RwLock::new(0);
}

pub fn patchable(mut fn_item: ItemImpl, modpath: Option<String>) -> TokenStream {
    let mut tt = proc_macro2::TokenStream::new();
    fn_item.self_ty.clone().to_tokens(&mut tt);
    let self_ty = fn_item.self_ty.clone();
    let impl_name = tt.to_string();
    
    fn_item.items = fn_item
        .items
        .drain(..)
        .map(|item| {
            match item {
                syn::ImplItem::Method(m) => {
                    let (mut fargs, mut output_type, mut item, mut fn_name, sigtext) = gather_info(m);

		    let wrapper_num;
		    {
			// scope is used so EXPORTNUM is unlocked faster
			let mut r = WRAPPER_NUM.write().unwrap();
			wrapper_num = *r;
			*r += 1;
		    }
		    
		    // transform arguements from Self notation to concrete type (only in inetermediate variables)
		    if let syn::Type::Tuple(ref mut t) = fargs {
			for farg in t.elems.iter_mut() {
			    transform_self(&impl_name, farg);
			}
		    }
		    // same but for return value
		    transform_self(&impl_name, &mut output_type);
		    
                    let vis = item.vis.clone(); // pass through pub
                    let mut docitem = item.clone();
                    docitem.attrs.append(
                        &mut syn::parse2::<syn::ItemStruct>(quote! {
                            ///
                            /// ---
                            /// ## Hotpatch
                            /// **Warning**: This item is [`#[patchable]`](hotpatch::patchable). Runtime behavior may not
                            /// follow the source implementation. See the
                            /// [Hotpatch Documentation](hotpatch) for more information.
                        #[cfg(doc)]
                            struct Dummy {}
                        })
                            .unwrap()
                        .attrs,
                    );
                    let item_name = fn_name.clone();
                    fn_name = Ident::new(&format!("__hotpatch_internal_staticwrap_{}", wrapper_num), Span::call_site());
                    item.sig.ident = fn_name.clone();
		    let mname = match &modpath {
			Some(mpath) => 
			    format!("!__associated_fn:{}:{}", impl_name, mpath),
			None => 
			    format!("!__associated_fn:{}:{}", impl_name, item_name),
		    };
		    
		    let c_item = syn::parse2::<ImplItemConst>(quote! {
			#[cfg(not(doc))]
			#[allow(non_upper_case_globals)]
			#vis const #item_name: hotpatch::MutConst<Patchable<dyn Fn#fargs -> #output_type + Send + Sync + 'static>> =hotpatch::MutConst::new(|| {
			    #[cfg(not(doc))]
			    #[allow(non_upper_case_globals)]
			    static __hotpatch_internal_pwrap: hotpatch::Patchable<
				    dyn Fn#fargs -> #output_type + Send + Sync + 'static,
				> = hotpatch::Patchable::__new(|| {
				    hotpatch::Patchable::__new_internal(
					Box::new(#self_ty::#fn_name)
					    as Box<dyn Fn#fargs -> #output_type + Send + Sync + 'static>,
					concat!(module_path!(), "::", #mname),
					#sigtext,
				    )
				});
			    &__hotpatch_internal_pwrap
			});
		    }).unwrap();
		    let f_item = syn::parse2::<ImplItemMethod>(quote! {
			#item
		    }).unwrap();
		    (syn::ImplItem::Method(docitem), syn::ImplItem::Const(c_item), syn::ImplItem::Method(f_item))
                }
                _ => panic!("There's something in this impl block I can't hotpatch yet"),
            }
        }).fold(vec![], |mut acc, (c1, c2, c3)| {acc.push(c1); acc.push(c2); acc.push(c3); acc});

    TokenStream::from(quote! {
    #fn_item
    })
}


pub fn patch(mut fn_item: ItemImpl, modpath: Option<String>) -> TokenStream {
    
    let mut tt = proc_macro2::TokenStream::new();
    fn_item.self_ty.clone().to_tokens(&mut tt);
    let impl_name = tt.to_string();
    let self_type = fn_item.self_ty.clone();
    
    let exports: Vec<_> = fn_item
        .items
        .iter_mut()
        .map(|item| {
            match item {
                syn::ImplItem::Method(m) => {
                    let (mut fargs, mut output_type, _item, fn_name, sigtext) = gather_info(m.clone());
		    
		    // transform arguements from Self notation to concrete type (only in inetermediate variables)
		    if let syn::Type::Tuple(ref mut t) = fargs {
			for farg in t.elems.iter_mut() {
			    transform_self(&impl_name, farg);
			}
		    }
		    // same but for return value
		    transform_self(&impl_name, &mut output_type);

		    let exnum;
		    {
			// scope is used so EXPORTNUM is unlocked faster
			let mut r = EXPORTNUM.write().unwrap();
			exnum = *r;
			*r += 1;
		    }
		    
                    m.attrs.append(
                        &mut syn::parse2::<syn::ItemStruct>(quote! {
			    ///
			    /// ---
			    /// ## Hotpatch
			    /// This item is a [`#[patch]`](hotpatch::patch). It will silently define a public static
			    /// symbol `__HOTPATCH_EXPORT_N` for use in shared object files. See the
			    /// [Hotpatch Documentation](hotpatch) for more information.
                            struct Dummy {}
                        })
                            .unwrap()
                            .attrs,
                    );
		    
                    let item_name = fn_name.clone();

		    let mname = match &modpath {
			Some(mpath) =>
			    quote! {
				concat!("::!__associated_fn:", #impl_name, ":" #mpath)
			    },
			None => quote! {
			    concat!(module_path!(), "::!__associated_fn:", #impl_name, ":", stringify!(#fn_name))
			},
		    };
		    let hotpatch_name = Ident::new(&format!("__HOTPATCH_EXPORT_{}", exnum), Span::call_site());
		    
		    quote! {
			#[doc(hidden)]
			#[no_mangle]
			pub static #hotpatch_name: hotpatch::HotpatchExport<fn#fargs -> #output_type> =
			    hotpatch::HotpatchExport::__new(
				#self_type :: #item_name,
				#mname,
				#sigtext,
			    );
		    }
                }
                _ => panic!("There's something in this impl block I can't hotpatch yet"),
            }
            }).collect();

    TokenStream::from(quote! {
	#fn_item
	#(#exports)*
    })
}

fn gather_info(item: ImplItemMethod) -> (syn::Type, syn::Type, ImplItemMethod, Ident, String) {
    let fn_name = item.sig.ident.clone();
    let output_type = if let Type(_, t) = &item.sig.output {
        *(t.clone())
    } else {
        syn::parse2::<syn::Type>(quote! {
            ()
        })
        .unwrap()
    };

    let mut ts = proc_macro2::TokenStream::new();
    output_type.to_tokens(&mut ts);

    let mut args = vec![];
    for i in 0..item.sig.inputs.len() {
        if let Typed(arg) = &item.sig.inputs[i] {
            args.push(arg.ty.clone());
        }
    }

    let fargs = syn::parse2::<syn::Type>(if args.len() == 0 {
        quote! {
            ()
        }
    } else {
        quote! {
            (#(#args),*,)
        }
    })
	.unwrap();

    
    let sigtext = format!(
        "fn({}) -> {}",
        item.sig
            .inputs
            .clone()
            .into_iter()
            .map(|input| {
                if let syn::FnArg::Typed(t) = input {
                    let mut ts = proc_macro2::TokenStream::new();
                    t.ty.to_tokens(&mut ts);
                    ts.to_string()
                } else {
                    panic!("self parameter is not allowed. Can't hotpatch methods (yet!)")
                }
            })
            .collect::<Vec<String>>()
            .join(", "), 
        ts
    );

    (fargs, output_type, item, fn_name, sigtext)
}

// TODO: is there a crate for this?
fn transform_self(impl_name: &str, farg: &mut syn::Type) {
    use syn::Type::*;
    match farg {
	Path(p) => {
	    if p.path.segments.first().map(|s| s.ident.to_string()) == Some("Self".to_owned()) {
		let span = p.path.segments.first().unwrap().ident.span();
		p.path.segments.first_mut().unwrap().ident = Ident::new(&impl_name, span);
	    }

	    // generics too
	    use syn::PathArguments::*;
	    for seg in p.path.segments.iter_mut() {
		match &mut seg.arguments {
		    AngleBracketed(args) => {
			for arg in args.args.iter_mut() {
			    use syn::GenericArgument::*;
			    match arg {
				Type(t) => transform_self(impl_name, t),
				Binding(b) => transform_self(impl_name, &mut b.ty),
				Constraint(c) => {
				    c.ident.span().unwrap().error("Can't hotpatch a non-fully-defined function")
					.help("Trait bounds in functions are not allowed")
					.help("Patchable items cannot be generic")
					.emit();
				},
				Const(_c) => todo!("The hotpatch dev was lazy and doesn't want to figure out how to do recursive type analysis on const generics. File an issue on the github repo: https://github.com/Shizcow/hotpatch."),
				Lifetime(_) => (),
			    }
			}
		    },
		    Parenthesized(p) => {
			for input in p.inputs.iter_mut() {
			    transform_self(impl_name, input);
			}
			use syn::ReturnType::*;
			match &mut p.output {
			    Type(_, t) => transform_self(impl_name, t),
			    Default => (),
			}
		    },
		    None => (),
		}
	    }
	},
	Reference(r) => {
	    transform_self(impl_name, &mut r.elem);
	},
	Group(g) => {
	    transform_self(impl_name, &mut g.elem);
	},
	BareFn(b) => {
	    for input in b.inputs.iter_mut() {
		transform_self(impl_name, &mut input.ty);
	    }
	    use syn::ReturnType::*;
	    match &mut b.output {
		Type(_, t) => transform_self(impl_name, t),
		Default => (),
	    }
	},
	TraitObject(d) => {
	    for bound in d.bounds.iter_mut() {
		if let syn::TypeParamBound::Trait(t) = bound {
		    // I can't think of a less stupid way to do this
		    let mut tpath = syn::Type::Path(syn::TypePath {
			qself: None,
			path: t.path.clone(),
		    });
		    transform_self(impl_name, &mut tpath);
		    if let syn::Type::Path(p) = tpath {
			t.path = p.path;
		    }
		}
	    }
	},
	ImplTrait(i) => {
	    for bound in i.bounds.iter_mut() {
		if let syn::TypeParamBound::Trait(t) = bound {
		    // I can't think of a less stupid way to do this
		    let mut tpath = syn::Type::Path(syn::TypePath {
			qself: None,
			path: t.path.clone(),
		    });
		    transform_self(impl_name, &mut tpath);
		    if let syn::Type::Path(p) = tpath {
			t.path = p.path;
		    }
		}
	    }
	},
	Array(a) => {
	    transform_self(impl_name, &mut a.elem);
	},
	Infer(_) => (),
	Macro(m) => 
	    m.mac.path.span().unwrap().error("Can't hotpatch an associated function/method with macro type arguements")
	    .help("Try this as a bare function (not inside an impl) instead")
	    .note("hotpatch is trying to make `Self` as a type work and can't guarentee this will pass through with macros")
	    .emit(),
	Never(_) => (),
	Paren(p) => {
	    transform_self(impl_name, &mut p.elem);
	},
	Ptr(p) => {
	    transform_self(impl_name, &mut p.elem);
	},
	Slice(p) => {
	    transform_self(impl_name, &mut p.elem);
	},
	Tuple(t) => {
	    for elem in t.elems.iter_mut() {
		transform_self(impl_name, elem);
	    }
	},
	Verbatim(_) => (), // not found in normal source code
	_ => (), // nonexhaustive
    }
}
