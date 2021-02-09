use hotpatch::*;

/// There is a system of trust here
/// Foo is assumed to be the same struct everywhere
/// This may be possible to lock down even more with typeid, but that's WIP upstream
pub struct Foo {
    pub description: &'static str,
}

#[patch]
impl Foo {
    /// remember, #[patch] is top-level
    pub fn new() -> Self {
        Self {
            description: "This object was created in a dynamic library",
        }
    }
}
