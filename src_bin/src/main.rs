use std::collections::HashMap;
use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref HOTPATCH_IMPORTS: RwLock<HashMap<String, fn() -> i32>> = {
	let mut m: HashMap<String, fn() -> i32> = HashMap::new();
	m.insert("::foo".to_owned(), foo);
	RwLock::new(m)
    };
    static ref LIB: libloading::Library =
	libloading::Library::new("../src_obj/target/debug/libsrc_obj.so").unwrap();
}

fn foo() -> i32 {
    println!("I am from source");
    0
}


fn dispatch(fun_name: &str) -> Result<i32, Box<dyn std::error::Error>> {
    match HOTPATCH_IMPORTS.read()?.get(fun_name) {
	None => simple_error::bail!(format!("No importable function named {} found", fun_name)),
	Some(ptr) => return Ok(ptr()),
    }
}


fn cc(fun_name: &str) -> Result<fn() -> i32, Box<dyn std::error::Error>> {
    unsafe {
        let exports: libloading::Symbol<*mut phf::Map<&'static str, fn() -> i32>>
	    = LIB.get(b"HOTPATCH_EXPORTS")?;
	Ok(*(**exports).get(fun_name).unwrap())
    }
}

fn hotpatch(fun_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let new_ptr = cc(fun_name)?;
    HOTPATCH_IMPORTS.write()?.insert(fun_name.to_owned(), new_ptr);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dispatch("::foo")?;
    hotpatch("::foo")?;
    dispatch("::foo")?;
    Ok(())
}
