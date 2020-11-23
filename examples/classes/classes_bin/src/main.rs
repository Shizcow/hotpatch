use hotpatch::patchable;

struct Dummy {
    data: i32,
}

impl Dummy {
    pub fn new(data: i32) -> Self {
	Self{data}
    }
}

#[patchable]
impl Dummy {
    pub fn foo(&self) {
	println!("Hello from source foo. My data is: {}", self.data);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let d = Dummy::new(1);
    d.foo();
    d.foo.hotpatch("target/debug/libclasses_obj.so")?;
    d.foo();
    Ok(())
}
