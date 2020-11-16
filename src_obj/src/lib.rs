use patchable::HotpatchExport;
use patch_proc::patch;

#[patch]
pub fn foo() {
    println!("Hello from foo");
}

mod a {
    use patch_proc::patch;
    use patchable::HotpatchExport;
    #[patch]
    pub fn bar() {
	println!("Hello from bar");
    }
}
