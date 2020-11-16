use patchable::HotpatchExport;
use patch_proc::patch;

#[patch]
pub fn foo() {
    println!("Hello from foo");
}
