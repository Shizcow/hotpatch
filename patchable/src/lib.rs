#![feature(unboxed_closures)]
#![feature(fn_traits)]

pub use lazy_static::lazy_static;
use std::sync::RwLock;
use simple_error::bail;

struct PatchableInternal {
    ptr: fn() -> i32,
    libs: Vec<libloading::Library>, // TODO: make into a reference or RC or something
}

impl PatchableInternal {
    pub fn new(ptr: fn() -> i32) -> Self {
	Self{ptr, libs: vec![]}
    }
    pub fn hotpatch(&mut self, lib_name: &str, mpath: &str) -> Result<(), Box<dyn std::error::Error>> {
	unsafe {
	    let lib = 
		libloading::Library::new(lib_name).unwrap();
            let exports: libloading::Symbol<*mut phf::Map<&'static str, fn() -> i32>>
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

pub struct Patchable {
    r: RwLock<PatchableInternal>,
    mpath: &'static str,
}

impl Patchable {
    pub fn new(ptr: fn() -> i32, mpath: &'static str) -> Self {
	Self{r: RwLock::new(PatchableInternal::new(ptr)), mpath: mpath.trim_start_matches(|c| c!=':')}
    }
    pub fn hotpatch(&self, lib_name: &str) -> Result<(), Box<dyn std::error::Error + '_>> {
	self.r.write()?.hotpatch(lib_name, self.mpath)
    }
}
impl FnOnce<()> for Patchable {
    type Output = i32;
    extern "rust-call" fn call_once(self, _: ()) -> <Self as std::ops::FnOnce<()>>::Output {
	(self.r.read().unwrap().ptr)()
    }
}
impl FnMut<()> for Patchable {
    extern "rust-call" fn call_mut(&mut self, _: ()) -> <Self as std::ops::FnOnce<()>>::Output {
	(self.r.read().unwrap().ptr)()
    }
}
impl Fn<()> for Patchable {
    extern "rust-call" fn call(&self, _: ()) -> <Self as std::ops::FnOnce<()>>::Output {
	(self.r.read().unwrap().ptr)()
    }
}
