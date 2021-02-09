use hotpatch::*;

/// This is a struct
struct Foo {
    description: &'static str,
}
/// And this is where free associated items can be defined
#[patchable]
impl Foo {
    fn typetest(_m: &mut Result<Self, &Self>) {
        unimplemented!();
    }
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

    Foo::new.hotpatch_fn(|| Foo {
        description: "Created with an anonymous definition",
    })?;
    let f = Foo::new();
    println!("Second description: {}", f.description);

    Foo::new.hotpatch_lib("target/debug/libmethods_obj.so")?;
    let f = Foo::new();
    println!("Third description: {}", f.description);
    Ok(())
}
