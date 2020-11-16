#![feature(unboxed_closures)]
#![feature(fn_traits)]

#[derive(Copy, Clone)]
pub struct HotpatchExport<T> {
    pub symbol: &'static str, // field order is important
    pub sig: &'static str,
    pub ptr: T,
}

pub use lazy_static::lazy_static;
use std::sync::RwLock;
use simple_error::bail;

struct PatchableInternal<Args, Ret> {
    ptr: fn(Args) -> Ret,
    sig: &'static str,
    libs: Vec<libloading::Library>, // TODO: make into a reference or RC or something
}

impl<Args: 'static, Ret: 'static> PatchableInternal<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret, sig: &'static str) -> Self {
	Self{ptr, libs: vec![], sig}
    }
    pub fn hotpatch(&mut self, lib_name: &str, mpath: &str) -> Result<(), Box<dyn std::error::Error>> {
	unsafe {
	    let lib = 
		libloading::Library::new(lib_name).unwrap();
	    
	    let mut i: usize = 0;

	    loop {
		let symbol_name = format!("{}{}", "__HOTPATCH_EXPORT_", i);
		let exports: libloading::Symbol<*mut HotpatchExport<fn(Args) -> Ret>>
		    = lib.get(symbol_name.as_bytes()).map_err(
			|_| format!("Hotpatch for {} failed: symbol not found in library {}",
				    mpath, lib_name))?;
		let export_obj = &**exports;
		if export_obj.symbol == mpath { // found the correct symbol
		    if self.sig != export_obj.sig {
			bail!("Hotpatch for {} failed: symbol found but of wrong type. Expecter {} but found {}", mpath, self.sig, export_obj.sig);
		    }
		    self.ptr = export_obj.ptr;
		    self.libs.push(lib);
		    break;
		}
		i += 1;
	    }
	}
	Ok(())
    }
}

pub struct Patchable<Args, Ret> {
    r: RwLock<PatchableInternal<Args, Ret>>,
    mpath: &'static str,
}

impl<Args: 'static, Ret: 'static> Patchable<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret, mpath: &'static str, sig: &'static str) -> Self {
	Self{r: RwLock::new(PatchableInternal::new(ptr, sig)), mpath: mpath.trim_start_matches(|c| c!=':')}
    }
    pub fn hotpatch(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
	self.r.write()?.hotpatch(lib_name, self.mpath)
    }
}
impl<Args, Ret> FnOnce<Args> for Patchable<Args, Ret> {
    type Output = Ret;
    extern "rust-call" fn call_once(self, args: Args) -> <Self as std::ops::FnOnce<Args>>::Output {
	(self.r.read().unwrap().ptr)(args)
    }
}
impl<Args, Ret> FnMut<Args> for Patchable<Args, Ret> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> <Self as std::ops::FnOnce<Args>>::Output {
	(self.r.read().unwrap().ptr)(args)
    }
}
impl<Args, Ret> Fn<Args> for Patchable<Args, Ret> {
    extern "rust-call" fn call(&self, args: Args) -> <Self as std::ops::FnOnce<Args>>::Output {
	(self.r.read().unwrap().ptr)(args)
    }
}
