//! Internal implementation details of this library.
//!
//! Typically you shouldn't need to touch these types, unless you're implementing another
//! storage backend for iter-merge.
//!
//! [`PeekIter`] holds an iterator and the eagerly peeked item from it. The iterator with the
//! smallest (according to the [`comparator`](crate::comparators)) [`item`](PeekIter::item)
//! will be advanced.
//!
//! [`PeekIter`]s are stored within some contiguous allocation of `MaybeUninit<PeekIter>` type in
//! order of insertion (to make [`tie breakers`](crate::comparators::tie_breaker) work by comparing
//! `&PeekIter::item` by numeric pointer value).
//!
//! The heap is a contiguous allocation of `*mut PeekIter` type. Pointers in the heap represent
//! all currently live `PeekIter`s. This library only works with the heap via [`BaseStorage`] trait.
//!
//! Heap is constructed to store `*mut PeekIter`s in the following order:
//! ```custom
//! [
//!     smallest,
//!     second_smallest (also the root of the binary min-heap),
//!     second_smallest_child_1, second_smallest_child_2,
//!     ...
//! ]
//! ```
mod heap;
use core::mem;
pub(crate) mod nums;
pub(crate) mod pointers;

pub(crate) use heap::Heap;
mod hole;
pub(crate) use hole::Hole;

/// Holds within itself one peeked item from the iterator and the iterator itself.
/// It's like [`iter::Peekable`](core::iter::Peekable), except eager.
#[derive(Debug)]
pub struct PeekIter<IT: Iterator> {
    /// Item peeked from the iter
    pub item: IT::Item,
    /// Iterator, containing the rest of the items
    pub iter: IT,
}

impl<IT> Clone for PeekIter<IT>
where
    IT: Iterator + Clone,
    IT::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            item: self.item.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<IT: Iterator> PeekIter<IT> {
    const _CHECK: () = assert!(
        mem::size_of::<Self>() > 0,
        "iter-merge doesn't work when both iterator and item are ZST",
    );

    /// Create a new [`PeekIter`] from a `peeked_item` and `iter`
    #[inline]
    pub const fn new(peeked_item: IT::Item, iter: IT) -> Self {
        Self {
            item: peeked_item,
            iter,
        }
    }

    /// Advances the iterator, returning current peeked [`item`](Self::item) and replacing it
    /// with new item from the [`iter`](Self::iter). If `iter` is out of items - returns None,
    /// with [`item`](Self::item) being the last item of the iterator.
    pub fn advance(&mut self) -> Option<IT::Item> {
        let Self { item, iter } = self;
        iter.next().map(|new_item| mem::replace(item, new_item))
    }

    /// Create a new [`PeekIter`] from an `iter`
    ///
    /// If the iterator is empty - returns None.
    pub fn new_from_iter<Iter>(iter: Iter) -> Option<Self>
    where
        Iter: IntoIterator<IntoIter = IT>,
    {
        let mut iter = iter.into_iter();
        iter.next()
            .map(move |peeked_item| Self::new(peeked_item, iter))
    }
}

/// Trait implemented by all storage backends.
///
/// # Invariant
/// Assuming no external mutation of pointers or length, after any call to this library first
/// [`len`](BaseStorage::len) elements of [`heap`](BaseStorage::heap) are valid unique pointers to
/// valid [`PeekIter`] items.
///
/// In other words, [`heap`](BaseStorage::heap) upholds all of the properties to be treated as a
/// slice `&[&mut PeekIter]` with length [`len`](BaseStorage::len)
///
/// # Safety
/// Storage (conceptually) represents an owned array of [`PeekIter<IT: Iterator>`] with length
/// `len()`, that could be accessed indirectly via heap of `*mut PeekIter` with length
/// [`len`](BaseStorage::len).
///
/// Simple way to satisfy all these contracts:
/// * allocate and fill `[PeekIter; CAP]`, do not access or create references until `drop()`;
///   make sure that this allocation could not move (pinned) until the `drop()`
/// * allocate `[*mut PeekIter; CAP]` and fill it with pointers to previous
///   allocation, do not access or create references until drop, make sure that this allocation
///   could not move (pinned) until the `drop()`
/// * Provide access to the second allocation as [`BaseStorage::heap`], and to its length as
///   [`BaseStorage::len`], initially equal to `CAP`
/// * In `Drop`:
///   * deallocate the remaining live [`PeekIter`]s via call to
///     [`StorageOps::clear()`](crate::internal::StorageOps::clear)
///   * deallocate heap and storage, assuming that they are uninitialized, with the same initial
///     capacity
///     (`[MaybeUninit<*mut PeekIter>; CAP]` and `[MaybeUninit<PeekIter>; CAP]`)
#[allow(clippy::len_without_is_empty)]
pub unsafe trait BaseStorage {
    /// Iterator contained in this storage
    type IT: Iterator;

    /// Pointer to the heap of pointers
    fn heap(&self) -> *mut *mut PeekIter<Self::IT>;

    /// Length of the heap of pointers
    fn len(&self) -> usize;

    /// Set the length of the storage
    /// # Safety
    /// Caller guarantees that heap elements in `self.len()..new_len` are initialized
    unsafe fn set_len(&mut self, new_len: usize);

    /// Returns true if [`Self::len`](crate::internal::BaseStorage::len) == 0
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Provides access to the iterator type within the storage
pub type Iter<S> = <S as BaseStorage>::IT;
/// Provides access to the iterator's item type within the storage
pub type Item<S> = <Iter<S> as Iterator>::Item;

/// Extra methods for working with [`BaseStorage`] for this library
///
/// It is implemented automatically for all implementors of [`BaseStorage`]
pub trait StorageOps: BaseStorage {
    /// Decrements length by 1 and returns the new length.
    /// Does nothing to the last item, so
    /// `self.heap().add(self.dec_len())` is still a valid pointer to a live item,
    /// but you are guaranteed that this trait will never return it again (if safety contracts
    /// are not broken)
    /// # Safety
    /// Caller guarantees that `self.len()` > 0
    #[inline]
    unsafe fn dec_len(&mut self) -> usize {
        let mut len = self.len();
        debug_assert!(len != 0);
        len = len.wrapping_sub(1);
        // SAFETY: decreasing length is safe, caller guaranteed that len != 0
        unsafe {
            self.set_len(len);
        }
        len
    }

    /// Produces pointer to the first (smallest) item
    /// Pointers are valid, initialized and unique
    /// It's valid to treat them as mut refs if no other
    /// mut refs to the `first()` exist.
    /// # Safety
    /// Caller guarantees that `len()` >= 1
    #[inline]
    unsafe fn first(&self) -> *mut *mut PeekIter<Self::IT> {
        debug_assert!(self.len() >= 1);
        self.heap()
    }

    /// Produces pointer to the second (second-smallest) item
    /// which is also the root of the binary heap.
    /// Pointers are valid, initialized and unique
    /// It's valid to treat them as mut refs if no other
    /// mut refs to the `second()` exist.
    /// # Safety
    /// Caller guarantees that `len()` >= 2
    #[inline]
    unsafe fn second(&self) -> *mut *mut PeekIter<Self::IT> {
        debug_assert!(self.len() >= 2);
        // SAFETY: caller guarantees it's safe
        unsafe { self.heap().add(1) }
    }

    /// Produces pointer to the last item in the heap and decrements its length
    /// by 1.
    /// Pointers are valid, initialized and unique
    /// It's valid to treat them as mut refs.
    /// # Safety
    /// Caller guarantees that `len()` != 0.
    /// The operation will produce the same pointer as
    /// `first()` or `second()` if `len()` is 1 or 2.
    #[inline]
    unsafe fn pop_last(&mut self) -> *mut PeekIter<Self::IT> {
        debug_assert!(self.len() != 0);
        // SAFETY: caller guarantees it's safe
        unsafe { self.heap().add(self.dec_len()).read() }
    }

    /// Drops all remaining items in the storage and sets its length to 0.
    /// It's safe to call multiple times (repeated calls are no-ops)
    fn clear(&mut self) {
        let len = self.len();
        // SAFETY: decreasing length is safe
        unsafe {
            self.set_len(0);
        }
        for i in 0..len {
            // SAFETY: this operations are valid for `len` heap items
            unsafe {
                self.heap().add(i).read().drop_in_place();
            };
        }
    }

    /// Iterates over all items in the heap and calls `func` for each
    /// Only the first two items are in order, order of the rest is *unspecified*.
    #[inline]
    fn map_items(&self, mut func: impl FnMut(&PeekIter<Self::IT>)) {
        for i in 0..self.len() {
            func(
                // SAFETY: pointers up to self.len() are valid
                unsafe { &**self.heap().add(i) },
            );
        }
    }

    /// Peeks the first item of the heap
    #[inline]
    fn peek(&self) -> Option<&Item<Self>> {
        if self.is_empty() {
            return None;
        }
        // SAFETY: len >= 1
        Some(unsafe { &(**self.first()).item })
    }

    /// Pops the last item+iterator tuple in the heap (no order guaranteed, heap structure is preserved)
    #[inline]
    fn pop_last_item(&mut self) -> Option<(Item<Self>, Self::IT)> {
        if self.is_empty() {
            return None;
        }
        // SAFETY: self.len() != 0, heap items are valid
        let PeekIter { item, iter } = unsafe { self.heap().add(self.dec_len()).read().read() };
        Some((item, iter))
    }
}

impl<S: BaseStorage> StorageOps for S {}
