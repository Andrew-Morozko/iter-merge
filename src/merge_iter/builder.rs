use core::cmp::Ordering;

use super::Heap;
use crate::{
    MergeIter,
    comparators::{ByFunc, ByKey, ByOrd, Chain, Comparator, MaxFirst, tie_breaker},
    internal::Item,
    storage::Storage,
};

/// [`MergeIter`] with default comparator
pub type DefaultMergeIter<S> = MergeIter<S, Chain<ByOrd, tie_breaker::InsertionOrder>>;

/// [`MergeIter`] with default comparator
pub type DefaultBuilder<S> = Builder<S, ByOrd, tie_breaker::InsertionOrder>;

/// Builder for [`MergeIter`](crate::MergeIter)
///
/// Allows to configure how to compare the items in the iterators we are merging.
///
/// By default items are compared using [`Ord`], smallest item yielded first, and if the
/// items are equal - they are yielded in insertion order, earliest -- first.
#[derive(Debug)]
pub struct Builder<S, CMP, TieBreaker> {
    comparator: CMP,
    tie_breaker: TieBreaker,
    storage: S,
}

impl<S, CMP, TieBreaker> Builder<S, CMP, TieBreaker>
where
    S: Storage,
{
    #[inline]
    pub(crate) const fn new(storage: S, comparator: CMP, tie_breaker: TieBreaker) -> Self {
        Self {
            comparator,
            tie_breaker,
            storage,
        }
    }

    /// Compare heap items using comparator `cmp` and yield smallest item first
    #[inline]
    pub fn min_by<C: Comparator<Item<S>>>(self, cmp: C) -> Builder<S, C, TieBreaker> {
        Builder::new(self.storage, cmp, self.tie_breaker)
    }

    /// Compare heap items using comparator `cmp` and yield largest item first
    #[inline]
    pub fn max_by<C: Comparator<Item<S>>>(
        self, cmp: C,
    ) -> Builder<S, MaxFirst<C>, TieBreaker> {
        self.min_by(MaxFirst(cmp))
    }

    /// Compare heap items using `func` and yield smallest item first
    #[inline]
    pub fn min_by_func<F>(self, func: F) -> Builder<S, ByFunc<F>, TieBreaker>
    where
        F: Fn(&Item<S>, &Item<S>) -> Ordering,
    {
        self.min_by(ByFunc(func))
    }

    /// Compare heap items using `func` and yield largest item first
    #[inline]
    pub fn max_by_func<F>(self, func: F) -> Builder<S, MaxFirst<ByFunc<F>>, TieBreaker>
    where
        F: Fn(&Item<S>, &Item<S>) -> Ordering,
    {
        self.max_by(ByFunc(func))
    }

    /// Compare heap items by comparing their keys produced by `func` and yield smallest item first
    #[inline]
    pub fn min_by_key<F, K>(self, func: F) -> Builder<S, ByKey<F>, TieBreaker>
    where
        F: Fn(&Item<S>) -> K,
        K: Ord,
    {
        self.min_by(ByKey(func))
    }

    /// Compare heap items by comparing their keys produced by `func` and yield largest item first
    #[inline]
    pub fn max_by_key<F, K>(self, func: F) -> Builder<S, MaxFirst<ByKey<F>>, TieBreaker>
    where
        F: Fn(&Item<S>) -> K,
        K: Ord,
    {
        self.max_by(ByKey(func))
    }

    /// If items are equal - compare them again using `tie_breaker`, yielding smallest item first
    #[inline]
    pub fn tie_breaker<TB: Comparator<Item<S>>>(self, tie_breaker: TB) -> Builder<S, CMP, TB> {
        Builder::new(self.storage, self.comparator, tie_breaker)
    }
}

impl<S, CMP, TieBreaker> Builder<S, CMP, TieBreaker>
where
    S: Storage,
    CMP: Comparator<Item<S>>,
    TieBreaker: Comparator<Item<S>>,
{
    /// Builds the [`MergeIter`] using specified comparator and tie breaker.
    ///
    /// Getting a compiler error
    /// ```custom
    /// the method `build` exists for struct `Builder<...>`,
    /// but its trait bounds were not satisfied
    /// ```
    /// means that the item type does not implement [`Ord`].
    /// Either implement it for your type or specify another way to compare items by using builder
    /// methods documented above.
    #[inline]
    pub fn build(self) -> MergeIter<S, Chain<CMP, TieBreaker>> {
        MergeIter(Heap::new(
            Chain::new(self.comparator, self.tie_breaker),
            self.storage,
        ))
    }
}
