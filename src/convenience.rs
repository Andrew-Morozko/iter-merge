#![allow(clippy::type_complexity)]
use core::cmp::Ordering;

use crate::{
    MergeIter, VecStorage,
    comparators::{ByFunc, ByKey, Chain, tie_breaker},
    merge_iter::DefaultMergeIter,
    storage::InternalVecStorage,
};

/// Constructs a new [`MergeIter`] with default parameters:
/// * Uses [`VecStorage`]
/// * Yields items according to their [`Ord`] implementation, smallest-first
/// * Equal items are yielded in order of their respective iterators
pub fn merge<IT>(
    iters: IT,
) -> DefaultMergeIter<InternalVecStorage<<IT::Item as IntoIterator>::IntoIter>>
where
    IT: IntoIterator,
    IT::Item: IntoIterator,
    <IT::Item as IntoIterator>::Item: Ord,
{
    VecStorage::from_iter(iters).build()
}

/// Constructs a new [`MergeIter`] with default parameters:
/// * Uses [`VecStorage`]
/// * Yields smallest items according to `func`
/// * Equal items are yielded in order of their respective iterators
pub fn merge_by<IT, F>(
    iters: IT, func: F,
) -> MergeIter<
    InternalVecStorage<<IT::Item as IntoIterator>::IntoIter>,
    Chain<ByFunc<F>, tie_breaker::InsertionOrder>,
>
where
    IT: IntoIterator,
    IT::Item: IntoIterator,
    <IT::Item as IntoIterator>::Item: Ord,
    F: Fn(&<IT::Item as IntoIterator>::Item, &<IT::Item as IntoIterator>::Item) -> Ordering,
{
    VecStorage::from_iter(iters)
        .into_builder()
        .min_by_func(func)
        .build()
}

/// Constructs a new [`MergeIter`] with default parameters:
/// * Uses [`VecStorage`]
/// * Yields smallest items with the smallest key according to `func`
/// * Equal items are yielded in order of their respective iterators
pub fn merge_by_key<IT, F, K>(
    iters: IT, func: F,
) -> MergeIter<
    InternalVecStorage<<IT::Item as IntoIterator>::IntoIter>,
    Chain<ByKey<F>, tie_breaker::InsertionOrder>,
>
where
    IT: IntoIterator,
    IT::Item: IntoIterator,
    <IT::Item as IntoIterator>::Item: Ord,
    F: Fn(&<IT::Item as IntoIterator>::Item) -> K,
    K: Ord,
{
    VecStorage::from_iter(iters)
        .into_builder()
        .min_by_key(func)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_works() {
        assert!(merge([[3, 6], [1, 4], [2, 5]]).eq([1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn merge_by_works() {
        assert!(merge_by([[3, 6], [1, 4], [2, 5]], |a, b| { b.cmp(a) }).eq([3, 6, 2, 5, 1, 4]));
    }

    #[test]
    fn merge_by_key_works() {
        assert!(
            merge_by_key([[-3_i32, 6], [-1, 4], [2, -5]], |val| val.abs())
                .eq([-1, 2, -3, 4, -5, 6])
        );
    }
}
