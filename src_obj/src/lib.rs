use patchable::HotpatchExport;

pub fn foo(args: ()) -> i32 {
    println!("Hello from foo");
    1
}

#[no_mangle]
pub static __HOTPATCH_EXPORT_0: HotpatchExport<fn(()) -> i32> =
    HotpatchExport{ptr: foo,
		   symbol: "::foo",
		   sig: "fn(()) -> i32"};

