use patch_proc::patchable;

#[patchable]
fn foo(args: (i32,)) -> i32 {
    println!("I am from source");
    0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(1); // prints "I am from source"
    foo.hotpatch("../src_obj/target/debug/libsrc_obj.so")?;
    foo(2); // prints something totally different
    Ok(())
}
