use patch_proc::patchable;

#[patchable]
fn foo(a: i32, b: i32) -> i32 {
    println!("I am from source. I have: ");
    0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(1, 2); // prints "I am from source"
    foo.hotpatch("../src_obj/target/debug/libsrc_obj.so")?;
    foo(2, 3); // prints something totally different
    Ok(())
}
