#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(const_fn_fn_ptr_basics)]

use std::sync::RwLock;
use simple_error::bail;

#[doc(hidden)] // TODO: invert and wrapper so that HotpatchImport is public facing
pub use once_cell::sync::Lazy;
use variadic_generics::*;
pub use hotpatch_macros::*;

/// Created by (`#[patch]`)[patch]. Internal use only.
///
/// Creates a public static `#[no_mangle]` instance to be imported in another
/// binary by (`Patchable::hotpatch()`)[Patchable::hotpatch].
pub struct HotpatchExport<T> {
    pub symbol: &'static str, // field order is important
    pub sig: &'static str,
    pub ptr: T,
}

struct HotpatchImportInternal<Args, Ret> {
    current_ptr: Box<dyn Fn(Args) -> Ret + Send + Sync + 'static>,
    default_ptr: fn(Args) -> Ret,
    sig: &'static str,
    lib: Option<libloading::Library>,
}

impl<Args: 'static, Ret: 'static> HotpatchImportInternal<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret, sig: &'static str) -> Self {
	Self{current_ptr: Box::new(ptr), default_ptr: ptr, lib: None, sig}
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
    pub fn hotpatch_lib(&mut self, lib_name: &str, mpath: &str) -> Result<(), Box<dyn std::error::Error>> {
	unsafe {
	    let lib = libloading::Library::new(lib_name)?;
	    
	    let mut i: usize = 0;

	    loop {
		let symbol_name = format!("{}{}", "__HOTPATCH_EXPORT_", i);
		let exports: libloading::Symbol<*mut HotpatchExport<fn(Args) -> Ret>>
		    = lib.get(symbol_name.as_bytes()).map_err(
			|_| format!("Hotpatch for {} failed: symbol not found in library {}",
				    mpath, lib_name))?;
		let export_obj = &**exports;
		if export_obj.symbol.trim_start_matches(|c| c!=':') == mpath { // found the correct symbol
		    if self.sig != export_obj.sig {
			bail!("Hotpatch for {} failed: symbol found but of wrong type. Expected {} but found {}", mpath, self.sig, export_obj.sig);
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

pub struct Patchable<Args, Ret> {
    pub lazy: Lazy<HotpatchImport<Args, Ret>>,
}

impl<Args: 'static, Ret: 'static> Patchable<Args, Ret> {
    pub const fn new(ptr: fn() -> HotpatchImport<Args, Ret>) -> Self {
	Self{lazy: Lazy::new(ptr)}
    }
    pub fn hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
	self.lazy.r.write()?.hotpatch_lib(lib_name, self.lazy.mpath)
    }
    pub fn restore_default(&self) -> Result<(), Box<dyn std::error::Error + '_>> {
	self.lazy.r.write()?.restore_default()
    }
}

pub struct HotpatchImport<Args, Ret> {
    r: RwLock<HotpatchImportInternal<Args, Ret>>,
    mpath: &'static str,
}

impl<Args: 'static, Ret: 'static> HotpatchImport<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret, mpath: &'static str, sig: &'static str) -> Self {
	Self{r: RwLock::new(HotpatchImportInternal::new(ptr, sig)),
	     mpath: mpath.trim_start_matches(|c| c!=':')}
    }
}

va_expand_with_nil!{ ($va_len:tt) ($($va_idents:ident),*) ($($va_indices:tt),*)
	     impl<$($va_idents: 'static,)* Ret: 'static> Patchable<($($va_idents,)*), Ret> {
		 pub fn hotpatch_fn<F: Send + Sync + 'static>(&self, ptr: F)
							     -> Result<(), Box<dyn std::error::Error + '_>>
		 where F: Fn($($va_idents),*) -> Ret {
		     self.lazy.r.write()?.hotpatch_fn(move |args| ptr.call(args))
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
	//std::ops::Fn::call(&self.r.read().unwrap().ptr, args)
	(self.lazy.r.read().unwrap().current_ptr)(args)
    }
}
impl<Args, Ret> FnMut<Args> for Patchable<Args, Ret> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Ret {
	(self.lazy.r.read().unwrap().current_ptr)(args)
    }
}
impl<Args, Ret> Fn<Args> for Patchable<Args, Ret> {
    extern "rust-call" fn call(&self, args: Args) -> Ret {
	(self.lazy.r.read().unwrap().current_ptr)(args)
    }
}
