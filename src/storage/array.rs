use core::{
    cell::Cell,
    fmt::{Debug, Display},
    marker::{PhantomData, PhantomPinned},
    mem::MaybeUninit,
    pin::Pin,
};

use crate::{
    internal::{BaseStorage, PeekIter},
    merge_iter::{DefaultBuilder, DefaultMergeIter},
    storage::{Storage as _, debug_formatter},
};

/// Error signaling an overflow of the array's capacity
#[derive(Debug, Clone, Copy)]
pub struct ArrayCapacityOverflow;

impl Display for ArrayCapacityOverflow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Capacity overflow")
    }
}

#[rustversion::since(1.81)]
impl core::error::Error for ArrayCapacityOverflow {}

/// Fixed-capacity array-based storage for [`MergeIter`](crate::MergeIter)
pub struct ArrayStorage<const CAP: usize, IT: Iterator> {
    storage: [MaybeUninit<PeekIter<IT>>; CAP],
    heap: [MaybeUninit<*mut PeekIter<IT>>; CAP],
    len: Cell<usize>,
    _p: PhantomPinned,
}

impl<const CAP: usize, IT: Iterator> Debug for ArrayStorage<CAP, IT>
where
    PeekIter<IT>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ArrayStorage")
            .field("CAP", &CAP)
            .field("len", &self.len)
            .field("storage", &
                // SAFETY: array is initialized up to self.len()
                unsafe {
                    core::slice::from_raw_parts(
                        self.storage.as_ptr().cast::<PeekIter<IT>>(),
                        self.len(),
                    )
                })
            .finish_non_exhaustive()
    }
}

#[inline(always)]
const fn uninit_array<const CAP: usize, T>() -> [MaybeUninit<T>; CAP] {
    // SAFETY: array of MaybeUninit does not need initialization
    unsafe { MaybeUninit::<[MaybeUninit<T>; CAP]>::uninit().assume_init() }
}

impl<IT: Iterator> ArrayStorage<0, IT> {
    /// Create [`ArrayStorage`] with given capacity and inferred iterator type
    #[must_use]
    #[inline(always)]
    pub const fn with_capacity<const CAP: usize>() -> ArrayStorage<CAP, IT> {
        ArrayStorage::new()
    }
}

impl<const CAP: usize, IT: Iterator> ArrayStorage<CAP, IT> {
    /// Create a new [`ArrayStorage`]
    ///
    /// # Example
    /// Building a merge iterator from an `ArrayStorage`.
    ///
    /// ```
    /// use core::{iter, pin::pin};
    ///
    /// use iter_merge::ArrayStorage;
    ///
    /// let mut storage: ArrayStorage<5, _> = ArrayStorage::new();
    /// storage.push(iter::once(2));
    /// storage.push(iter::once(1));
    /// let storage = pin!(storage);
    /// let it = storage.build();
    /// assert!(it.eq([1, 2]));
    /// ```
    #[must_use]
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            storage: uninit_array(),
            heap: uninit_array(),
            len: Cell::new(0),
            _p: PhantomPinned,
        }
    }

    /// Creates a new [`ArrayStorage`] with the same `CAP` as
    /// the provided array.
    ///
    /// # Example
    /// Building a merge iterator from an `ArrayStorage`.
    ///
    /// ```
    /// use core::{iter, pin::pin};
    ///
    /// use iter_merge::ArrayStorage;
    /// let storage = ArrayStorage::from_arr([[1, 3], [2, 4]]);
    /// assert_eq!(storage.capacity(), 2)
    /// ```
    #[must_use]
    #[inline]
    pub fn from_arr<T: IntoIterator<IntoIter = IT>>(iters: [T; CAP]) -> Self {
        let mut res = Self::new();
        res.extend(iters);
        res
    }

    /// Returns the number of non-empty iterators stored in [`ArrayStorage`]
    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    /// Returns the (fixed) capacity of [`ArrayStorage`]
    #[inline]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Returns `true` if this [`ArrayStorage`] is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends an element to the back of a collection.
    ///
    /// # Panics
    ///
    /// Panics if the collection is full.
    pub fn push<Iter>(&mut self, iter: Iter)
    where
        Iter: IntoIterator<IntoIter = IT>,
    {
        self.try_push(iter).unwrap();
    }

    /// Tries to append an element to the back of a collection.
    /// # Errors
    /// Returns error if the [`ArrayStorage`] is full
    pub fn try_push<Iter>(&mut self, iter: Iter) -> Result<(), ArrayCapacityOverflow>
    where
        Iter: IntoIterator<IntoIter = IT>,
    {
        if let Some(peek_iter) = PeekIter::new_from_iter(iter) {
            let len = self.len.get();
            if len >= CAP {
                return Err(ArrayCapacityOverflow);
            }
            self.storage[len].write(peek_iter);
            self.len.set(len.checked_add(1).expect("unreachable"));
        }
        Ok(())
    }

    /// Constructs a [`Builder`] from this storage.
    ///
    /// Note: the storage cannot move for [`MergeIter`](crate::MergeIter) to work, thus
    /// you need to call this method on a pinned mutable reference.
    ///
    /// Example:
    /// ```
    /// use core::{iter, pin::pin};
    ///
    /// use iter_merge::ArrayStorage;
    /// let mut storage = ArrayStorage::<5, _>::new();
    /// storage.push(iter::once(1));
    /// let storage = pin!(storage);
    /// let _builder = storage.into_builder();
    /// ```
    #[must_use]
    pub fn into_builder(self: Pin<&mut Self>) -> DefaultBuilder<InternalArrayStorage<'_, IT>> {
        let len = self.len.replace(0);
        debug_assert!(len <= CAP);
        let (storage, heap) = {
            // SAFETY: we're never moving the data out of mut_ref, we're just copying the
            // mut pointers.
            // InternalArrayStorage lives for 'a, same as our pinned pointer
            // during this time it's safe to rely on pin guarantee
            let mut_ref = unsafe { Pin::get_unchecked_mut(self) };
            (
                mut_ref.storage.as_mut_ptr().cast::<PeekIter<IT>>(),
                mut_ref.heap.as_mut_ptr().cast::<*mut PeekIter<IT>>(),
            )
        };
        for i in 0..len {
            // SAFETY: storage pointer is valid for adding up to CAP (>= len), heap - for writitng
            //         up to CAP (>= len).
            //         self is pinned up to 'a, so we are relying on pin guarantee by constructing
            //         InternalArrayStorage valid for 'a
            unsafe {
                heap.add(i).write(storage.add(i));
            }
        }
        InternalArrayStorage {
            heap,
            len,
            _p: PhantomData,
        }
        .into_builder()
    }

    /// Constructs a [`MergeIter`] from this storage with default parameters.
    ///
    /// Equivalent to calling <code>[Self::into_builder()].[build()](crate::merge_iter::Builder::build)</code>
    #[must_use]
    pub fn build(self: Pin<&mut Self>) -> DefaultMergeIter<InternalArrayStorage<'_, IT>>
    where
        IT::Item: Ord,
    {
        self.into_builder().build()
    }
}

impl<const CAP: usize, IT, Item> FromIterator<Item> for ArrayStorage<CAP, IT>
where
    IT: Iterator,
    Item: IntoIterator<IntoIter = IT>,
{
    fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
        let mut res = Self::new();
        res.extend(iter);
        res
    }
}

impl<const CAP: usize, IT: Iterator, A> Extend<A> for ArrayStorage<CAP, IT>
where
    A: IntoIterator<IntoIter = IT>,
{
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<const CAP: usize, IT: Iterator> Default for ArrayStorage<CAP, IT> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize, IT: Iterator> Drop for ArrayStorage<CAP, IT> {
    fn drop(&mut self) {
        // We are dropping potentially pinned value.
        // As soon as the [`build`] method is called
        // (first method that relies on pinning guarantees)
        // the self.len is set to 0 and never modified until pinned borrow ends,
        // so this loop is a noop and we're not violating any guarantees of pin.
        for i in 0..self.len.replace(0) {
            // SAFETY: up to self.len items are initialized, the pointers were not given
            // to Heap that could've invalidated some stored items.
            // self.len is replaced by 0, so there's no possibility of double-free,
            // only memory leak if the item drop code panics
            unsafe {
                self.storage[i].assume_init_drop();
            }
        }
    }
}

/// Internal representation of the [`ArrayStorage`] that's actually used as the
/// [`MergeIter`](crate::MergeIter)'s [`Storage`](crate::internal::BaseStorage) backend.
pub struct InternalArrayStorage<'a, IT: Iterator> {
    heap: *mut *mut PeekIter<IT>,
    len: usize,
    // represents us holding the pinned ArrayStorage, capacity is irrelevant,
    // this is only for lifetime management
    _p: PhantomData<Pin<&'a mut ArrayStorage<1, IT>>>,
}

unsafe impl<IT: Iterator> BaseStorage for InternalArrayStorage<'_, IT> {
    type IT = IT;

    #[inline]
    fn heap(&self) -> *mut *mut PeekIter<IT> {
        self.heap
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }
}

impl<IT: Iterator> Debug for InternalArrayStorage<'_, IT>
where
    PeekIter<<Self as BaseStorage>::IT>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InternalArrayStorage")
            .field("len", &self.len)
            .field("storage", &debug_formatter(self))
            .finish_non_exhaustive()
    }
}

impl<IT: Iterator> Drop for InternalArrayStorage<'_, IT> {
    fn drop(&mut self) {
        crate::storage::StorageOps::clear(self);
        // The storage itself is owned by ArrayStorage and will be deallocated by it
    }
}

// SAFETY: InternalArrayStorage is just a reference to pinned ArrayStorage.
// It's safe for them to be send and sync, if the `Pin<&'a mut ArrayStorage<IT>>` is send and sync
// respectively
unsafe impl<'a, IT> Send for InternalArrayStorage<'a, IT>
where
    IT: Iterator,
    Pin<&'a mut ArrayStorage<1, IT>>: Send,
{
}

// SAFETY: see above.
unsafe impl<'a, IT> Sync for InternalArrayStorage<'a, IT>
where
    IT: Iterator,
    Pin<&'a mut ArrayStorage<1, IT>>: Sync,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_overflow() {
        let mut s: ArrayStorage<1, _> = ArrayStorage::default();
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
        s.push([1, 2, 3]);
        assert_eq!(s.len(), 1);
        assert!(!s.is_empty());
        assert!(matches!(s.try_push([4, 5, 6]), Err(ArrayCapacityOverflow)));
    }
}
