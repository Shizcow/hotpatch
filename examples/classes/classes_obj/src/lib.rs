use hotpatch::patchable;

// struct definitions REALLY should be shared. You're playing with fire!
struct Dummy {
    data: i32,
}

impl Dummy {
    pub fn new(data: i32) -> Self {
	Self{data}
    }
}

#[patch]
impl Dummy {
    pub fn foo(&self) {
	println!("Hello from patched foo. My data is: {}", self.data);
    }
}
