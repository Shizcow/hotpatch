use hotpatch::*;

/// I'm a functor
#[patchable]
fn foo(depth: i32) -> Result<(), Box<dyn std::error::Error>> {
    println!("I am Foo. Depth: {}", depth);
    unsafe {
	foo.force_hotpatch_fn(move |depth: i32| {
	    println!("This is very unsafe {}", depth);
	    foo.force_hotpatch_fn(move |depth: i32| {
		println!("It gets even worse! {}", depth);
		Ok(())
	    })?;
	    foo(depth+1)
	})?;
    }
    foo(depth+1)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    foo(1)
}
