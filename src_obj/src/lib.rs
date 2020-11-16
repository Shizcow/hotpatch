use patch_proc::patch;

#[patch]
pub fn foo() {
    println!("Hello from foo");
}

mod a {
    use patch_proc::patch;
    #[patch]
    pub fn bar(a: i32) {
	println!("Hello from bar. I have {} as an arg.", a);
	crate::foo();
    }
}
