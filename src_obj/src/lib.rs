use phf::phf_map;

#[no_mangle]
pub static HOTPATCH_EXPORTS: phf::Map<&'static str, fn((i32,)) -> i32> = phf_map! {
    "::foo" => foo,
};

pub fn foo(args: (i32,)) -> i32 {
    println!("Hello from foo {}", args.0);
    1
}
