use hotpatch::patch;

#[patch]
pub fn foo() -> &'static str {
    "Foo: Patched"
}


#[patch]
pub fn bar() -> &'static str {
    "Bar: Patched"
}


