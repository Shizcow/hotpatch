use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn::{FnArg::Typed, Ident, ImplItemConst, ImplItemMethod, ItemImpl, ReturnType::Type};

pub fn patchable(mut fn_item: ItemImpl, modpath: Option<String>) -> TokenStream {
    fn_item.items = fn_item
        .items
        .drain(..)
        .map(|item| {
            match item {
                syn::ImplItem::Method(m) => {
                    let (fargs, output_type, mut item, mut fn_name) = gather_info(m);
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
                    fn_name = Ident::new("__hotpatch_internal_staticwrap", Span::call_site());
                    item.sig.ident = fn_name.clone();
		    let p_item = syn::parse2::<ImplItemConst>(quote! {
			#[cfg(not(doc))]
			#[allow(non_upper_case_globals)]
			#vis const #item_name: hotpatch::MutConst<Patchable<dyn Fn#fargs -> #output_type + Send + Sync + 'static>> =hotpatch::MutConst::new(|| {
			    #[patchable(#modpath)]
			    #item
			    &#fn_name
			});
		    }).unwrap();
		    (syn::ImplItem::Method(docitem), syn::ImplItem::Const(p_item))
                }
                _ => panic!("There's something in this impl block I can't hotpatch yet"),
            }
        }).fold(vec![], |mut acc, (c1, c2)| {acc.push(c1); acc.push(c2); acc});

    TokenStream::from(quote! {
    #fn_item
    })
}

pub fn gather_info(item: ImplItemMethod) -> (syn::Type, syn::Type, ImplItemMethod, Ident) {
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

    (fargs, output_type, item, fn_name)
}
