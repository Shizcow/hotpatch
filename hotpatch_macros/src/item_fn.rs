use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn::{ItemFn, Ident, ReturnType::Type, FnArg::Typed};

use crate::EXPORTNUM;

pub fn patchable(fn_item: ItemFn) -> TokenStream {
    let (fargs, output_type, fn_name, sigtext, item, targs)
	= gather_info(fn_item, true);

    if !cfg!(feature = "allow-main") && fn_name == "main" {
	fn_name.span().unwrap().error("Attempted to set main as patchable")
	    .note("calling main.hotpatch() would cause a deadlock")
	    .help(format!("enable the 'allow-main' feature in hotpatch to ignore (I hope you're using #[main] or #[start])"))
	    .emit();
	return TokenStream::new();
    }

    let newsg = item.sig.ident.clone();

    TokenStream::from(quote!{
	#[allow(non_upper_case_globals)]
	pub static #fn_name: hotpatch::Lazy<hotpatch::HotpatchImport<#fargs, #output_type>>
	    = hotpatch::Lazy::new(|| {
		#[inline(always)]
		#item
		hotpatch::HotpatchImport::new(move |args| #newsg #targs,
					      concat!(module_path!(), "::", stringify!(#fn_name)),
					      #sigtext)
	    });
    })
}

pub fn patch(fn_item: ItemFn) -> TokenStream {
    let (fargs, output_type, fn_name, sigtext, item, targs)
	= gather_info(fn_item, false);

    let exnum;
    { // scope is used so EXPORTNUM is unlocked faster
	let mut r = EXPORTNUM.write().unwrap();
	exnum = *r;
	*r += 1;
    }
    
    let newsg = item.sig.ident.clone();

    let hotpatch_name = Ident::new(&format!("__HOTPATCH_EXPORT_{}", exnum), Span::call_site());

    TokenStream::from(quote!{
	#[no_mangle]
	pub static #hotpatch_name: hotpatch::HotpatchExport<fn(#fargs) -> #output_type> =
	    hotpatch::HotpatchExport{ptr: move |args| #newsg #targs,
				     symbol: concat!(module_path!(), "::", stringify!(#fn_name)),
				     sig: #sigtext};

	#item
    })
	
}

fn gather_info(mut item: ItemFn, mangle_src: bool) -> (syn::Type, syn::Type, Ident, String, ItemFn, proc_macro2::TokenStream) {
    let fn_name = item.sig.ident.clone();
    if mangle_src {
	item.sig.ident = Ident::new(&format!("patch_proc_source_{}", fn_name),
				    Span::call_site());
    }
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
