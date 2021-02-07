use hotpatch::*;

/// There is a system of trust here
/// Foo is assumed to be the same struct everywhere
/// This may be possible to lock down even more with typeid, but that's WIP upstream
struct Foo {}

#[patch]
impl Foo {
    /// remember, #[patch] is top-level
    pub fn bar() {
        println!("this is external!");
    }
}
