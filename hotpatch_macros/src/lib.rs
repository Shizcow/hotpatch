use proc_macro::TokenStream;
use syn::{ItemFn, parse::Nothing};
use std::sync::RwLock;

mod item_fn;

lazy_static::lazy_static! {
    static ref EXPORTNUM: RwLock<usize> = RwLock::new(0);
}

#[proc_macro_attribute]
pub fn patchable(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args

    if let Ok(fn_item) = syn::parse::<ItemFn>(input) {
	item_fn::patchable(fn_item)
    } else {
	panic!("I can't hotpatch this yet!");
    }
}

#[proc_macro_attribute]
pub fn patch(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args

    if let Ok(fn_item) = syn::parse::<ItemFn>(input) {
	item_fn::patch(fn_item)
    } else {
	panic!("I can't patch this yet!");
    }
}

