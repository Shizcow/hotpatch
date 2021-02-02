use hotpatch::*;

/// This is a struct
struct Foo {}
/// And this is where free associated items can be defined
#[patchable]
impl Foo {
    /// Here's one of them!
    pub fn bar() {
        println!("this is passthrough!");
    }
}
// begin macro generation output
// impl Foo {
//     #[cfg(not(doc))]
//     #[allow(non_upper_case_globals)]
//     pub const bar: hotpatch::MutConst<Patchable<dyn Fn() -> () + Send + Sync + 'static>> =
//         hotpatch::MutConst::new(|| {
//             #[patchable]
//             fn __hotpatch_staticwrap() {
//                 println!("this is passthrough!");
//             }
//             &__hotpatch_staticwrap
//         });
//     #[cfg(doc)]
//     /// Warnings here
//     pub fn bar() {
//         println!("this is passthrough!");
//     }
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Foo::bar();
    Foo::bar.hotpatch_fn(|| println!("this is patch!"))?;
    Foo::bar();
    Ok(())
}