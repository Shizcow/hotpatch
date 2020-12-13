use hotpatch::patchable;

use std::{thread, time};
    
#[patchable]
fn foo() -> &'static str {
    // The patched version does not sleep. This shows that
    // no two threads can execute different definitions at
    // the same time. Threads themselves may be out of order
    // but after patching the first definition will never be
    // called.
    std::thread::sleep(time::Duration::from_micros(100));
    "Foo: Default"
}

#[patchable]
fn bar() -> &'static str {
    // Same here, but slowed down for emphasis
    std::thread::sleep(time::Duration::from_secs(1));
    "Bar: Default"
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // make some threads
    let mut children = vec![];
    for i in 0..10000 { // on my machine this does 9900+-50 before patch goes through
        children.push(thread::spawn(move || {
	    // call foo before/during/after hotpatch
	    std::thread::sleep(time::Duration::from_micros(i));
	    println!("Hello from thread {}. {}", i, foo());
        }));
    }

    // hotpatch in the middle of execution
    std::thread::sleep(time::Duration::from_micros(5));
    foo.hotpatch_lib("target/debug/libthreads_obj.so")?;

    // wait for threads to finish
    for child in children {
        let _ = child.join();
    }


    // Here's a slower example
    children = vec![];
    children.push(thread::spawn(move || {
	println!("{}", bar()); // will not be patched
    }));
    children.push(thread::spawn(move || {
	std::thread::sleep(time::Duration::from_secs(2));
	println!("{}", bar()); // expected to be patched
    }));

    std::thread::sleep(time::Duration::from_millis(500));
    // This should patch after the first call but before the second
    bar.hotpatch_lib("target/debug/libthreads_obj.so")?;

    // wait for threads to finish
    for child in children.into_iter().rev() {
        let _ = child.join();
    }
    
    Ok(())
}
