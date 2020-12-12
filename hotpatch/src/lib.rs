#![feature(unboxed_closures)]
#![feature(fn_traits)]

use std::sync::RwLock;
use simple_error::bail;

pub use once_cell::sync::Lazy;
pub use hotpatch_macros::*;

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
    pub fn hotpatch_closure<F: Send + Sync + 'static>(&mut self, ptr: F)
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
			bail!("Hotpatch for {} failed: symbol found but of wrong type. Expecter {} but found {}", mpath, self.sig, export_obj.sig);
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

pub struct HotpatchImport<Args, Ret> {
    r: RwLock<HotpatchImportInternal<Args, Ret>>,
    mpath: &'static str,
}

impl<Args: 'static, Ret: 'static> HotpatchImport<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret, mpath: &'static str, sig: &'static str) -> Self {
	Self{r: RwLock::new(HotpatchImportInternal::new(ptr, sig)),
	     mpath: mpath.trim_start_matches(|c| c!=':')}
    }
    pub fn hotpatch_lib(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
	self.r.write()?.hotpatch_lib(lib_name, self.mpath)
    }
    pub fn restore_default(&self) -> Result<(), Box<dyn std::error::Error + '_>> {
	self.r.write()?.restore_default()
    }
}

#[macro_use]
extern crate variadic_generics;
va_expand_with_nil!{ ($va_len:tt) ($($va_idents:ident),*) ($($va_indices:tt),*)
	     impl<$($va_idents: 'static,)* Ret: 'static> HotpatchImport<($($va_idents,)*), Ret> {
		 pub fn hotpatch_closure<F: Send + Sync + 'static>(&self, ptr: F)
							     -> Result<(), Box<dyn std::error::Error + '_>>
		 where F: Fn($($va_idents),*) -> Ret {
		     self.r.write()?.hotpatch_closure(move |args| ptr.call(args))
		 }
	     }
}

/*impl<A: 'static, Ret: 'static> HotpatchImport<(A,), Ret> {
    pub fn experiment<F: Send + Sync + 'static>(&self, ptr: F)
			       -> Result<(), Box<dyn std::error::Error + '_>>
    where F: Fn(A) -> Ret {
	self.r.write()?.hotpatch_closure(move |args| ptr.call(args))
    }
}
impl<Ret: 'static> HotpatchImport<(()), Ret> {
    pub fn experiment<F: Send + Sync + 'static>(&self, ptr: F)
			       -> Result<(), Box<dyn std::error::Error + '_>>
    where F: Fn() -> Ret {
	self.r.write()?.hotpatch_closure(move |args| ptr.call(args))
    }
}*/
impl<Args, Ret> FnOnce<Args> for HotpatchImport<Args, Ret> {
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
	(self.r.read().unwrap().current_ptr)(args)
    }
}
impl<Args, Ret> FnMut<Args> for HotpatchImport<Args, Ret> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Ret {
	(self.r.read().unwrap().current_ptr)(args)
    }
}
impl<Args, Ret> Fn<Args> for HotpatchImport<Args, Ret> {
    extern "rust-call" fn call(&self, args: Args) -> Ret {
	(self.r.read().unwrap().current_ptr)(args)
    }
}
