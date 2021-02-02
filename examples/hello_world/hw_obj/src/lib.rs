use hotpatch::patch;

#[patch]
/// This is what a patch looks like. It's a normal function that can still be executed locally
pub fn foo() {
    println!("I am from patched foo.");
}

mod a {
    use hotpatch::patch;
    #[patch]
    pub fn bar(a: i32) {
        println!(
            "I am from patched bar. I have {} as an arg. I am module aware.",
            a
        );
    }
}
