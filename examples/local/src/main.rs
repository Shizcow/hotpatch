#![feature(unboxed_closures)]

use hotpatch::*;

#[allow(non_upper_case_globals)]
static foo: Patchable<dyn Fn(&str) -> &str + 'static> = Patchable::__new(|| {
    // direct copy
    fn foo(a: &str) -> &str {
        println!("I am Foo {}", a);
        a
    }
    Patchable::__new_internal(
        Box::new(foo) as Box<dyn Fn(&str) -> &str>,
        "local::foo",
        "fn(i32) -> ()",
    )
});

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
