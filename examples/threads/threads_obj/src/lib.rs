use hotpatch::patch;

#[patch]
pub fn foo() -> &'static str {
    "Foo: Patched"
}
