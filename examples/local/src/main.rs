use hotpatch::patchable;

#[patchable]
fn foo(_: i32) {
    println!("I am from source foo.");
}

fn bar(_: (i32,)) {
    println!("Foo Becomes Bar");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {    
    foo(1);
    foo.hotpatch_fn(bar)?;
    foo(1);
    let a = 5;
    foo.hotpatch_closure(move |(_)| println!("Foo becomes anonymous {}", a))?;
    foo(1);
    Ok(())
}
