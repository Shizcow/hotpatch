use hotpatch::patchable;

use std::{thread, time};
    
#[patchable]
fn foo() -> &'static str {
    "Foo: Default"
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // make some threads
    let mut children = vec![];
    for i in 0..1000 { // on my machine this does 990+-7 before patch goes through
        children.push(thread::spawn(move || {
	    // call foo before/during/after hotpatch
	    std::thread::sleep(time::Duration::from_micros(i));
	    println!("Hello from thread {}. {}", i, foo());
        }));
    }

    // hotpatch in the middle of execution
    std::thread::sleep(time::Duration::from_micros(5));
    foo.hotpatch("target/debug/libthreads_obj.so")?;

    // wait for threads to finish
    for child in children {
        let _ = child.join();
    }
    Ok(())
}
