#[cfg(not(any(feature = "vec_storage", feature = "stackvec_storage")))]
compile_error!(
    "At least one storage feature must be enabled ('vec_storage' or 'stackvec_storage')"
);

mod private {
    pub struct SealedToken;
}

pub(crate) use private::SealedToken;

/// Sealed storage trait for this crate. Methods may use unsafe, relying on safety
/// guaranteed by the caller, which is only this crate.
///
/// Why method sealing? Because otherwise users could use the methods, which may be
/// unsafe, but not marked as such.
/// Why not mark them as unsafe? Because if the library is compiled with `forbid_unsafe` feature,
/// then the methods *are* safe internally, but we can't call them because of the restriction.
///
/// This trait is public only to get around exposing restrictions in earlier rust versions.
///
/// # SAFETY
/// The caller must ensure that:
/// * The indices in all methods are valid (< len)
///
/// The implementer must guarantee that:
/// * `push` adds a single element at the end of the collection
/// * `remove` removes a single element at the given index, moving the rest
///   of the collection one position to the left
/// * `swap_remove` removes a single element at the given index, replacing it with the last element
/// * `len` returns the number of elements in the collection
/// * `get` returns a reference to the element at the given index
/// * `get_mut` returns a mutable reference to the element at the given index
#[doc(hidden)]
pub trait Storage {
    type Item;
    fn new(_: SealedToken) -> Self;
    fn push(&mut self, value: Self::Item, _: SealedToken);
    fn remove(&mut self, index: usize, _: SealedToken) -> Self::Item;
    fn swap_remove(&mut self, index: usize, _: SealedToken) -> Self::Item;
    fn len(&self, _: SealedToken) -> usize;
    fn get(&self, index: usize, _: SealedToken) -> &Self::Item;
    fn get_mut(&mut self, index: usize, _: SealedToken) -> &mut Self::Item;
    /// Rationale for this signature:
    /// We avoid calling `size_hint` for Storage implementations
    /// that have a fixed capacity.
    fn reserve_for<I: Iterator>(&mut self, iter: &I, _: SealedToken);
}

/// Chooses to use safe or unsafe implementation depending on enabled cfg options
macro_rules! select {
    ($safe:expr; unsafe { $unsafe:expr }) => {
        #[cfg(any(debug_assertions, feature = "forbid_unsafe", test))]
        {
            $safe
        }
        #[cfg(not(any(debug_assertions, feature = "forbid_unsafe", test)))]
        unsafe {
            $unsafe
        }
    };
}

#[cfg(feature = "vec_storage")]
mod vec;

#[cfg(feature = "stackvec_storage")]
mod stackvec;
