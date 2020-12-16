#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(const_fn_fn_ptr_basics)]

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
//! - `allow-main`: Allow setting `main` as [`#[patchable]`](patchable). Use with caution.
//! - `large-signatures`: Tweaks the variadic generics engine. See [`hotpatch_fn`](Patchable::hotpatch_fn).
//!
//! ## Warnings
//! Under normal operation, this crate provides type saftey, thread saftey,
//! namepace saftey, and a whole bunch of other guarentees. However, use of this
//! crate is still playing with fire.
//!
//! The one thing that cannot be checked against is call stack saftey. Because
//! [`Patchable`](Patchable) uses [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html)s
//! the current thread is blocked when trying to hotpatch a function.
//! This ensures that an out-of-date function body cannot be ran. However if the
//! function being hotpatched is the current function or anywhere within the
//! call stack (eg patching a function that called the current function) a
//! deadlock will occur. Be careful!
//!
//! The `try` methods within [`Patchable`](Patchable) provide additional checks
//! for this, but may cause other problems in multithreaded environments.
//!
//! ## Bypassing Thread Saftey
//! The previous section mentions being unable to hotpatch currently running functions.
//! This is a deliberate saftey feature. However, it can be bypassed by using the
//! `force` methods within [`Patchable`](Patchable). This allows multiple
//! functions definitions to run at once. This is unsafe, but allows for some really
//! interesting things, such as hotpatching `main`.

use std::sync::RwLock;
use simple_error::bail;

#[doc(hidden)]
pub use once_cell::sync::Lazy;
use variadic_generics::*;
pub use hotpatch_macros::*;

/// Created by [`#[patch]`](patch). Internal use only.
///
/// Creates a `#[no_mangle] pub static` instance to be imported in another
/// binary by [`Patchable`](Patchable) methods.
pub struct HotpatchExport<T> {
    symbol: &'static str, // field order is important
    sig: &'static str,
    ptr: T,
}

impl<T> HotpatchExport<T> {
    #[doc(hidden)]
    pub const fn __new(ptr: T, symbol: &'static str, sig: &'static str) -> Self {
	Self{symbol, sig, ptr}
    }
}

/// Created by [`#[patchable]`](patchable). A functor capable of overwriting its
/// own function.
pub struct Patchable<Args, Ret> {
    lazy: Lazy<Option<RwLock<HotpatchImportInternal<Args, Ret>>>>,
}

#[doc(hidden)]
pub struct HotpatchImportInternal<Args, Ret> {
    current_ptr: Box<dyn Fn(Args) -> Ret + Send + Sync + 'static>,
    default_ptr: fn(Args) -> Ret,
    sig: &'static str,
    lib: Option<libloading::Library>,
    mpath: &'static str,
}

impl<Args: 'static, Ret: 'static> HotpatchImportInternal<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret, sig: &'static str, mpath: &'static str) -> Self {
	Self{current_ptr: Box::new(ptr), default_ptr: ptr, lib: None, sig, mpath: mpath.trim_start_matches(|c| c!=':')}
    }
    fn clean(&mut self) -> Result<(), Box<dyn std::error::Error>> {
	if self.lib.is_some() {
	    self.lib.take().unwrap().close()?;
	}
	Ok(())
    }
    pub fn restore_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
	self.current_ptr = Box::new(self.default_ptr);
	self.clean()
    }
    pub fn hotpatch_fn<F: Send + Sync + 'static>(&mut self, ptr: F)
			       -> Result<(), Box<dyn std::error::Error>>
    where F: Fn(Args) -> Ret {
	self.current_ptr = Box::new(ptr);
	self.clean()
    }
    pub fn hotpatch_lib(&mut self, lib_name: &str) -> Result<(), Box<dyn std::error::Error>> {
	unsafe {
	    let lib = libloading::Library::new(lib_name)?;
	    
	    let mut i: usize = 0;

	    loop {
		let symbol_name = format!("{}{}", "__HOTPATCH_EXPORT_", i);
		let exports: libloading::Symbol<*mut HotpatchExport<fn(Args) -> Ret>>
		    = lib.get(symbol_name.as_bytes()).map_err(
			|_| format!("Hotpatch for {} failed: symbol not found in library {}",
				    self.mpath, lib_name))?;
		let export_obj = &**exports;
		if export_obj.symbol.trim_start_matches(|c| c!=':') == self.mpath { // found the correct symbol
		    if self.sig != export_obj.sig {
			bail!("Hotpatch for {} failed: symbol found but of wrong type. Expected {} but found {}", self.mpath, self.sig, export_obj.sig);
		    }
		    self.current_ptr = Box::new(export_obj.ptr);
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
impl<Args: 'static, Ret: 'static> Patchable<Args, Ret> {
    #[doc(hidden)]
    pub const fn __new(ptr: fn() -> Option<RwLock<HotpatchImportInternal<Args, Ret>>>) -> Self {
	Self{lazy: Lazy::new(ptr)}
    }
    #[doc(hidden)]
    pub fn __new_internal(ptr: fn(Args) -> Ret, mpath: &'static str, sig: &'static str) -> Option<RwLock<HotpatchImportInternal<Args, Ret>>> {
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
	self.lazy.as_ref().unwrap().try_write()?.hotpatch_lib(lib_name)
    }
    /// Like [`hotpatch_lib`](Patchable::hotpatch_lib) but uses
    /// unsafe features to completly bypass the
    /// [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
    /// Can be used to patch the current function or parent functions.
    /// **Use with caution**.
    pub unsafe fn force_hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
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

#[cfg(doc)]
impl<VaGen: 'static, Ret: 'static> Patchable<VaGen, Ret> {
    /// Hotpatch this functor with functionality defined in `ptr`.
    /// `ptr` can be a function pointer or `move` closure with the
    /// same type signature as the functor's function.
    ///
    /// ## Example
    /// ```
    /// #[patchable]
    /// fn foo(_: i32, _: i32, _: i32) {}
    ///
    /// fn bar(_: i32, _: i32, _: i32) {}
    /// 
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///   foo.hotpatch_fn(bar)?;
    ///   foo.hotpatch_fn(move |a, b, c| println!("{} {} {}", a, b, c))?;
    ///   Ok(())
    /// }
    /// ```
    ///
    /// ## VaArgs Note
    /// Implementation is defined with the [`variadic_generics`](https://docs.rs/variadic_generics)
    /// crate. This means
    /// a macro is used to define a finite but large number of templated inputs.
    /// If using functions with large numbers of inputs and `hotpatch_fn` does not
    /// appear to be defined, compile `hotpatch` with the `large-signatures` feature
    /// to increase the number of supported arguements.
    ///
    /// This is the only place where `large_signatures` is needed. Large signature
    /// functions are supported out of the box for [`hotpatch_lib`](Patchable::hotpatch_lib) and
    /// [`restore_default`](Patchable::restore_default).
    pub fn hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F) ->
	Result<(), Box<dyn std::error::Error + '_>>
    where F: Fn(VaGen) -> Ret {
	// The actual implementation is below
    }
    /// Like [`hotpatch_fn`](Patchable::hotpatch_fn) but uses
    /// [`RwLock::try_write`](https://doc.rust-lang.org/std/sync/struct.RwLock.html#method.try_write).
    pub fn try_hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F) ->
	Result<(), Box<dyn std::error::Error + '_>>
    where F: Fn(VaGen) -> Ret {
	// The actual implementation is below
    }
    /// Like [`hotpatch_fn`](Patchable::hotpatch_fn) but uses
    /// unsafe features to completly bypass the
    /// [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
    /// Can be used to patch the current function or parent functions.
    /// **Use with caution**.
    pub unsafe fn force_hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F) ->
	Result<(), Box<dyn std::error::Error + '_>>
    where F: Fn(VaGen) -> Ret {
	// The actual implementation is below
    }
}
#[cfg(not(doc))]
#[cfg(not(feature = "large-signatures"))]
va_expand_with_nil!{ ($va_len:tt) ($($va_idents:ident),*) ($($va_indices:tt),*)
		      impl<$($va_idents: 'static,)* Ret: 'static> Patchable<($($va_idents,)*), Ret> {
	pub fn hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F) ->
	    Result<(), Box<dyn std::error::Error + '_>>
	where F: Fn($($va_idents),*) -> Ret {
	    self.lazy.as_ref().unwrap().write()?.hotpatch_fn(move |args| ptr.call(args))
	}
	pub fn try_hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F) ->
	    Result<(), Box<dyn std::error::Error + '_>>
	where F: Fn($($va_idents),*) -> Ret {
	    self.lazy.as_ref().unwrap().try_write()?.hotpatch_fn(move |args| ptr.call(args))
	}
	pub unsafe fn force_hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F) ->
	    Result<(), Box<dyn std::error::Error + '_>>
	where F: Fn($($va_idents),*) -> Ret {
	    let sref = self as *const Self as *mut Self;
	    let mut rref = (*sref).lazy.take().unwrap();
	    let reslt = rref.get_mut().unwrap().hotpatch_fn(move |args| ptr.call(args));
	    *(*sref).lazy = Some(rref);
	    reslt
	}
    }
}
#[cfg(not(doc))]
#[cfg(feature = "large-signatures")]
va_expand_more_with_nil!{ ($va_len:tt) ($($va_idents:ident),*) ($($va_indices:tt),*)
		      impl<$($va_idents: 'static,)* Ret: 'static> Patchable<($($va_idents,)*), Ret> {
			  pub fn hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F) ->
			      Result<(), Box<dyn std::error::Error + '_>>
			  where F: Fn($($va_idents),*) -> Ret {
			      self.lazy.as_ref().unwrap().write()?.hotpatch_fn(move |args| ptr.call(args))
			  }
			  pub fn try_hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F) ->
			      Result<(), Box<dyn std::error::Error + '_>>
			  where F: Fn($($va_idents),*) -> Ret {
			      self.lazy.as_ref().unwrap().try_write()?.hotpatch_fn(move |args| ptr.call(args))
			  }
		      }
}

impl<Args, Ret> FnOnce<Args> for Patchable<Args, Ret> {
    type Output = Ret;
    extern "rust-call" fn call_once(self, args: Args) -> Ret {
	// When variadic generics are imlemented the following line can be used
	// to avoid the layer of indirection associated with a function having
	// a tuple list as an arguement. The current bottleneck is getting the
	// type bounds for variadic arguements on HotpatchImportInternal. Currently, a single
	// type arguement tuple is used to give a constant number of arguements.
	// When variadic template arguements are introduced, the stored function pointer
	// will be type-aware.
	//self.lazy.unwrap().read().unwrap().current_ptr.call(args)
	(self.lazy.as_ref().unwrap().read().unwrap().current_ptr)(args)
    }
}
impl<Args, Ret> FnMut<Args> for Patchable<Args, Ret> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Ret {
	(self.lazy.as_ref().unwrap().read().unwrap().current_ptr)(args)
    }
}
impl<Args, Ret> Fn<Args> for Patchable<Args, Ret> {
    extern "rust-call" fn call(&self, args: Args) -> Ret {
	(self.lazy.as_ref().unwrap().read().unwrap().current_ptr)(args)
    }
}
