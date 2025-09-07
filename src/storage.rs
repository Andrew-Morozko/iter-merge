#[cfg(not(any(feature = "vec_storage", feature = "stackvec_storage")))]
compile_error!("At least one storage feature must be enabled. Enable either 'vec_storage' (default) or 'stackvec_storage' feature.");

/// Storage trait for this library. Methods may use unsafe, safety is guaranteed by the caller
/// Trait is not public, so we are the only caller
/// SAFETY: The caller must ensure that
/// * The indices in all methods are valid (< len)
/// * `push` adds a single element at the end of the collection
/// * `remove` removes a single element at the given index, moving the rest
///   of the collection one position to the left
/// * `swap_remove` removes a single element at the given index, replacing it with the last element
pub(crate) trait Storage {
    type Item;
    fn new() -> Self;
    fn push(&mut self, value: Self::Item);
    fn remove(&mut self, index: usize) -> Self::Item;
    fn swap_remove(&mut self, index: usize) -> Self::Item;
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> &Self::Item;
    fn get_mut(&mut self, index: usize) -> &mut Self::Item;
    fn reserve_for<I: Iterator>(&mut self, iter: &I);
}

/// Chooses to use safe or unsafe implementation depending on enabled cfg options
macro_rules! select {
    ($safe:expr ; unsafe { $unsafe:expr }) => {
        #[cfg(any(debug_assertions, feature = "forbid_unsafe", test))]
        { $safe }
        #[cfg(not(any(debug_assertions, feature = "forbid_unsafe", test)))]
        unsafe { $unsafe }
    };
}
// pub(crate) use select;

#[cfg(feature = "vec_storage")]
mod vec;

#[cfg(feature = "stackvec_storage")]
mod stackvec;
