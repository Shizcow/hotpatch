use patch_proc::patchable;

#[patchable]
fn bar() {
    println!("I am from source bar.");
}

#[patchable]
fn foo() {
    println!("I am from source foo.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(); // prints "I am from source"
    foo.hotpatch("../src_obj/target/debug/libsrc_obj.so")?;
    foo(); // prints something totally different
    
    bar();
    bar.hotpatch("../src_obj/target/debug/libsrc_obj.so")?;
    bar();
    Ok(())
}
