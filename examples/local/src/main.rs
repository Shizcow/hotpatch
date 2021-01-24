use hotpatch::*;

#[allow(non_upper_case_globals)]
static foo: Patchable<fn(&str) -> &str, dyn Fn(&str) -> &str + Send + Sync + 'static> =
    Patchable::__new(|| {
        // direct copy
        fn foo(a: &str) -> &str {
            println!("I am Foo {}", a);
            a
        }
        Patchable::__new_internal(foo, "local::foo", "fn(i32) -> ()")
    });

// /// I'm a functor
// #[patchable]
// fn foo(_: i32) {
//     println!("I am Foo");
// }

// /// I'm a function with extra bits
// #[patch]
// fn tmp(_: i32) {

// }

fn bar(_: &str) -> &str {
    println!("Foo Becomes Bar");
    ""
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo("1");
    foo.ext_hotpatch_fn(bar)?;
    foo("2");
    let a = 5;
    foo.ext_hotpatch_fn(move |_: &str| {
        println!("Foo becomes anonymous {}", a);
        ""
    })?;
    foo("3");
    Ok(())
}
