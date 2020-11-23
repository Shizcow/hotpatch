use hotpatch::patch;

#[patch]
pub fn foo() {
    println!("Multiple 1");
}
