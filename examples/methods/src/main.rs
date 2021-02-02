use hotpatch::*;

struct Foo {}
// begin macro generation output
#[allow(non_camel_case_types)]
trait __HotpatchTraitGen_0 {
    fn bar() -> ();
}
impl __HotpatchTraitGen_0 for Foo {
    fn bar() {
        println!("this is passthrough!");
    }
}
impl Foo {
    #[cfg(not(doc))]
    #[allow(non_upper_case_globals)]
    pub const bar: hotpatch::MutConst<Patchable<dyn Fn() -> () + Send + Sync + 'static>> =
        hotpatch::MutConst::new(|| {
            static __hotpatch_implgen_0: hotpatch::Patchable<
                dyn Fn() -> () + Send + Sync + 'static,
            > = hotpatch::Patchable::__new(|| {
                hotpatch::Patchable::__new_internal(
                    Box::new(<Foo as __HotpatchTraitGen_0>::bar)
                        as Box<dyn Fn() -> () + Send + Sync + 'static>,
                    "hw_bin::foo",
                    "fn() -> ()",
                )
            });
            &__hotpatch_implgen_0
        });
    #[cfg(doc)]
    /// Warnings here
    pub fn bar() {
        println!("this is passthrough!");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Foo::bar();
    Foo::bar.hotpatch_fn(|| println!("this is patch!"))?;
    Foo::bar();
    Ok(())
}
