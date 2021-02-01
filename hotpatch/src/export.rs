/// Created by [`#[patch]`](patch). Internal use only.
///
/// Creates a `#[no_mangle] pub static` instance to be imported in another
/// binary by [`Patchable`](Patchable) methods.
pub struct HotpatchExport<T: 'static> {
    pub symbol: &'static str,
    pub sig: &'static str,
    pub ptr: &'static T,
}

impl<T: 'static> HotpatchExport<T> {
    #[doc(hidden)]
    pub const fn __new(ptr: &'static T, symbol: &'static str, sig: &'static str) -> Self {
        Self { symbol, sig, ptr }
    }
}
