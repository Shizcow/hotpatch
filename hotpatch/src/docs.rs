//! A significant amount of source code uses variadic generics, which clutter
//! documentation. This module holds documentation for those, in a format
//! which is much easier to read.

#[cfg(doc)]
impl<RealType: ?Sized + Send + Sync + 'static> Patchable<RealType> {
    /// Hotpatch this functor with functionality defined in `ptr`.
    /// `ptr` can be a function pointer or `move` closure with the
    /// same type signature as the functor's function.
    ///
    /// ## Example
    /// ```
    /// #[patchable]
    /// fn foo(_: i32, _: i32, _: i32) {}
    ///
    /// fn bar(_: i32, _: i32, _: i32) {}
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///   foo.hotpatch_fn(bar)?;
    ///   foo.hotpatch_fn(move |a, b, c| println!("{} {} {}", a, b, c))?;
    ///   Ok(())
    /// }
    /// ```
    ///
    /// ## VaArgs Note
    /// Implementation is defined with the [`variadic_generics`](https://docs.rs/variadic_generics)
    /// crate. This means
    /// a macro is used to define a finite but large number of templated inputs.
    /// If using functions with large numbers of inputs and `hotpatch_fn` does not
    /// appear to be defined, compile `hotpatch` with the `large-signatures` feature
    /// to increase the number of supported arguements.
    pub fn hotpatch_fn<F>(&self, c: F) -> Result<(), Box<dyn std::error::Error + '_>>
    where
        F: Fn(VaGen) -> Ret,
    {
        // The actual implementation is in toplevel
    }
    /// Like [`hotpatch_fn`](Patchable::hotpatch_fn) but uses
    /// [`RwLock::try_write`](https://doc.rust-lang.org/std/sync/struct.RwLock.html#method.try_write).
    pub fn try_hotpatch_fn<F>(&self, c: F) -> Result<(), Box<dyn std::error::Error + '_>>
    where
        F: Fn(VaGen) -> Ret,
    {
        // The actual implementation is in toplevel
    }
    /// Like [`hotpatch_fn`](Patchable::hotpatch_fn) but uses
    /// unsafe features to completly bypass the
    /// [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
    /// Can be used to patch the current function or parent functions.
    /// **Use with caution**.
    pub unsafe fn force_hotpatch_fn<F>(&self, c: F) -> Result<(), Box<dyn std::error::Error + '_>>
    where
        F: Fn(VaGen) -> Ret,
    {
        // The actual implementation is in toplevel
    }
}
