#![feature(main)]

use hotpatch::*;

#[patchable]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("You'll only see me once.");
    unsafe { // because we're patching the current function, this will
	// let multiple function definitions exist at the same time.
	// That's unsafe, so the unsafe block is required.
	main.force_hotpatch_lib("target/debug/libunsafe_obj.so")?;
    }
    main()?; // and now libunsafe_obj defines what happens
    unsafe { main.force_hotpatch_fn(foo)?; }
    main()?;
    main()?; // Something different happens each time!
    main()?;
    main()?;
    main() // And the base case remains the same
}

fn foo() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello from foo");
    unsafe { main.force_hotpatch_fn(bar) }
}

fn bar() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello from bar");
    unsafe { main.force_hotpatch_fn(baz) }
}

fn baz() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello from baz");
    unsafe { main.force_hotpatch_fn(|| {
	println!("Hello from a closure");
	Ok(())
    }) }
}
