/// Created by [`#[patch]`](patch). Internal use only.
///
/// Creates a `#[no_mangle] pub static` instance to be imported in another
/// binary by [`Patchable`](Patchable) methods.
pub struct HotpatchExport<T> {
    pub symbol: &'static str, // field order is important
    pub sig: &'static str,
    pub ptr: T, // in the form Fn(Args...) -> Ret
}

impl<T> HotpatchExport<T> {
    #[doc(hidden)]
    pub const fn __new(ptr: T, symbol: &'static str, sig: &'static str) -> Self {
        Self { symbol, sig, ptr }
    }
}
