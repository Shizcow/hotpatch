use hotpatch::patch;

#[patch]
pub fn foo() {
    println!("I am from patched foo.");
}

mod a {
    use hotpatch::patch;
    #[patch]
    pub fn bar(a: i32) {
	println!("I am from patched bar. I have {} as an arg. I am module aware.", a);
    }
}
