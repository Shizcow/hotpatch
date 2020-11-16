use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn::{ItemFn, parse::Nothing, Ident, ReturnType::Type, FnArg::Typed};
use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref EXPORTNUM: RwLock<usize> = RwLock::new(0);
}

#[proc_macro_attribute]
pub fn patch(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args
    let mut item = syn::parse::<ItemFn>(input).unwrap();
    let fn_name = item.sig.ident.clone();
    let mut inline_fn = item.clone();
    inline_fn.sig.ident = Ident::new(&format!("patch_proc_inline_{}", fn_name),
				     Span::call_site());
    let inlineident = inline_fn.sig.ident.clone();
    item.sig.ident = Ident::new(&format!("patch_proc_source_{}", fn_name),
				Span::call_site());
    let newident = item.sig.ident.clone();
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

    let exnum;
    { // scope is used so EXPORTNUM is unlocked faster
	let mut r = EXPORTNUM.write().unwrap();
	exnum = *r;
	*r += 1;
    }

    let hotpatch_name = Ident::new(&format!("__HOTPATCH_EXPORT_{}", exnum), Span::call_site());


    TokenStream::from(quote!{
	#[no_mangle]
	pub static #hotpatch_name: patchable::HotpatchExport<fn(#fargs) -> #output_type> =
	    patchable::HotpatchExport{ptr: #inlineident,
			   symbol: concat!(module_path!(), "::", stringify!(#fn_name)),
			   sig: #sigtext};

	#inline_fn

	#[inline(always)]
	#item
    })
}

#[proc_macro_attribute]
pub fn patchable(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args
    let mut item = syn::parse::<ItemFn>(input).unwrap();
    let fn_name = item.sig.ident.clone();
    let mut inline_fn = item.clone();
    inline_fn.sig.ident = Ident::new(&format!("patch_proc_inline_{}", fn_name),
				     Span::call_site());
    let inlineident = inline_fn.sig.ident.clone();
    item.sig.ident = Ident::new(&format!("patch_proc_source_{}", fn_name),
				Span::call_site());
    let newident = item.sig.ident.clone();
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

    TokenStream::from(quote!{
	patchable::lazy_static! {
	    #[allow(non_upper_case_globals)] // ree
	    pub static ref #fn_name: patchable::Patchable<#fargs, #output_type> = patchable::Patchable::new(#inlineident, concat!(module_path!(), "::", stringify!(#fn_name)), #sigtext);
	}

	#inline_fn

	#[inline(always)]
	#item
    })
	
}
