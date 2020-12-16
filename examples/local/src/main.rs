#![feature(main)]

use hotpatch::*;

fn notmain() -> Result<(), Box<dyn std::error::Error>> {
    println!("I'm not main!");
    unsafe {
	main.force_hotpatch_fn(|| {
	    println!("Neither am I!");
	    Ok(())
	})?;
    }
    Ok(())
}

#[patchable]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello #1");
    unsafe {
	main.force_hotpatch_fn(notmain)?;
    }
    main()?;
    main()
}
