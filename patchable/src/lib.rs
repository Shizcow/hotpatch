#![feature(unboxed_closures)]
#![feature(fn_traits)]

pub use lazy_static::lazy_static;
use std::sync::RwLock;
use simple_error::bail;

struct PatchableInternal<Args, Ret> {
    ptr: fn(Args) -> Ret,
    libs: Vec<libloading::Library>, // TODO: make into a reference or RC or something
}

impl<Args: 'static, Ret: 'static> PatchableInternal<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret) -> Self {
	Self{ptr, libs: vec![]}
    }
    pub fn hotpatch(&mut self, lib_name: &str, mpath: &str) -> Result<(), Box<dyn std::error::Error>> {
	unsafe {
	    let lib = 
		libloading::Library::new(lib_name).unwrap();
            let exports: libloading::Symbol<*mut phf::Map<&'static str, fn(Args) -> Ret>>
		= lib.get(b"HOTPATCH_EXPORTS")?;
	    self.ptr = match (**exports).get(mpath) {
		Some(p) => *p,
		None => bail!(format!("Error, no symbol '{}' found in library {}", mpath, lib_name)),
	    };
	    self.libs.push(lib);
	}
	Ok(())
    }
}

pub struct Patchable<Args, Ret> {
    r: RwLock<PatchableInternal<Args, Ret>>,
    mpath: &'static str,
}

impl<Args: 'static, Ret: 'static> Patchable<Args, Ret> {
    pub fn new(ptr: fn(Args) -> Ret, mpath: &'static str) -> Self {
	Self{r: RwLock::new(PatchableInternal::new(ptr)), mpath: mpath.trim_start_matches(|c| c!=':')}
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
