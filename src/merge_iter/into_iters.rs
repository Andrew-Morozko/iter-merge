//! Iterators over the iterators within the [`MergeIter`](crate::MergeIter)
use core::iter::FusedIterator;

use super::Heap;
use crate::{
    comparators::Comparator,
    internal::{Item, Iter, PeekIter},
    storage::Storage,
};

/// Iterator, yielding unordered tuples of `(peeked_item, iter)` from existing
/// [`MergeIter`](crate::MergeIter)
#[derive(Debug)]
pub struct UnorderedItersIter<'a, S>(pub(crate) &'a mut S);

impl<S: Storage> Iterator for UnorderedItersIter<'_, S> {
    type Item = (Item<S>, Iter<S>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_last_item()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.len()
    }
}

impl<S: Storage> FusedIterator for UnorderedItersIter<'_, S> {}

/// Iterator, yielding ordered tuples of `(peeked_item, iter)` from existing
/// [`MergeIter`](crate::MergeIter)
#[derive(Debug)]
pub struct ItersIter<'a, S, CMP>(pub(crate) &'a mut Heap<S, CMP>);

impl<S, CMP> Iterator for ItersIter<'_, S, CMP>
where
    S: Storage,
    CMP: Comparator<Item<S>>,
{
    type Item = (Item<S>, Iter<S>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front_iter().map(|it| {
            let PeekIter { item, iter } = it;
            (item, iter)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.storage.len(), Some(self.0.storage.len()))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.storage.len()
    }
}

impl<S, CMP> FusedIterator for ItersIter<'_, S, CMP>
where
    S: Storage,
    CMP: Comparator<Item<S>>,
{
}

#[cfg(test)]
mod tests {
    use core::{array, pin::pin};

    use crate::ArrayStorage;

    #[test]
    fn unordered() {
        let s = ArrayStorage::from_arr([[5, 2], [2, 6], [3, 4], [0, 2]]);
        let s = pin!(s);
        let mut m = s.build();
        let mut iters_iter = m.as_unordered_iters();
        let mut popped: [_; 4] = array::from_fn(|_idx| iters_iter.next().unwrap().0);
        popped.sort();
        assert_eq!(popped, [0, 2, 3, 5]);
        assert!(iters_iter.next().is_none());
        assert!(m.next().is_none());
    }

    #[test]
    fn ordered() {
        let s = ArrayStorage::from_arr([[5, 2], [2, 6], [3, 4], [0, 2]]);
        let s = pin!(s);
        let mut m = s.build();
        let mut iters_iter = m.as_iters();
        let (item, iter) = iters_iter.next().unwrap();
        assert_eq!(item, 0);
        assert!(iter.eq([2]));
        let (item, iter) = iters_iter.next().unwrap();
        assert_eq!(item, 2);
        assert!(iter.eq([6]));
        assert_eq!(m.peek(), Some(&3));
        let mut iters_iter = m.as_iters();
        let (item, iter) = iters_iter.next().unwrap();
        assert_eq!(item, 3);
        assert!(iter.eq([4]));
        let (item, iter) = iters_iter.next().unwrap();
        assert_eq!(item, 5);
        assert!(iter.eq([2]));
        assert!(iters_iter.next().is_none());
    }
}
