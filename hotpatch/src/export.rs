/// Created by [`#[patch]`](crate::patch). Internal use only.
///
/// Creates a `#[no_mangle] pub static` instance to be imported in another
/// binary by [`Patchable`](crate::Patchable) methods.
pub struct HotpatchExport<T: 'static> {
    pub symbol: &'static str,
    pub sig: &'static str,
    pub ptr: T,
}

#[doc(hidden)]
impl<T: 'static> HotpatchExport<T> {
    pub const fn __new(ptr: T, symbol: &'static str, sig: &'static str) -> Self {
        Self { symbol, sig, ptr }
    }
}
