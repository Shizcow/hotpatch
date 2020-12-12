use hotpatch::patchable;

#[patchable]
fn foo() {
    println!("I am from source foo.");
}

fn bar(_: ()) {
    println!("Foo Becomes Bar");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {    
    foo();
    foo.hotpatch_fn(bar)?;
    foo();
    Ok(())
}
