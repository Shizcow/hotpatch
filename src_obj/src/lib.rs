use patchable::HotpatchExport;
use patch_proc::patch;

#[patch]
pub fn foo() -> i32 {
    println!("Hello from foo");
    1
}
