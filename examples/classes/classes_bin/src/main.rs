use hotpatch::patchable;

struct Dummy {
    data: i32,
}

impl Dummy {
    pub fn new(data: i32) -> Self {
	Self{data}
    }
}

//#[patchable]
impl Dummy {
    hotpatch::lazy_static! {
	pub static ref foo: hotpatch::HotpatchImport<&Dummy, ()>
	    = hotpatch::HotpatchImport::new(pach_proc_inline_foo, concat!(module_path!(), "::", stringify!(foo)), "fn(&Dummy) -> ()");
    }
    fn patch_proc_source_foo(&self) {
	println!("Hello from source foo. My data is: {}", self.data);
    }
    fn pach_proc_inline_foo(&self) {
	self.patch_proc_source_foo()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let d = Dummy::new(1);
    //d.foo();
    //d.foo.hotpatch("target/debug/libclasses_obj.so")?;
    //d.foo();
    Ok(())
}
