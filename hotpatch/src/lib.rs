#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(unsized_fn_params)]
#![feature(unsize)]
#![feature(coerce_unsized)]

//! Changing function definitions at runtime.
//!
//! This crate is primarily used to load new function definitions from shared
//! object files in an exceedingly easy way.
//!
//! ## Short Example
//! The following shows how
//! dead-simple this crate is to use:
//! ```
//! // main.rs
//! use hotpatch::patchable;
//!
//! #[patchable]
//! fn foo() { }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!   foo(); // does nothing
//!   foo.hotpatch_lib("libsomething.so")?;
//!   foo(); // does something totally different!
//!   Ok(())
//! }
//! ```
//! What about defining a patch? Also easy:
//! ```
//! // lib.rs
//! use hotpatch::patch;
//!
//! #[patch]
//! fn foo() { }
//! ```
//! For more examples see the [git repo](https://github.com/Shizcow/hotpatch).
//!
//! ## Features
//! For reference, this crate recognizes the following features:
//! - `allow-main`: Allow setting `main` as [`#[patchable]`](patchable). Only useful if using `#[start]` or `#[main]`.
//! - `redirect-main`: Same as `allow-main` but also generates a stub `#[main]` to call the [`Patchable`](Patchable).
//!   If you just want to hotpatch `main`, this is probably the right feature. Requires nightly and `#[feature(main)]`.
//! - `large-signatures`: Tweaks the variadic generics engine. See [`hotpatch_fn`](Patchable::hotpatch_fn).
//!
//! ## Warnings
//! Under normal operation, this crate provides type safety, thread safety,
//! namepace safety, and a whole bunch of other guarantees. However, use of this
//! crate is still playing with fire.
//!
//! The one thing that cannot be checked against is call stack safety. Because
//! [`Patchable`](Patchable) uses [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html)s
//! the current thread is blocked when trying to hotpatch a function.
//! This ensures that an out-of-date function body cannot be run. However if the
//! function being hotpatched is the current function or anywhere within the
//! call stack (eg patching a function that called the current function) a
//! deadlock will occur. Be careful!
//!
//! The `try` methods within [`Patchable`](Patchable) provide additional checks
//! for this, but may cause other problems in multithreaded environments.
//!
//! ## Bypassing Thread Safety
//! The previous section mentions being unable to hotpatch currently running functions.
//! This is a deliberate safety feature. However, it can be bypassed by using the
//! `force` methods within [`Patchable`](Patchable). This allows multiple
//! functions definitions to run at once. This is unsafe, but allows for some really
//! interesting things such as hotpatching `main`.

use simple_error::bail;
use std::sync::RwLock;

pub use hotpatch_macros::*;
#[doc(hidden)]
pub use once_cell::sync::Lazy;
use variadic_generics::*;

mod export;
pub use export::*;

/// Created by [`#[patchable]`](patchable). A functor capable of overwriting its
/// own function.
pub struct Patchable<TraitPtr> {
    lazy: Lazy<Option<RwLock<HotpatchImportInternal<TraitPtr>>>>,
}

#[doc(hidden)]
pub struct HotpatchImportInternal<TraitPtr> {
    current_ptr: Option<TraitPtr>,
    default_ptr: TraitPtr,
    sig: &'static str,
    lib: Option<libloading::Library>,
    mpath: &'static str,
}

impl<TraitPtr> HotpatchImportInternal<TraitPtr> {
    fn new(ptr: TraitPtr, mpath: &'static str, sig: &'static str) -> Self {
        Self {
            current_ptr: None,
            default_ptr: ptr,
            lib: None,
            sig,
            mpath: mpath.trim_start_matches(|c| c != ':'),
        }
    }
    fn clean(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.lib.is_some() {
            self.lib.take().unwrap().close()?;
        }
        Ok(())
    }
    fn restore_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_ptr = None;
        self.clean()
    }
    fn hotpatch_fn(&mut self, ptr: TraitPtr) -> Result<(), Box<dyn std::error::Error>> {
        self.current_ptr = Some(ptr);
        self.clean()
    }
    fn hotpatch_lib(&mut self, lib_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let lib = libloading::Library::new(lib_name)?;

            let mut i: usize = 0;

            loop {
                let symbol_name = format!("{}{}", "__HOTPATCH_EXPORT_", i);
                let exports: libloading::Symbol<*mut HotpatchExport<TraitPtr>> =
                    lib.get(symbol_name.as_bytes()).map_err(|_| {
                        format!(
                            "Hotpatch for {} failed: symbol not found in library {}",
                            self.mpath, lib_name
                        )
                    })?;
                let export_obj = Box::from_raw(*exports);
                if export_obj.symbol.trim_start_matches(|c| c != ':') == self.mpath {
                    // found the correct symbol
                    if self.sig != export_obj.sig {
                        bail!("Hotpatch for {} failed: symbol found but of wrong type. Expected {} but found {}", self.mpath, self.sig, export_obj.sig);
                    }
                    self.current_ptr = Some(export_obj.ptr);
                    self.clean()?;
                    self.lib = Some(lib);
                    break;
                }
                i += 1;
            }
        }
        Ok(())
    }
}

// passthrough methods
impl<TraitPtr> Patchable<TraitPtr> {
    #[doc(hidden)]
    pub const fn __new(ptr: fn() -> Option<RwLock<HotpatchImportInternal<TraitPtr>>>) -> Self {
        Self {
            lazy: Lazy::new(ptr),
        }
    }
    #[doc(hidden)]
    pub fn __new_internal(
        ptr: TraitPtr,
        mpath: &'static str,
        sig: &'static str,
    ) -> Option<RwLock<HotpatchImportInternal<TraitPtr>>> {
        Some(RwLock::new(HotpatchImportInternal::new(ptr, mpath, sig)))
    }

    /// Hotpatch this functor with functionality defined in `lib_name`.
    /// Will search a shared object `cdylib` file for [`#[patch]`](patch) exports,
    /// finding the definition that matches module path and signature.
    ///
    /// ## Example
    /// ```
    /// #[patchable]
    /// fn foo() {}
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///   foo(); // does something
    ///   foo.hotpatch_lib("libtest.so")?;
    ///   foo(); // does something else
    ///   Ok(())
    /// }
    /// ```
    pub fn hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
        self.lazy.as_ref().unwrap().write()?.hotpatch_lib(lib_name)
    }
    /// Like [`hotpatch_lib`](Patchable::hotpatch_lib) but uses
    /// [`RwLock::try_write`](https://doc.rust-lang.org/std/sync/struct.RwLock.html#method.try_write).
    pub fn try_hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
        self.lazy
            .as_ref()
            .unwrap()
            .try_write()?
            .hotpatch_lib(lib_name)
    }
    /// Like [`hotpatch_lib`](Patchable::hotpatch_lib) but uses
    /// unsafe features to completly bypass the
    /// [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
    /// Can be used to patch the current function or parent functions.
    /// **Use with caution**.
    pub unsafe fn force_hotpatch_lib(
        &self,
        lib_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + '_>> {
        let sref = self as *const Self as *mut Self;
        let mut rref = (*sref).lazy.take().unwrap();
        let reslt = rref.get_mut().unwrap().hotpatch_lib(lib_name);
        *(*sref).lazy = Some(rref);
        reslt
    }

    /// Hotpatch this functor back to its original definition.
    ///
    /// ## Example
    /// ```
    /// #[patchable]
    /// fn foo() {}
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///   foo(); // does A
    ///   foo.hotpatch_lib("libtest.so")?;
    ///   foo(); // does B
    ///   foo.restore_default();
    ///   foo(); // does A again
    ///   Ok(())
    /// }
    /// ```
    pub fn restore_default(&self) -> Result<(), Box<dyn std::error::Error + '_>> {
        self.lazy.as_ref().unwrap().write()?.restore_default()
    }
    /// Like [`restore_default`](Patchable::restore_default) but uses
    /// [`RwLock::try_write`](https://doc.rust-lang.org/std/sync/struct.RwLock.html#method.try_write).
    pub fn try_restore_default(&self) -> Result<(), Box<dyn std::error::Error + '_>> {
        self.lazy.as_ref().unwrap().try_write()?.restore_default()
    }
    /// Like [`restore_default`](Patchable::restore_default) but uses
    /// unsafe features to completly bypass the
    /// [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
    /// Can be used to patch the current function or parent functions.
    /// **Use with caution**.
    pub unsafe fn force_restore_default(&self) -> Result<(), Box<dyn std::error::Error + '_>> {
        let sref = self as *const Self as *mut Self;
        let mut rref = (*sref).lazy.take().unwrap();
        let reslt = rref.get_mut().unwrap().restore_default();
        *(*sref).lazy = Some(rref);
        reslt
    }
}

impl<UnboxedTraitPtr: ?Sized> Patchable<Box<UnboxedTraitPtr>> {
    pub fn hotpatch_fn<F, Ret: ?Sized>(&self, ptr: F) -> Result<(), Box<dyn std::error::Error + '_>>
    where
        F: Fn(&str) -> &Ret + Send + Sync + 'static,
        Box<UnboxedTraitPtr>: From<Box<dyn Fn(&str) -> &Ret + Send + Sync + 'static>>,
    {
        let pre_boxed: Box<dyn Fn(&str) -> &Ret + Send + Sync + 'static> = Box::new(ptr);
        self.lazy
            .as_ref()
            .unwrap()
            .write()?
            .hotpatch_fn(pre_boxed.into())
    }
}

va_expand_with_nil! { ($va_len:tt) ($($va_idents:ident),*) ($($va_indices:tt),*)
               impl<TraitPtr, Ret $(,$va_idents)*> FnOnce<($($va_idents,)*)> for Patchable<TraitPtr>
    where TraitPtr: Fn($($va_idents),*) -> Ret, {
               type Output = Ret;
               extern "rust-call" fn call_once(self, args: ($($va_idents,)*)) -> Ret {
                   let inner =
                       self.lazy
                       .as_ref()
                       .unwrap()
                       .read()
                       .unwrap();
           if inner.current_ptr.is_some() {
               inner.current_ptr.as_ref().unwrap().call(args)
           } else {
               inner.default_ptr.call(args)
           }
               }
               }
}
va_expand_with_nil! { ($va_len:tt) ($($va_idents:ident),*) ($($va_indices:tt),*)
               impl<TraitPtr, Ret $(,$va_idents)*> FnMut<($($va_idents,)*)> for Patchable<TraitPtr>
    where TraitPtr: Fn($($va_idents),*) -> Ret, {
               extern "rust-call" fn call_mut(&mut self, args: ($($va_idents,)*)) -> Ret {
                   let inner =
                       self.lazy
                       .as_ref()
                       .unwrap()
                       .read()
                       .unwrap();
           if inner.current_ptr.is_some() {
               inner.current_ptr.as_ref().unwrap().call(args)
           } else {
               inner.default_ptr.call(args)
           }
               }
               }
}
va_expand_with_nil! { ($va_len:tt) ($($va_idents:ident),*) ($($va_indices:tt),*)
               impl<TraitPtr, Ret $(,$va_idents)*> Fn<($($va_idents,)*)> for Patchable<TraitPtr>
    where TraitPtr: Fn($($va_idents),*) -> Ret, {
               extern "rust-call" fn call(&self, args: ($($va_idents,)*)) -> Ret {
                   let inner =
                       self.lazy
                       .as_ref()
                       .unwrap()
                       .read()
                       .unwrap();
           if inner.current_ptr.is_some() {
               inner.current_ptr.as_ref().unwrap().call(args)
           } else {
               inner.default_ptr.call(args)
           }
               }
               }
}
