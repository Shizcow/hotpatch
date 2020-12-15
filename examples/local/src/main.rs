use hotpatch::patchable;

#[patchable]
fn foo(_: i32, _: f64) {
    println!("I am Foo");
}

fn bar(_: i32) {
    println!("Foo Becomes Bar");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(1);
    foo.hotpatch_fn(bar)?;
    foo(1);
    let a = 5;
    foo.hotpatch_fn(move |_: i32| println!("Foo becomes anonymous {}", a))?;
    foo(1);
    Ok(())
}
