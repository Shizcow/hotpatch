use hotpatch::patchable;

#[patchable]
fn foo() {
    println!("I am Foo");
}

fn bar() {
    println!("Foo Becomes Bar");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {    
    foo();
    foo.hotpatch_fn(bar)?;
    foo();
    let a = 5;
    foo.hotpatch_fn(move || println!("Foo becomes anonymous {}", a))?;
    foo();
    Ok(())
}
