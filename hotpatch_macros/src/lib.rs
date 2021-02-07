#![feature(proc_macro_diagnostic)]

//! You probably want documentation for the [`hotpatch`](https://docs.rs/hotpatch) crate.

use proc_macro::TokenStream;
use std::sync::RwLock;
use syn::{parse::Nothing, ItemFn, ItemImpl, Path};

mod item_fn;
mod item_impl;

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
    let modpath = get_modpath(attr);
    if modpath.is_err() {
        return TokenStream::new();
    }
    if let Ok(item) = syn::parse::<ItemFn>(input.clone()) {
        item_fn::patchable(item, modpath.unwrap())
    } else if let Ok(item) = syn::parse::<ItemImpl>(input) {
        item_impl::patchable(item, modpath.unwrap())
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
    let modpath = get_modpath(attr);
    if modpath.is_err() {
        return TokenStream::new();
    }
    if let Ok(fn_item) = syn::parse::<ItemFn>(input.clone()) {
        item_fn::patch(fn_item, modpath.unwrap())
    } else if let Ok(item) = syn::parse::<ItemImpl>(input) {
        item_impl::patch(item, modpath.unwrap())
    } else {
        panic!("I can't turn this into a patch yet!");
    }
}

fn get_modpath(attr: TokenStream) -> Result<Option<String>, ()> {
    if syn::parse::<Nothing>(attr.clone()).is_ok() {
        Ok(None)
    } else {
        let s = attr.to_string();
        if !s.starts_with("\"") && !s.ends_with("\"") {
            let path = syn::parse::<Path>(attr.clone());
            if path.is_err() {
                proc_macro::Span::call_site().error("Expected module path")
		    .help("Just use #[patchable]; it's already module aware.")
		    .help("If you're trying to spoof a module path, the supplied arguement is an invalid path")
		    .emit();
                return Err(());
            }
        }
        Ok(Some(s.replace(" ", "")))
    }
}
