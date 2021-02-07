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
//! use hotpatch::*;
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
//! ## Methods and Free Functions
//!
//! `hotpatch` also works with methods.
//!
//! **Note:** `hotpatch` currently only works with free functions (methods without a self parameter).
//! Actual method support is in progress and expected in the next version.
//!
//! **Note:** [`#[patchable]`](patchable) and [`#[patch]`](patch) must be placed __outside__ of `impl` bodies!
//!
//! ### Example
//! Setting up is done like so:
//! ```
//! // main.rs
//! use hotpatch::*;
//!
//! struct Foo {}
//! #[patchable]
//! impl Foo {
//!     pub fn bar() {
//!         println!("I can be changed!");
//!     }
//! }
//! ```
//! Patches are declared like this:
//! ```
//! struct Foo {}
//!
//! #[patch]
//! impl Foo {
//!     /// remember, #[patch] is top-level
//!     pub fn bar() {
//!         println!("this is external!");
//!     }
//! }
//! ```
//!
//! Finally, patching is done as so.
//! ```
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     Foo::bar();
//!     Foo::bar.hotpatch_fn(|| println!("this is patch!"))?;
//!     Foo::bar();
//!     Foo::bar.hotpatch_lib("target/debug/libmethods_obj.so")?;
//!     Foo::bar();
//!     Ok(())
//! }
//! ```
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

use std::marker::PhantomData;

use simple_error::bail;
use std::sync::RwLock;

pub use hotpatch_macros::*;
#[doc(hidden)]
pub use once_cell::sync::Lazy;
use variadic_generics::*;

mod export;
pub use export::*;

mod docs;
pub use docs::*;

use std::mem::{transmute, transmute_copy};

type FnVoid = dyn Fn() -> () + Send + Sync + 'static;

macro_rules! va_largesig {
    ($va_len:tt, $va_idents:tt, $va_indices:tt, $($tt:tt)+) => {
	#[cfg(not(feature = "large-signatures"))]
	va_expand_with_nil! { $va_len $va_idents $va_indices $($tt)* }
	#[cfg(feature = "large-signatures")]
	va_expand_more_with_nil! { $va_len $va_idents $va_indices $($tt)* }
    }
}

/// Created by [`#[patchable]`](patchable). A functor capable of overwriting its
/// own function.
pub struct Patchable<RealType: ?Sized + Send + Sync + 'static> {
    lazy: Lazy<Option<RwLock<HotpatchImportInternal<RealType>>>>,
}

#[doc(hidden)]
pub struct HotpatchImportInternal<RealType: ?Sized + Send + Sync + 'static> {
    current_ptr: Box<FnVoid>,       // void pointer
    default_ptr: Box<FnVoid>,       // void pointer
    phantom: PhantomData<RealType>, // store the real type for correct casts
    sig: &'static str,
    lib: Option<libloading::Library>,
    mpath: &'static str,
}

impl<RealType: ?Sized + Send + Sync + 'static> HotpatchImportInternal<RealType> {
    fn new<T>(ptr: T, mpath: &'static str, sig: &'static str) -> Self {
        // we know that ptr is a Box<'static raw fn ptr>, so it DOES impl Copy (kinda)
        // and because new is hidden, this assumption is safe
        let r = &ptr;
        unsafe {
            Self {
                current_ptr: transmute_copy(r),
                default_ptr: transmute_copy(r),
                phantom: PhantomData,
                lib: None,
                sig,
                mpath: mpath.trim_start_matches(|c| c != ':'),
            }
        }
    }
    fn clean(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.lib.is_some() {
            self.lib.take().unwrap().close()?;
        }
        Ok(())
    }
    fn restore_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // see Self::new for why this is safe
        self.current_ptr = unsafe { transmute_copy(&self.default_ptr) };
        self.clean()
    }
    fn upcast_self(&self) -> &Box<RealType> {
        let p: &Box<FnVoid> = &self.current_ptr;
        let casted: &Box<RealType> = unsafe { transmute(p) };
        casted
    }
}

// passthrough methods
impl<RealType: ?Sized + Send + Sync + 'static> Patchable<RealType> {
    #[doc(hidden)]
    pub const fn __new(ptr: fn() -> Option<RwLock<HotpatchImportInternal<RealType>>>) -> Self {
        Self {
            lazy: Lazy::new(ptr),
        }
    }
    #[doc(hidden)]
    pub fn __new_internal<T>(
        ptr: T,
        mpath: &'static str,
        sig: &'static str,
    ) -> Option<RwLock<HotpatchImportInternal<RealType>>> {
        Some(RwLock::new(HotpatchImportInternal::new(ptr, mpath, sig)))
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

trait HotpatchFnInternal<T, Dummy> {
    unsafe fn hotpatch_fn(&mut self, c: T) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(not(doc))]
va_largesig! { ($va_len:tt), ($($va_idents:ident),*), ($($va_indices:tt),*),
        impl<RealType: ?Sized + 'static, T, Ret, $($va_idents,)*> HotpatchFnInternal<T, (Ret, $($va_idents,)*)>
        for HotpatchImportInternal<RealType>
    where
        T: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,
        RealType: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,
        {
            unsafe fn hotpatch_fn(&mut self, c: T) -> Result<(), Box<dyn std::error::Error>> {
            let boxed: Box<T> = Box::new(c);
            let reboxed: Box<dyn Fn($($va_idents,)*) -> Ret> = boxed;
            let dbox: Box<FnVoid> = std::mem::transmute(reboxed);
            self.current_ptr = dbox;
            self.clean()
            }
        }
}
trait HotpatchLibInternal<Dummy> {
    fn hotpatch_lib(&mut self, lib_name: &str) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(not(doc))]
va_largesig! { ($va_len:tt), ($($va_idents:ident),*), ($($va_indices:tt),*),
        impl<RealType: ?Sized + Send + Sync + 'static, Ret: 'static, $($va_idents: 'static,)*> HotpatchLibInternal<(Ret, $($va_idents,)*)>
    for HotpatchImportInternal<RealType>
where
    RealType: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,
{
    fn hotpatch_lib(&mut self, lib_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let lib = libloading::Library::new(lib_name)?;

            let mut i: usize = 0;

            loop {
                let symbol_name = format!("{}{}", "__HOTPATCH_EXPORT_", i);
                let exports: libloading::Symbol<*mut HotpatchExport<fn($($va_idents,)*) -> Ret>> =
                    lib.get(symbol_name.as_bytes()).map_err(|_| {
                        format!(
                            "Hotpatch for {} failed: symbol not found in library {}",
                            self.mpath, lib_name
                        )
                    })?;
                let export_obj = &**exports;
                if export_obj.symbol.trim_start_matches(|c| c != ':') == self.mpath {
                    // found the correct symbol
                    if self.sig != export_obj.sig {
                        bail!("Hotpatch for {} failed: symbol found but of wrong type. Expected {} but found {}", self.mpath, self.sig, export_obj.sig);
                    }
                    let d: Box<fn($($va_idents,)*) -> Ret> = Box::new(export_obj.ptr);
                    let t: Box<dyn Fn($($va_idents,)*) -> Ret + Send + Sync + 'static> = d;
                    self.current_ptr = transmute(t);
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
}

/// Public interface for [Patchable::hotpatch_lib] and associated; requires import to use
pub trait HotpatchLib<Dummy> {
    fn hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>>;
    fn try_hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>>;
    unsafe fn force_hotpatch_lib(
        &self,
        lib_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + '_>>;
}

#[cfg(not(doc))]
va_largesig! { ($va_len:tt), ($($va_idents:ident),*), ($($va_indices:tt),*),
        impl<RealType: ?Sized + Send + Sync + 'static, Ret: 'static, $($va_idents: 'static,)*> HotpatchLib<(Ret, $($va_idents,)*)>
        for Patchable<RealType>
    where
        RealType: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,

        {
    fn hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
        self.lazy.as_ref().unwrap().write()?.hotpatch_lib(lib_name)
    }
    fn try_hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
        self.lazy
            .as_ref()
            .unwrap()
            .try_write()?
            .hotpatch_lib(lib_name)
    }
    unsafe fn force_hotpatch_lib(
        &self,
        lib_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + '_>> {
        let sref = self as *const Self as *mut Self;
        let mut rref = (*sref).lazy.take().unwrap();
        let reslt = rref.get_mut().unwrap().hotpatch_lib(lib_name);
        *(*sref).lazy = Some(rref);
        reslt
    }
        }
}

/// Public interface for [Patchable::hotpatch_fn] and associated; requires import to use
pub trait HotpatchFn<T, Dummy> {
    fn hotpatch_fn(&self, c: T) -> Result<(), Box<dyn std::error::Error + '_>>;
    fn try_hotpatch_fn(&self, c: T) -> Result<(), Box<dyn std::error::Error + '_>>;
    unsafe fn force_hotpatch_fn(&self, c: T) -> Result<(), Box<dyn std::error::Error + '_>>;
}

#[cfg(not(doc))]
va_largesig! { ($va_len:tt), ($($va_idents:ident),*), ($($va_indices:tt),*),
        impl<RealType: ?Sized + Send + Sync + 'static, T, Ret, $($va_idents,)*> HotpatchFn<T, (Ret, $($va_idents,)*)>
        for Patchable<RealType>
    where
        T: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,
        RealType: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,
        {
            fn hotpatch_fn(&self, c: T) -> Result<(), Box<dyn std::error::Error + '_>> {
            unsafe { self.lazy.as_ref().unwrap().write()?.hotpatch_fn(c) }
            }
            fn try_hotpatch_fn(&self, c: T) -> Result<(), Box<dyn std::error::Error + '_>> {
            unsafe { self.lazy.as_ref().unwrap().try_write()?.hotpatch_fn(c) }
            }
            unsafe fn force_hotpatch_fn(&self, c: T) -> Result<(), Box<dyn std::error::Error + '_>> {
            let sref = self as *const Self as *mut Self;
            let mut rref = (*sref).lazy.take().unwrap();
            let reslt = rref.get_mut().unwrap().hotpatch_fn(c);
            *(*sref).lazy = Some(rref);
            reslt
            }
        }
}

// Fn Traits
#[cfg(not(doc))]
va_largesig! { ($va_len:tt), ($($va_idents:ident),*), ($($va_indices:tt),*),
                impl<RealType: ?Sized + 'static, Ret, $($va_idents,)*> FnOnce<($($va_idents,)*)> for Patchable<RealType>
    where
                RealType: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,
                {
                type Output = Ret;
                    extern "rust-call" fn call_once(self, args: ($($va_idents,)*)) -> Ret {
            let inner = self.lazy.as_ref().unwrap().read().unwrap();
                    inner.upcast_self().call(args)
                }
                }
}
#[cfg(not(doc))]
va_largesig! { ($va_len:tt), ($($va_idents:ident),*), ($($va_indices:tt),*),
                impl<RealType: ?Sized + 'static, Ret, $($va_idents,)*> FnMut<($($va_idents,)*)> for Patchable<RealType>
    where
                RealType: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,
                {
                extern "rust-call" fn call_mut(&mut self, args: ($($va_idents,)*)) -> Ret {
                    let inner = self.lazy.as_ref().unwrap().read().unwrap();
                    inner.upcast_self().call(args)
                }
                }
}
#[cfg(not(doc))]
va_largesig! { ($va_len:tt), ($($va_idents:ident),*), ($($va_indices:tt),*),
                impl<RealType: ?Sized + 'static, Ret, $($va_idents,)*> Fn<($($va_idents,)*)> for Patchable<RealType>
    where
                RealType: Fn($($va_idents,)*) -> Ret + Send + Sync + 'static,
                {
                extern "rust-call" fn call(&self, args: ($($va_idents,)*)) -> Ret {
                    let inner = self.lazy.as_ref().unwrap().read().unwrap();
                    inner.upcast_self().call(args)
                }
                }
}

// methods stuff
pub struct MutConst<T: 'static> {
    f: fn() -> &'static T,
}
impl<T> MutConst<T> {
    pub const fn new(f: fn() -> &'static T) -> Self {
        Self { f }
    }
}
impl<T> std::ops::Deref for MutConst<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        (self.f)()
    }
}
