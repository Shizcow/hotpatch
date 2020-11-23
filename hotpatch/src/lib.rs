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
    current_ptr: fn(Args) -> Ret,
    default_ptr: fn(Args) -> Ret,
    sig: &'static str,
    lib: Option<libloading::Library>, // TODO: make into a reference or RC or something
}

impl<Args: 'static, Ret: 'static> HotpatchImportInternal<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret, sig: &'static str) -> Self {
	Self{current_ptr: ptr, default_ptr: ptr, lib: None, sig}
    }
    pub fn hotpatch(&mut self, lib_name: &str, mpath: &str) -> Result<(), Box<dyn std::error::Error>> {
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
		    self.current_ptr = export_obj.ptr;
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
    pub fn hotpatch(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
	self.r.write()?.hotpatch(lib_name, self.mpath)
    }
    pub fn restore_default(&self) -> Result<(), Box<dyn std::error::Error + '_>> {
	let r = &mut self.r.write()?;
	r.current_ptr = r.default_ptr;
	Ok(())
    }
}
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
