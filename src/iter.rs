use core::{cmp::Ordering, mem};

use crate::storage::Storage;

#[cfg(feature = "vec_storage")]
use alloc::vec::Vec;

/// An iterator that pulls the smallest item from multiple iterators.
///
/// `MergedIter` is created by [`Merged`]. It merges items from several base iterators
/// based on a comparison function.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "vec_storage")]
/// # {
/// use iter_merge::Merged;
///
/// let iter1 = vec![1, 3, 5];
/// let iter2 = vec![2, 4, 6];
///
/// let mut merged = Merged::new([iter1, iter2]).build();
/// let result = merged.into_vec();
///
/// assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
/// # }
/// ```
#[derive(Debug)]
pub struct MergedIter<const STABLE_TIE_BREAKING: bool, S, Cmp> {
    peek_iters: S,
    cmp: Cmp,
    min_idx: usize,
    next_min_idx: usize,
}

#[expect(private_bounds)]
impl<const STABLE_TIE_BREAKING: bool, S, Cmp, Item, Iter> MergedIter<STABLE_TIE_BREAKING, S, Cmp>
where
    Iter: Iterator<Item = Item>,
    Cmp: Fn(&Item, &Item) -> Ordering,
    S: Storage<Item = (Item, Iter)>,
{
    pub(crate) fn new(cmp: Cmp) -> Self {
        Self {
            peek_iters: S::new(),
            cmp,
            min_idx: 0,
            next_min_idx: 1,
        }
    }

    /// Compares two peeked items by indexes
    #[inline(always)]
    fn cmp_idx(&self, idx_a: usize, idx_b: usize) -> Ordering {
        (self.cmp)(&self.peek_iters.get(idx_a).0, &self.peek_iters.get(idx_b).0)
    }

    /// Removes the smallest item-iterator pair, returns the item and leaves
    /// `self.next_min_idx` in an invalid state
    #[inline(always)]
    fn pop_min(&mut self) -> Item {
        let res;
        if STABLE_TIE_BREAKING {
            res = self.peek_iters.remove(self.min_idx).0;
            self.min_idx = if self.next_min_idx > self.min_idx {
                self.next_min_idx - 1
            } else {
                self.next_min_idx
            };
        } else {
            res = self.peek_iters.swap_remove(self.min_idx).0;
            // if self.next_min_idx == self.peek_iters.len()
            // then swap_remove moved second_smallest element to the min_idx position
            // The length of `peek_iters` has already been reduced by `swap_remove` at this point.
            // So `self.peek_iters.len()` is the old length minus one. The condition checks if
            // `next_min_idx` was pointing to the last element of the slice before `swap_remove`.
            if self.next_min_idx != self.peek_iters.len() {
                self.min_idx = self.next_min_idx;
            }
        }
        res
    }

    /// Works on all lengths of the iterator, but meaningful only for len >= 2
    fn find_next_min(&mut self) {
        // need to find the new next_min_idx
        // creating next_min_idx to be distinct from self.min_idx
        let mut next_min_idx: usize = (self.min_idx == 0).into();
        // Two loops in order to skip self.min_idx
        for i in (next_min_idx + 1)..self.min_idx {
            if self.cmp_idx(i, next_min_idx).is_lt() {
                next_min_idx = i;
            }
        }
        for i in (self.min_idx + 1)..self.peek_iters.len() {
            if self.cmp_idx(i, next_min_idx).is_lt() {
                next_min_idx = i;
            }
        }
        self.next_min_idx = next_min_idx;
    }

    #[inline]
    fn update_after_peek(&mut self) {
        debug_assert!(self.peek_iters.len() >= 2);
        match self.cmp_idx(self.min_idx, self.next_min_idx) {
            Ordering::Equal if STABLE_TIE_BREAKING && self.next_min_idx < self.min_idx => {
                mem::swap(&mut self.next_min_idx, &mut self.min_idx);
                // second smallest can't be before min_idx (since min_idx
                // is the first of the smallest elements, and can't be after current
                // value of self.next_min_idx, since it was selected to be leftmost
                // in the previous run.
                // in case [[0] [0] [-1, 0]]
                // smallest = 2; second smallest = 0
                // after iteration: [[0] [0] [0]]
                // smallest = 0; second smallest = 2 (1 is the correct value)
                // loop below rechecks minimal portion of iterators to have correct
                // second smallest with stable tie break
                for i in (self.min_idx + 1)..self.next_min_idx {
                    if self.cmp_idx(i, self.min_idx).is_eq() {
                        self.next_min_idx = i;
                        break;
                    }
                }
            }
            Ordering::Greater => {
                self.min_idx = self.next_min_idx;
                self.find_next_min();
            }
            _ => {}
        }
    }

    /// REQUIRES len >= 1
    #[inline]
    fn produce_next(&mut self) -> Item {
        debug_assert!(self.peek_iters.len() >= 1);
        let res;
        let smallest = self.peek_iters.get_mut(self.min_idx);
        if let Some(new_peeked) = smallest.1.next() {
            res = mem::replace(&mut smallest.0, new_peeked);
            if self.peek_iters.len() > 1 {
                self.update_after_peek();
            }
        } else {
            res = self.pop_min();
            self.find_next_min();
        }
        res
    }

    /// Expects len > 2, min_idx and next_min_idx to be correct for up to new_item_idx.
    /// Results in min_idx and next_min_idx being correct for up to and including new_item_idx.
    #[inline]
    fn upd_idx_n(&mut self, new_item_idx: usize) {
        debug_assert!(self.peek_iters.len() > 2);
        if self.cmp_idx(new_item_idx, self.next_min_idx).is_ge(){
            return;
        }

        // new < next_min
        if self.cmp_idx(new_item_idx, self.min_idx).is_lt() {
            // new < min <= next_min
            self.next_min_idx = self.min_idx;
            self.min_idx = new_item_idx;
        } else {
            // min <= new <= next_min
            self.next_min_idx = new_item_idx;
        }
    }

    /// Expects len >= 2.
    /// Results in min_idx and next_min_idx being correct for the first two items.
    #[inline]
    fn upd_idx_2(&mut self) {
        debug_assert!(self.peek_iters.len() >= 2);
        if self.cmp_idx(0, 1).is_le() {
            self.min_idx = 0;
            self.next_min_idx = 1;
        } else {
            self.min_idx = 1;
            self.next_min_idx = 0;
        }
    }

    /// Adds a single iterator to the merge.
    ///
    /// This method allows you to dynamically add iterators to an existing `MergedIter`.
    /// The new iterator will be integrated into the merge according to the comparison
    /// function, maintaining the sorted order of the output.
    ///
    /// Empty iterators are ignored and do not affect the merge state.
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator (or anything that implements `IntoIterator`) to add to the merge
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    ///
    /// let mut merged = Merged::new([vec![2, 5, 8]]).build();
    ///
    /// assert_eq!(merged.next(), Some(2));
    ///
    /// // Add another iterator dynamically
    /// merged.add_iter(vec![1, 4, 7]);
    /// assert_eq!(merged.next(), Some(1));
    ///
    /// merged.add_iter(vec![3, 6, 9]);
    ///
    /// let result = merged.into_vec();
    /// assert_eq!(result, vec![3, 4, 5, 6, 7, 8, 9]);
    /// # }
    /// ```
    pub fn add_iter(&mut self, iter: impl IntoIterator<IntoIter=Iter>) {
        let mut iter = iter.into_iter();
        let Some(peeked) = iter.next() else {
            return;
        };
        let new_item_idx = self.peek_iters.len();
        self.peek_iters.push((peeked, iter));
        match new_item_idx {
            0 => {},
            1 => self.upd_idx_2(),
            n => self.upd_idx_n(n),
        }
    }

    /// Adds multiple iterators to the merge at once.
    ///
    /// This method efficiently adds multiple iterators to an existing `MergedIter` in a single
    /// operation. It's more efficient than calling [`add_iter`] multiple times because it can
    /// optimize memory allocation and minimize index recalculations.
    ///
    /// Empty iterators in the collection are automatically filtered out and ignored.
    ///
    /// # Arguments
    ///
    /// * `iters` - A collection of iterators to add to the merge. Each element should implement
    ///   `IntoIterator` with the same item type as the existing merge.
    ///
    /// # Performance
    ///
    /// This method attempts to reserve storage space based on the size hint of the input
    /// collection, which can improve performance by reducing memory reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    ///
    /// let mut merged = Merged::new([vec![1, 6, 11]]).build();
    ///
    /// // Add multiple iterators at once
    /// merged.add_iters([
    ///     vec![2, 7, 12],
    ///     vec![3, 8, 13],
    ///     vec![4, 9, 14],
    ///     vec![5, 10, 15],
    /// ]);
    ///
    /// let result = merged.into_vec();
    /// assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    /// # }
    /// ```
    ///
    /// [`add_iter`]: MergedIter::add_iter
    pub fn add_iters(&mut self, iters: impl IntoIterator<Item = impl IntoIterator<IntoIter=Iter>>) {
        let iters = iters.into_iter();
        self.peek_iters.reserve_for(&iters);
        let mut item_iter = iters.filter_map(|iter| {
            let mut iter = iter.into_iter();
            iter.next().map(|peeked| (peeked, iter))
        });
        let len = self.peek_iters.len();
        if len < 2 {
            if len == 0 {
                let Some(item) = item_iter.next() else {
                    return;
                };
                self.peek_iters.push(item);
            }
            // now len == 1
            let Some(item) = item_iter.next() else {
                return;
            };
            self.peek_iters.push(item);
            self.upd_idx_2();
        }
        // now len >= 2
        for item in item_iter {
            let new_item_idx = self.peek_iters.len();
            self.peek_iters.push(item);
            self.upd_idx_n(new_item_idx);
        }
    }

    /// Replaces the comparison function and returns a new `MergedIter`.
    ///
    /// This method consumes the current iterator and creates a new one with a different
    /// comparison function. The new iterator will use the provided comparison function
    /// to determine the ordering of items from the merged iterators.
    ///
    /// # Arguments
    ///
    /// * `cmp` - A function that compares two items and returns an `Ordering`
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    /// use std::cmp::Ordering;
    ///
    /// let mut merged = Merged::new([
    ///     vec![1, 4],
    ///     vec![2, 5],
    ///     vec![3, 6],
    /// ]).build();
    ///
    /// assert_eq!(merged.next(), Some(1));
    /// assert_eq!(merged.next(), Some(2));
    /// assert_eq!(merged.next(), Some(3));
    /// // Reverse the ordering
    /// let mut merged = merged.replace_cmp(|a, b| b.cmp(a));
    /// assert_eq!(merged.next(), Some(6));
    /// assert_eq!(merged.next(), Some(5));
    /// assert_eq!(merged.next(), Some(4));
    ///
    /// # }
    /// ```
    pub fn replace_cmp<F>(self, cmp: F) -> MergedIter<STABLE_TIE_BREAKING, S, F> where
        F: Fn(&Item, &Item) -> Ordering,
    {
        let mut new = MergedIter {
            peek_iters: self.peek_iters,
            cmp,
            min_idx: 0,
            next_min_idx: 1,
        };
        let len = new.peek_iters.len();
        if len < 2 {
            return new;
        }
        new.upd_idx_2();
        for new_item_idx in 2..len{
            new.upd_idx_n(new_item_idx);
        }
        new
    }

    /// Consumes the iterator and produces a `Vec` of its items.
    ///
    /// This method is an optimization over `collect::<Vec<_>>()`. It can be more
    /// efficient, specifically when only one iterator remains, as it can extend the
    /// result vector with the remaining items of that iterator directly.
    ///
    /// # Examples
    ///
    /// ```
    /// use iter_merge::Merged;
    ///
    /// let iter1 = vec![1, 3, 5];
    /// let iter2 = vec![2, 4, 6];
    ///
    /// let mut merged = Merged::new([iter1, iter2]).build();
    /// let result = merged.into_vec();
    ///
    /// assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    /// ```
    #[cfg(feature = "vec_storage")]
    pub fn into_vec(&mut self) -> Vec<Item> {
        let mut res = Vec::with_capacity(self.size_hint().0);
        match self.peek_iters.len() {
            0 => return res,
            1 => {
                // If we have 1 iterator left - noting to compare it against
                let (item, rest) = self.peek_iters.swap_remove(0);
                res.push(item);
                res.extend(rest);
                return res;
            }
            _ => {}
        }

        loop {
            let min = self.peek_iters.get_mut(self.min_idx);
            // following is self.produce_next(), but with last iterator optimization
            if let Some(new_peeked) = min.1.next() {
                res.push(mem::replace(&mut min.0, new_peeked));
                // there are at least 2 iterators remaining
                self.update_after_peek();
            } else {
                res.push(self.pop_min());
                if self.peek_iters.len() == 1 {
                    let (item, iter) = self.peek_iters.swap_remove(0);
                    res.push(item);
                    res.extend(iter);
                    return res;
                }
                // there are at least 2 iterators remaining
                self.find_next_min();
            }
        }
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
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    ///
    /// let iter1 = vec![1, 3, 5];
    /// let iter2 = vec![2, 4, 6];
    ///
    /// let mut merged = Merged::new([iter1, iter2]).build();
    ///
    /// assert_eq!(merged.peek(), Some(&1));
    /// assert_eq!(merged.next(), Some(1));
    /// assert_eq!(merged.peek(), Some(&2));
    /// # }
    /// ```
    ///
    /// [`Peekable::peek`]: std::iter::Peekable::peek
    pub fn peek<'a>(&'a mut self) -> Option<&'a Item>
    where
        Iter: 'a,
    {
        if self.min_idx >= self.peek_iters.len() {
            return None;
        }
        Some(&self.peek_iters.get(self.min_idx).0)
    }

    /// Returns the next item of the iterator if it satisfies a predicate.
    ///
    /// This method behaves identically to [`Peekable::next_if`] from the standard library:
    /// it returns the next item if it satisfies the predicate, otherwise returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    ///
    /// let iter1 = vec![1, 1, 2, 3];
    /// let iter2 = vec![1, 4];
    ///
    /// let mut merged = Merged::new([iter1, iter2]).build();
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
    /// [`Peekable::next_if`]: std::iter::Peekable::next_if
    pub fn next_if(&mut self, func: impl FnOnce(&Item) -> bool) -> Option<Item> {
        if self.peek_iters.len() == 0 || !func(&self.peek_iters.get(self.min_idx).0) {
            return None;
        }
        Some(self.produce_next())
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
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    ///
    /// let iter1 = vec![1, 1, 2, 3];
    /// let iter2 = vec![1, 4];
    ///
    /// let mut merged = Merged::new([iter1, iter2]).build();
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
    /// [`Peekable::next_if_eq`]: std::iter::Peekable::next_if_eq
    pub fn next_if_eq<T>(&mut self, expected: &T) -> Option<Item>
    where
        T: ?Sized,
        Item: PartialEq<T>,
    {
        self.next_if(|item| item == expected)
    }

    /// Consumes the `MergedIter` iterator and returns the remaining iterators.
    ///
    /// This method returns the internal storage, which contains the remaining "peeked"
    /// items and their associated iterators. Each element in the returned storage is
    /// a tuple of the form
    /// `(Item, Iter)`, where:
    /// - `Item` is the "peeked" value from that component iterator,
    /// - `Iter` is the remaining iterator for that component.
    ///
    /// The order of the elements in the storage may be different from the order of original
    /// iterators if [`Merged::arbitrary_tie_breaking`] was used.
    ///
    /// If an input iterator was fully consumed, it will be missing from the returned storage.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    ///
    /// let iter1 = vec![1, 3];
    /// let iter2 = vec![2, 4];
    /// let mut merged = Merged::new([iter1, iter2]).build();
    ///
    /// assert_eq!(merged.next(), Some(1));
    ///
    /// let mut storage = merged.break_up();
    ///
    /// assert_eq!(storage[0].0, 3);
    /// assert_eq!(storage[0].1.next(), None);
    /// assert_eq!(storage[1].0, 2);
    /// assert_eq!(storage[1].1.next(), Some(4));
    /// assert_eq!(storage[1].1.next(), None);
    /// # }
    /// ```
    pub fn break_up(self) -> S {
        self.peek_iters
    }
}

impl<const STABLE_TIE_BREAKING: bool, Item, Iter, S, Cmp> Iterator
    for MergedIter<STABLE_TIE_BREAKING, S, Cmp>
where
    Iter: Iterator<Item = Item>,
    S: Storage<Item = (Item, Iter)>,
    Cmp: Fn(&Item, &Item) -> Ordering,
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peek_iters.len() == 0 {
            return None;
        }
        Some(self.produce_next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Accounts for currently peeked items in both lower and upper.
        let mut lower = self.peek_iters.len();
        let mut upper = lower;
        let mut has_upper = true;
        for i in 0..self.peek_iters.len() {
            let (it_lower, it_upper) = self.peek_iters.get(i).1.size_hint();
            lower = lower.saturating_add(it_lower);
            if let Some(it_upper) = it_upper {
                let overflow;
                (upper, overflow) = upper.overflowing_add(it_upper);
                has_upper &= !overflow;
            } else {
                has_upper = false;
            }
        }
        (lower, has_upper.then_some(upper))
    }
}

impl<const STABLE_TIE_BREAKING: bool, S, Cmp, Item, Iter> Clone for MergedIter<STABLE_TIE_BREAKING, S, Cmp>
where
    Iter: Iterator<Item = Item>,
    Cmp: Clone + Fn(&Item, &Item) -> Ordering,
    S: Clone + Storage<Item = (Item, Iter)>,
{
    fn clone(&self) -> Self {
        Self {
            peek_iters: self.peek_iters.clone(),
            cmp: self.cmp.clone(),
            min_idx: self.min_idx,
            next_min_idx: self.next_min_idx,
        }
    }
}

