#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(unboxed_closures)]

use hotpatch::*;

#[allow(non_upper_case_globals)]
static foo: hotpatch::Patchable<
    fn(i32) -> (),
    Box<dyn Fn<(i32,), Output = ()> + Send + Sync + 'static>,
> = hotpatch::Patchable::__new(|| {
    // direct copy
    fn foo(_: i32) {
        println!("I am Foo");
    }
    hotpatch::Patchable::__new_internal(foo, "local::foo", "fn(i32) -> ()")
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
    foo(1);
    foo.hotpatch_fn(Box::new(bar))?;
    foo(1);
    let a = 5;
    foo.hotpatch_fn(Box::new(move |_: i32| {
        println!("Foo becomes anonymous {}", a)
    }))?;
    foo(1);
    Ok(())
}
