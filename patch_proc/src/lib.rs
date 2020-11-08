use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{ItemFn, parse::Nothing, Ident, ReturnType::Type, FnArg::Typed};

#[proc_macro_attribute]
pub fn patchable(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args
    let mut item = syn::parse::<ItemFn>(input).unwrap();
    let fn_name = item.sig.ident.clone();
    let modpathname = Ident::new(&format!("patch_proc_mod_path_{}", fn_name),
				 Span::call_site());
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

    let mut args = vec![];
    for i in 0..item.sig.inputs.len() {
	if let Typed(arg) = &item.sig.inputs[i] {
	    args.push(arg.ty.clone());
	}
    }
    
    TokenStream::from(quote!{
	const fn #modpathname() -> &'static str {
	    concat!(module_path!(), "::foo")
	}

	patchable::lazy_static! {
	    #[allow(non_upper_case_globals)] // ree
	    pub static ref #fn_name: patchable::Patchable<(#(#args),*), #output_type> = patchable::Patchable::new(#newident, #modpathname());
	}

	#item
    })
}
