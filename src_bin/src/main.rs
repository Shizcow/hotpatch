use patch_proc::patchable;

mod a {
    use patch_proc::patchable;
    #[patchable]
    fn bar(a: i32) {
	println!("I am from source bar. I have {} as an arg.", a);
    }
}
    
#[patchable]
fn foo() {
    println!("I am from source foo.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(); // prints "I am from source"
    foo.hotpatch("../src_obj/target/debug/libsrc_obj.so")?;
    foo(); // prints something totally different

    use crate::a::bar;
    bar(1);
    bar.hotpatch("../src_obj/target/debug/libsrc_obj.so")?;
    bar(2);
    Ok(())
}
