#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use syn::{ItemFn, parse::Nothing};
use std::sync::RwLock;

mod item_fn;

lazy_static::lazy_static! {
    static ref EXPORTNUM: RwLock<usize> = RwLock::new(0);
}

/// Transforms a function into a [`Patchable`](struct.Patchable.html) capable of having
/// its behavior redefined at runtime.
///
/// Takes a single optional arguement: `modpath`. Used to spoof the module
/// path.
///
/// ## Example
/// ```
/// #[patchable]
/// fn foo() {}
///
/// #[patchable(mymod::baz)] // will look for the function ::mymod::baz instead of ::bar
/// fn bar() {
///   foo(); // foo is callable, just as a functor
/// }
/// ```
#[proc_macro_attribute]
pub fn patchable(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args

    if let Ok(fn_item) = syn::parse::<ItemFn>(input) {
	item_fn::patchable(fn_item)
    } else {
	panic!("I can't hotpatch this yet!");
    }
}

/// Transforms a function into a [`HotpatchExport`](struct.HotpatchExport.html) capable of
/// being exported and changing the behavior of a function in a seperate binary
/// at runtime. **The original function is preserved.** 
///
/// Takes a single optional arguement: `modpath`. Used to spoof the module
/// path.
///
/// ## Example
/// ```
/// #[patch]
/// fn foo() {}
///
/// #[patch(mymod::baz)] // looks like: mod mymod { fn baz() {} }
/// fn bar() {
///   foo(); // can still call foo
/// }
/// ```
#[proc_macro_attribute]
pub fn patch(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args

    if let Ok(fn_item) = syn::parse::<ItemFn>(input) {
	item_fn::patch(fn_item)
    } else {
	panic!("I can't patch this yet!");
    }
}

