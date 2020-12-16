use hotpatch::*;

/// I'm a functor
#[patchable]
fn foo(_: i32) {
    println!("I am Foo");
}

/// I'm a function with extra bits
#[patch]
fn tmp(_: i32) {

}

fn bar(_: i32) {
    println!("Foo Becomes Bar");
}

fn baz(_: i32) {
    unsafe {
	foo.force_restore_default().unwrap();
    }
    println!("hello from baz");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(1);
    foo.hotpatch_fn(bar)?;
    foo(1);
    let a = 5;
    foo.hotpatch_fn(move |_: i32| println!("Foo becomes anonymous {}", a))?;
    foo(1);
    foo.hotpatch_fn(baz)?;
    foo(1);
    foo(1);
    Ok(())
}
