use hotpatch::*;

/// This is a struct
struct Foo {
    description: &'static str,
}
/// And this is where free associated items can be defined
#[patchable]
impl Foo {
    /// Here's one of them!
    fn new() -> Self {
        Self {
            description: "This object was created with the original definition",
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = Foo::new();
    println!("First description: {}", f.description);

    // Foo::bar();
    // Foo::bar.hotpatch_fn(|| println!("this is patch!"))?;
    // Foo::bar();
    // Foo::bar.hotpatch_lib("target/debug/libmethods_obj.so")?;
    // Foo::bar();
    Ok(())
}
