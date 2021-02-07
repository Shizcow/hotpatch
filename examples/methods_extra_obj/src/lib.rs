use hotpatch::*;

/// There is a system of trust here
/// Foo is assumed to be the same struct everywhere
/// This may be possible to lock down even more with typeid, but that's WIP upstream
struct Foo {}

// macro input:
#[patch]
impl Foo {
    /// DOCTEST_ME_HARDER
    pub fn bar() {
        println!("this is external!");
    }
}
