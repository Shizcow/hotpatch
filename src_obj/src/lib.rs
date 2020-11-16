use patch_proc::patch;

#[patch]
pub fn foo() {
    println!("Hello from foo");
}

mod a {
    use patch_proc::patch;
    #[patch]
    pub fn bar() {
	println!("Hello from bar");
    }
}
