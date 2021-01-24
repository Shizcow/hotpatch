use hotpatch::*;

#[allow(non_upper_case_globals)]
static foo: hotpatch::Patchable<dyn Fn(i32) -> (), fn(i32) -> ()> =
    hotpatch::Patchable::__new(|| {
        // direct copy
        fn foo(_: i32) {
            println!("I am Foo");
        }
        hotpatch::Patchable::__new_internal(s, "local::foo", "fn(i32) -> ()")
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

fn bar(_: i32) {
    println!("Foo Becomes Bar");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // foo(1);
    // foo.hotpatch_fn(bar)?;
    // foo(1);
    // let a = 5;
    // foo.hotpatch_fn(move |_: i32| println!("Foo becomes anonymous {}", a))?;
    // foo(1);
    Ok(())
}
