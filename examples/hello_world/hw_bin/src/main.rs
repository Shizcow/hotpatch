use hotpatch::patchable;

mod a {
    use hotpatch::patchable;
    #[patchable]
    pub fn bar(a: i32) {
        println!(
            "I am from source bar. I have {} as an arg. I am module aware.",
            a
        );
    }
}

#[patchable]
fn foo() {
    println!("I am from source foo.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(); // prints "I am from source"
    foo.hotpatch_lib("target/debug/libhw_obj.so")?;
    foo(); // prints something totally different

    use crate::a::bar;
    bar(1);
    bar.hotpatch_lib("target/debug/libhw_obj.so")?;
    bar(2);
    Ok(())
}
