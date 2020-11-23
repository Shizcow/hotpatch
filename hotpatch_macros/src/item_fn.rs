use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn::{ItemFn, Ident, ReturnType::Type, FnArg::Typed};

use crate::EXPORTNUM;

pub fn patchable(fn_item: ItemFn) -> TokenStream {
    let (fargs, output_type, inlineident, fn_name, sigtext, inline_fn, item)
	= gather_info(fn_item, true);

    TokenStream::from(quote!{
	#[allow(non_upper_case_globals)]
	pub static #fn_name: hotpatch::Lazy<hotpatch::HotpatchImport<#fargs, #output_type>>
	    = hotpatch::Lazy::new(|| {
		#inline_fn
		#[inline(always)]
		#item
		hotpatch::HotpatchImport::new(#inlineident,
					      concat!(module_path!(), "::", stringify!(#fn_name)),
					      #sigtext)
	    });
    })
}

pub fn patch(fn_item: ItemFn) -> TokenStream {
    let (fargs, output_type, inlineident, fn_name, sigtext, inline_fn, item)
	= gather_info(fn_item, false);

    let exnum;
    { // scope is used so EXPORTNUM is unlocked faster
	let mut r = EXPORTNUM.write().unwrap();
	exnum = *r;
	*r += 1;
    }

    let hotpatch_name = Ident::new(&format!("__HOTPATCH_EXPORT_{}", exnum), Span::call_site());

    TokenStream::from(quote!{
	#[no_mangle]
	pub static #hotpatch_name: hotpatch::HotpatchExport<fn(#fargs) -> #output_type> =
	    hotpatch::HotpatchExport{ptr: #inlineident,
				     symbol: concat!(module_path!(), "::", stringify!(#fn_name)),
				     sig: #sigtext};

	#inline_fn // TODO: can this be put inside the above definition?

	#item
    })
	
}

fn gather_info(mut item: ItemFn, mangle_src: bool) -> (syn::Type, syn::Type, Ident, Ident, String, ItemFn, ItemFn) {
    let fn_name = item.sig.ident.clone();
    let mut inline_fn = item.clone();
    inline_fn.sig.ident = Ident::new(&format!("patch_proc_inline_{}", fn_name),
				     Span::call_site());
    let inlineident = inline_fn.sig.ident.clone();
    if mangle_src {
	item.sig.ident = Ident::new(&format!("patch_proc_source_{}", fn_name),
				    Span::call_site());
    }
    let newident = &item.sig.ident;
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
    
    *inline_fn.block = syn::parse2::<syn::Block>(quote!{
	{
	    #newident (#(args.#argnums),*)
	}
    }).unwrap();

    inline_fn.sig.inputs.clear();

    let fargs = syn::parse2::<syn::Type>(
	if args.len() == 0 {quote!{
	    ()
	}} else {quote!{
	    (#(#args),*,)
	}}).unwrap();
	
    inline_fn.sig.inputs.push(syn::parse2::<syn::FnArg>(quote!{
	args: #fargs
    }).unwrap());

    (fargs, output_type, inlineident, fn_name, sigtext, inline_fn, item)
}
