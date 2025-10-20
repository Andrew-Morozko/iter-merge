//! Implementation of [`MergeIter`]

use core::iter::FusedIterator;

use crate::{
    comparators::Comparator,
    internal::{Heap, Item},
    storage::Storage,
};

mod builder;
mod into_iters;
pub use builder::{Builder, DefaultBuilder, DefaultMergeIter};
pub use into_iters::{ItersIter, UnorderedItersIter};

/// Iterator over merged iterators
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct MergeIter<S, CMP>(pub(crate) Heap<S, CMP>);

impl<CMP, S> MergeIter<S, CMP>
where
    CMP: Comparator<Item<S>>,
    S: Storage,
{
    #[cfg(feature = "alloc")]
    /// Efficiently merges items into a [`Vec`](alloc::vec::Vec)
    ///
    /// This is faster than [`collect::<Vec<_>>`](Self::collect) by
    /// optimizing merges with 2 or 1 iterators remaining
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use iter_merge::merge;
    /// let v = merge([vec![1, 3, 5], vec![2, 4, 6]]).into_vec();
    /// assert_eq!(v, vec![1, 2, 3, 4, 5, 6]);
    /// # }
    /// ```
    pub fn into_vec(self) -> alloc::vec::Vec<Item<S>> {
        self.0.into_vec()
    }

    /// Returns a reference to the next item that will be returned by `next()` without
    /// consuming it.
    ///
    /// This method behaves identically to [`Peekable::peek`] from the standard library:
    /// it returns a reference to the next item, or `None` if the iterator is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use iter_merge::merge;
    ///
    /// let mut merged = merge([vec![1, 3, 5], vec![2, 4, 6]]);
    ///
    /// assert_eq!(merged.peek(), Some(&1));
    /// assert_eq!(merged.next(), Some(1));
    /// assert_eq!(merged.peek(), Some(&2));
    /// # }
    /// ```
    ///
    /// [`Peekable::peek`]: core::iter::Peekable::peek
    #[inline]
    pub fn peek(&self) -> Option<&Item<S>> {
        self.0.storage.peek()
    }

    /// Returns the next item of the iterator if it satisfies a predicate.
    ///
    /// This method behaves identically to [`Peekable::next_if`] from the standard library:
    /// it returns the next item if it satisfies the predicate, otherwise returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use iter_merge::merge;
    ///
    /// let mut merged = merge([vec![1, 1, 2, 3], vec![1, 4, 5, 6]]);
    ///
    /// // Consume all 1s
    /// while let Some(item) = merged.next_if(|&x| x == 1) {
    ///     assert_eq!(item, 1);
    /// }
    ///
    /// assert_eq!(merged.next(), Some(2));
    /// # }
    /// ```
    ///
    /// [`Peekable::next_if`]: core::iter::Peekable::next_if
    pub fn next_if(&mut self, func: impl FnOnce(&Item<S>) -> bool) -> Option<Item<S>> {
        match self.peek() {
            Some(item) if func(item) => {
                // SAFETY: self.peek would've returned None if len == 0.
                // Since the len > 0 - we always would have an item to produce
                Some(unsafe { self.0.pop_front_item().unwrap_unchecked() })
            }
            _ => None,
        }
    }

    /// Returns the next item of the iterator if it is equal to a given value.
    ///
    /// This method behaves identically to [`Peekable::next_if_eq`] from the standard library:
    /// it returns the next item if it is equal to the given value, otherwise returns `None`.
    ///
    /// This is a convenience method that is equivalent to `next_if(|item| item == expected)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use iter_merge::merge;
    ///
    /// let mut merged = merge([vec![1, 1, 2, 3], vec![1, 4, 5, 6]]);
    ///
    /// // Consume all 1s
    /// while let Some(item) = merged.next_if_eq(&1) {
    ///     assert_eq!(item, 1);
    /// }
    ///
    /// assert_eq!(merged.next(), Some(2));
    /// # }
    /// ```
    ///
    /// [`Peekable::next_if_eq`]: core::iter::Peekable::next_if_eq
    pub fn next_if_eq<T>(&mut self, expected: &T) -> Option<Item<S>>
    where
        T: ?Sized,
        Item<S>: PartialEq<T>,
    {
        self.next_if(|item| item == expected)
    }

    /// Returns an iterator, yielding unordered tuples of `(peeked_item, iter)`
    /// from the [`MergeIter`]
    ///
    /// No exact order is guaranteed, but you can expect the later iterators from [`MergeIter`]
    /// to be yielded first, and the frontmost iterator (that would've been polled by
    /// [`MergeIter::next()`]) to be yielded last.
    ///
    /// Original [`MergeIter`] remains valid after use of this iterator, items yielded by this
    /// iterator are excluded.
    #[inline]
    pub fn as_unordered_iters(&mut self) -> UnorderedItersIter<'_, S> {
        UnorderedItersIter(&mut self.0.storage)
    }

    /// Returns an ordered iterator, yielding tuples of `(peeked_item, iter)` from the [`MergeIter`]
    ///
    /// Items are ordered according to value of `peeked_item`, as compared by the [`MergeIter`]'s
    /// comparator
    ///
    /// Original [`MergeIter`] remains valid after use of this iterator, items yielded by this
    /// iterator are excluded.
    #[inline]
    pub fn as_iters(&mut self) -> ItersIter<'_, S, CMP> {
        ItersIter(&mut self.0)
    }
}

impl<CMP, S> Iterator for MergeIter<S, CMP>
where
    S: Storage,
    CMP: Comparator<Item<S>>,
{
    type Item = Item<S>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front_item()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // this accounts for peeked items
        let mut min = self.0.storage.len();
        let mut max = min;
        let mut no_max = false;
        self.0.storage.map_items(|it| {
            let (it_min, it_max) = it.iter.size_hint();
            min = min.saturating_add(it_min);
            let overflow;
            // if we're here - storage.len()>0, and so is the initial max value
            // If it_max is None it will become usize::MAX, and adding non-zero value to
            // usize::MAX will overflow, correctly setting the no_max
            (max, overflow) = max.overflowing_add(it_max.unwrap_or(usize::MAX));
            no_max |= overflow;
        });
        // If any inner iterator has an unbounded upper bound, or the sum of
        // upper bounds overflows a usize - overall upper bound is None.
        (min, (!no_max).then_some(max))
    }

    fn count(mut self) -> usize
    where
        Self: Sized,
    {
        let mut count = 0;
        while let Some((_, iter)) = self.0.storage.pop_last_item() {
            // panic in debug and wrapping in release is the expected behaiour
            #[allow(clippy::arithmetic_side_effects)]
            {
                count += 1 + iter.count();
            }
        }
        count
    }
}

// The iterator is definitely fused, since we're popping inner iterators after
// the first `None` is returned
impl<CMP, S> FusedIterator for MergeIter<S, CMP>
where
    CMP: Comparator<Item<S>>,
    S: Storage,
{
}

#[cfg(test)]
mod tests {
    use core::{iter::repeat, pin::pin};

    use crate::ArrayStorage;

    #[test]
    fn peek() {
        let s = ArrayStorage::from_arr([[3, 2], [2, 6], [3, 4]]);
        let s = pin!(s);
        let mut m = s.build();
        assert_eq!(m.peek(), Some(&2));
        assert_eq!(m.next(), Some(2));
        assert_eq!(m.peek(), Some(&3));
    }

    #[test]
    fn next_if() {
        let s = ArrayStorage::from_arr([[3, 6], [1, 4], [2, 5]]);
        let s = pin!(s);
        let mut m = s.build();
        assert_eq!(m.next_if(|&el| el <= 2), Some(1));
        assert_eq!(m.next_if(|&el| el <= 2), Some(2));
        m.nth(10);
        assert_eq!(m.next_if(|_el| true), None);
    }

    #[test]
    fn next_if_eq() {
        let s = ArrayStorage::from_arr([[3, 6], [1, 4], [2, 5]]);
        let s = pin!(s);
        let mut m = s.build();
        assert_eq!(m.next_if_eq(&1), Some(1));
        assert_eq!(m.next_if_eq(&200), None);
        m.nth(3);
        assert_eq!(m.next_if_eq(&6), Some(6));
        assert_eq!(m.next_if_eq(&7), None);
    }

    #[test]
    fn count() {
        let s = ArrayStorage::from_arr([[3, 6], [1, 4], [2, 5]]);
        let s = pin!(s);
        let m = s.build();
        assert_eq!(m.count(), 6);
    }

    #[inline]
    fn into_dyn<T>(iter: &mut dyn Iterator<Item = T>) -> &mut dyn Iterator<Item = T>{
        iter
    }

    #[test]
    fn size_hint() {
        let s = ArrayStorage::from_arr([[3, 6], [1, 4], [2, 5]]);
        let s = pin!(s);
        let m = s.build();
        assert_eq!(m.size_hint(), (6, Some(6)));
        let s = ArrayStorage::from_arr([repeat(2), repeat(1)]);
        let s = pin!(s);
        let m = s.build();
        assert_eq!(m.size_hint(), (usize::MAX, None));

        let mut it_a = [0, 1, 2].into_iter();
        let mut it_b = repeat(3).take(usize::MAX).filter(|&el| el == 3);

        let s =
            ArrayStorage::from_arr([into_dyn(&mut it_a), &mut it_b]);
        let s = pin!(s);
        let m = s.build();
        assert_eq!(m.size_hint(), (4, None));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn debug_formatters() {
        let m = crate::merge([[31415]]);
        assert!(alloc::format!("{m:?}").contains("31415"));
        let mut s = ArrayStorage::with_capacity::<5>();
        s.push([31415]);
        let s = pin!(s);
        let m = s.build();
        assert!(alloc::format!("{m:?}").contains("31415"));
    }
}
