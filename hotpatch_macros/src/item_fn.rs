use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn::{FnArg::Typed, Ident, ItemFn, ReturnType::Type};

use crate::EXPORTNUM;

pub fn patchable(fn_item: ItemFn, modpath: Option<String>) -> TokenStream {
    let (fargs, output_type, fn_name, sigtext, mut item) = gather_info(fn_item);

    if !cfg!(feature = "allow-main") && !cfg!(feature = "redirect-main") && fn_name == "main" {
        fn_name.span().unwrap().error("Attempted to set main as patchable")
	    .note("calling main.hotpatch() would cause a deadlock")
	    .help("enable the 'allow-main' feature if you're using #[main] or #[start]")
	    .help("enable the 'redirect-main' feature if you actually want main to be patchable (requires unsafe and nightly, read the docs on force functions)")
	    .emit();
        return TokenStream::new();
    }

    let vis = item.vis.clone(); // pass through pub

    item.attrs.append(
        &mut syn::parse2::<syn::ItemStruct>(quote! {
        ///
        /// ---
        /// ## Hotpatch
        /// **Warning**: This item is [`#[patchable]`](hotpatch::patchable). Runtime behavior may not
        /// follow the source implementation. See the
        /// [Hotpatch Documentation](hotpatch) for more information.
        struct Dummy {}
        })
        .unwrap()
        .attrs,
    );

    let docitem = item.clone();
    let doc_header = quote! {
    #[cfg(doc)]
    #docitem
    };

    let item_name = fn_name.clone();
    item.sig.ident = fn_name.clone();

    let redirected_main = if cfg!(feature = "redirect-main") && item_name == "main" {
        quote! {
            #[main]
            fn __hotpatch_redirect_main() -> #output_type {
            main()
            }
        }
    } else {
        quote! {}
    };

    let mname = match modpath {
        Some(mpath) => (quote! {concat!("::", #mpath)}),
        None => {
            quote! {
                concat!(module_path!(), "::", stringify!(#item_name))
            }
        }
    };

    TokenStream::from(quote! {
    #doc_header
    #[cfg(not(doc))]
    #[allow(non_upper_case_globals)]
    #vis static #item_name: hotpatch::Patchable<dyn Fn#fargs -> #output_type + Send + Sync + 'static> = hotpatch::Patchable::__new(
        || {
        #[inline(always)]
        #item
            hotpatch::Patchable::__new_internal(Box::new(#fn_name) as Box<dyn Fn#fargs -> #output_type + Send + Sync + 'static>,
                            #mname,
                            #sigtext)
        });
    #redirected_main
    })
}

pub fn patch(fn_item: ItemFn, modpath: Option<String>) -> TokenStream {
    let (fargs, output_type, fn_name, sigtext, mut item) = gather_info(fn_item);

    let exnum;
    {
        // scope is used so EXPORTNUM is unlocked faster
        let mut r = EXPORTNUM.write().unwrap();
        exnum = *r;
        *r += 1;
    }

    item.attrs.append(
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

    let hotpatch_name = Ident::new(&format!("__HOTPATCH_EXPORT_{}", exnum), Span::call_site());

    let mname = match modpath {
        Some(mpath) => (quote! {concat!("::", #mpath)}),
        None => {
            quote! {
                concat!(module_path!(), "::", stringify!(#fn_name))
            }
        }
    };

    TokenStream::from(quote! {
    #item
    #[doc(hidden)]
    #[no_mangle]
    pub static #hotpatch_name: hotpatch::HotpatchExport<fn(#fargs) -> #output_type> =
            hotpatch::HotpatchExport::__new(#fn_name,
                        #mname,
                        #sigtext);
    })
}

fn gather_info(item: ItemFn) -> (syn::Type, syn::Type, Ident, String, ItemFn) {
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
                    todo!() // give an error or something
                }
            })
            .collect::<Vec<String>>()
            .join(", "),
        ts
    );

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

    (fargs, output_type, fn_name, sigtext, item)
}
