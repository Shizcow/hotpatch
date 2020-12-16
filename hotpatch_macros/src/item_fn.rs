use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn::{ItemFn, Ident, ReturnType::Type, FnArg::Typed};

use crate::EXPORTNUM;

pub fn patchable(fn_item: ItemFn) -> TokenStream {
    let (fargs, output_type, mut fn_name, sigtext, mut item, targs)
	= gather_info(fn_item);

    if !cfg!(feature = "allow-main") && fn_name == "main" {
	fn_name.span().unwrap().error("Attempted to set main as patchable")
	    .note("calling main.hotpatch() would cause a deadlock")
	    .help("enable the 'allow-main' feature in hotpatch to ignore (I hope you're using #[main] or #[start])")
	    .emit();
	return TokenStream::new();
    }

    item.attrs.append(&mut syn::parse2::<syn::ItemStruct>(quote!{
	///
	/// ---
	/// ## Hotpatch
	/// **Warning**: This item is [`#[patchable]`](hotpatch::patchable). Runtime behavior may not
	/// follow the source implementation. See the
	/// [Hotpatch Documentation](hotpatch) for more information.
	struct Dummy {}
    }).unwrap().attrs);

    let docitem = item.sig.clone();
    let doc_header = quote!{
	#[cfg(doc)]
	#docitem {}
    };

    let item_name = fn_name.clone();
    fn_name = Ident::new("__hotpatch_internal_fn_mangle_name", Span::call_site());
    item.sig.ident = fn_name.clone();
    
    TokenStream::from(quote!{
	#doc_header
	#[cfg(not(doc))]
	#[allow(non_upper_case_globals)]
	pub static #item_name: hotpatch::Patchable<#fargs, #output_type> = hotpatch::Patchable::__new(
	    || {
		#[inline(always)]
		#item
		hotpatch::Patchable::__new_internal(move |args| #fn_name #targs,
						    concat!(module_path!(), "::", stringify!(#fn_name)),
						    #sigtext)
	    });
    })
}

pub fn patch(fn_item: ItemFn) -> TokenStream {
    let (fargs, output_type, fn_name, sigtext, mut item, targs)
	= gather_info(fn_item);

    let exnum;
    { // scope is used so EXPORTNUM is unlocked faster
	let mut r = EXPORTNUM.write().unwrap();
	exnum = *r;
	*r += 1;
    }

    item.attrs.append(&mut syn::parse2::<syn::ItemStruct>(quote!{
	///
	/// ---
	/// ## Hotpatch
	/// This item is a [`#[patch]`](hotpatch::patch). It will silently define a public static
	/// symbol `__HOTPATCH_EXPORT_N` for use in shared object files. See the
	/// [Hotpatch Documentation](hotpatch) for more information.
	struct Dummy {}
    }).unwrap().attrs);

    let hotpatch_name = Ident::new(&format!("__HOTPATCH_EXPORT_{}", exnum), Span::call_site());

    TokenStream::from(quote!{
	#item
	#[doc(hidden)]
	#[no_mangle]
	pub static #hotpatch_name: hotpatch::HotpatchExport<fn(#fargs) -> #output_type> =
	    hotpatch::HotpatchExport::__new(move |args| #fn_name #targs,
					    concat!(module_path!(), "::", stringify!(#fn_name)),
					    #sigtext);
    })
	
}

fn gather_info(item: ItemFn) -> (syn::Type, syn::Type, Ident, String, ItemFn, proc_macro2::TokenStream) {
    let fn_name = item.sig.ident.clone();
    let output_type = if let Type(_, t) = &item.sig.output {
	*(t.clone())
    } else {
	syn::parse2::<syn::Type>(quote!{
	    ()
	}).unwrap()
    };

    let mut ts = proc_macro2::TokenStream::new();
    output_type.to_tokens(&mut ts);

    let sigtext = format!("fn({}) -> {}", item.sig.inputs.clone().into_iter().map(
	|input| {
	    if let syn::FnArg::Typed(t) = input {
		let mut ts = proc_macro2::TokenStream::new();
		t.ty.to_tokens(&mut ts);
		ts.to_string()
	    } else {
		todo!() // give an error or something
	    }
	}
    ).collect::<Vec<String>>().join(", "), ts);

    let mut args = vec![];
    for i in 0..item.sig.inputs.len() {
	if let Typed(arg) = &item.sig.inputs[i] {
	    args.push(arg.ty.clone());
	}
    }

    let argnums = args.iter().enumerate().map(
	|(i, _)| syn::parse::<syn::LitInt>(i.to_string().parse::<TokenStream>().unwrap()).unwrap()
    ).collect::<Vec<syn::LitInt>>();
    
    let targs: proc_macro2::TokenStream = (quote!{
	(#(args.#argnums),*)
    }).into();

    let fargs = syn::parse2::<syn::Type>(
	if args.len() == 0 {quote!{
	    ()
	}} else {quote!{
	    (#(#args),*,)
	}}).unwrap();

    (fargs, output_type, fn_name, sigtext, item, targs)
}
