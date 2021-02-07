use hotpatch::*;

/// This is a struct
struct Foo {}
/// And this is where free associated items can be defined
#[patchable]
impl Foo {
    /// Here's one of them!
    pub fn bar() {
        println!("this is passthrough!");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Foo::bar();
    Foo::bar.hotpatch_fn(|| println!("this is patch!"))?;
    Foo::bar();
    Foo::bar.hotpatch_lib("target/debug/libmethods_obj.so")?;
    Foo::bar();
    Ok(())
}
