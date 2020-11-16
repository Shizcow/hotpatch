use patchable::HotpatchExport;
use patch_proc::patch;

#[patch]
pub fn foo() {
    println!("Hello from foo");
}

#[patch]
pub fn bar() {
    println!("Hello from bar");
}
