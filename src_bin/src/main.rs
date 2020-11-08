use patch_proc::patchable;

#[patchable]
fn foo() -> i32 {
    println!("I am from source");
    0
}

#[patchable]
fn bar() -> f64 {
    1.0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(); // prints "I am from source"
    foo.hotpatch("../src_obj/target/debug/libsrc_obj.so")?;
    foo(); // prints something totally different
    println!("{}", bar());
    Ok(())
}
