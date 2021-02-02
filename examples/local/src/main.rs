#![feature(unboxed_closures)]

use hotpatch::*;

#[patchable]
fn foo(a: &str) -> &str {
    println!("I am Foo {}", a);
    a
}

fn bar(_: &str) -> &str {
    println!("Foo Becomes Bar");
    ""
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo("1");
    foo.hotpatch_fn(bar)?;
    foo("2");
    let a = 5;
    foo.hotpatch_fn(move |_: &str| {
        println!("Foo becomes anonymous {}", a);
        ""
    })?;
    foo("3");
    Ok(())
}
