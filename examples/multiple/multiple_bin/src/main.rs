use hotpatch::patchable;

#[patchable]
fn foo() {
    println!("Source");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo.hotpatch("target/debug/libmultiple_obj1.so")?;
    foo();
    foo.restore_default()?;
    //foo.hotpatch("target/debug/libmultiple_obj2.so")?;
    //foo();
    //foo.restore_default()?;
    Ok(())
}
