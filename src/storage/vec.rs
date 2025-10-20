use alloc::{collections::TryReserveError, vec::Vec};
use core::{
    fmt::Debug,
    mem::{self, ManuallyDrop},
    slice,
};

use crate::{
    internal::{
        BaseStorage, PeekIter,
        nums::unchecked_add,
        pointers::{HalfUsize, ptr_to_usize, rebase_ptr},
    },
    merge_iter::{DefaultBuilder, DefaultMergeIter},
    storage::{Storage as _, debug_formatter},
};

/// [`Vec`]-based storage for [`MergeIter`](crate::MergeIter)
///
/// Most methods mirror corresponding methods on [Vec]
#[derive(Default)]
pub struct VecStorage<IT: Iterator>(Vec<PeekIter<IT>>);

impl<IT> Clone for VecStorage<IT>
where
    IT: Iterator,
    Vec<PeekIter<IT>>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<IT: Iterator> VecStorage<IT> {
    const _CHECK: () = assert!(
        mem::size_of::<*mut PeekIter<IT>>() == mem::size_of::<usize>(),
        // TODO: Link to github
        "Non-pointer-sized pointer. Please create an issue if this error is encountered"
    );

    /// Create a new [`VecStorage`]
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Constructs a new, empty [`VecStorage`] with at least the specified capacity.
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Appends an element to the back of a collection.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    pub fn push<Iter>(&mut self, iter: Iter)
    where
        Iter: IntoIterator<IntoIter = IT>,
    {
        if let Some(peek_iter) = PeekIter::new_from_iter(iter) {
            self.0.push(peek_iter);
        }
    }

    /// Appends an element to the back of a collection.
    ///
    /// # Errors
    /// Returns an error if the new capacity exceeds `isize::MAX` _bytes_.
    pub fn try_push<Iter>(&mut self, iter: Iter) -> Result<(), TryReserveError>
    where
        Iter: IntoIterator<IntoIter = IT>,
    {
        if let Some(peek_iter) = PeekIter::new_from_iter(iter) {
            self.0.try_reserve(1)?;
            self.0.push(peek_iter);
        }
        Ok(())
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given [`VecStorage`].
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the given [`VecStorage`].
    ///
    /// # Errors
    /// Returns an error if the capacity overflows, or the allocator reports a failure
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// Reserves the minimum capacity for at least additional more elements to be inserted in the
    /// given [`VecStorage`].
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    /// Tries to reserve the minimum capacity for `additional` more elements to be inserted
    /// in the given [`VecStorage`].
    /// # Errors
    /// Returns an error if the capacity overflows, or the allocator reports a failure
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    /// Tries to construct a [`Builder`] from this storage. Allocates additional vec; if
    /// the allocator reports a failure, then an error is returned.
    ///
    /// # Errors
    /// Returns error if it fails to allocate a necessary vec for constructing a heap
    pub fn try_into_builder(
        self,
    ) -> Result<DefaultBuilder<InternalVecStorage<IT>>, TryReserveError> {
        let Self(mut storage) = self;
        storage.shrink_to_fit();
        let len = storage.len();
        let mut heap: Vec<*mut PeekIter<IT>> = Vec::new();
        heap.try_reserve_exact(len)?;
        let extra_heap_cap = HalfUsize::try_from(
            heap.capacity()
                .checked_sub(len)
                .expect("Heap capacity is smaller than requested"),
        )
        .expect("Extra heap capacity is too large");
        let extra_storage_cap = HalfUsize::try_from(
            storage
                .capacity()
                .checked_sub(len)
                .expect("Storage capacity is smaller than storage len"),
        )
        .expect("Extra storage capacity is too large");
        let storage = ManuallyDrop::new(storage).as_mut_ptr();
        let heap = ManuallyDrop::new(heap).as_mut_ptr();
        for i in 0..len {
            // SAFETY: all pointers are valid and within respective allocations
            unsafe {
                heap.add(i).write(storage.add(i));
            }
        }
        Ok(InternalVecStorage {
            storage,
            heap,
            extra_storage_cap,
            extra_heap_cap,
            len,
            initial_len: len,
        }
        .into_builder())
    }

    /// Constructs a [`Builder`] from this storage
    ///
    /// # Panics
    /// Panics if fails to allocate a necessary vec.
    #[must_use]
    pub fn into_builder(self) -> DefaultBuilder<InternalVecStorage<IT>> {
        self.try_into_builder()
            .expect("failed to build vec storage")
    }

    /// Constructs a [`MergeIter`](crate::MergeIter) from this storage with default parameters.
    ///
    /// Equivalent to calling <code>[Self::into_builder()].[build()](crate::merge_iter::Builder::build)</code>
    #[must_use]
    pub fn build(self) -> DefaultMergeIter<InternalVecStorage<IT>>
    where
        IT::Item: Ord,
    {
        self.into_builder().build()
    }
}

impl<IT> Debug for VecStorage<IT>
where
    IT: Iterator,
    PeekIter<IT>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VecStorage").field(&self.0).finish()
    }
}

impl<IT: Iterator, A> Extend<A> for VecStorage<IT>
where
    A: IntoIterator<IntoIter = IT>,
{
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        let iters = iter.into_iter();
        let _ = self.try_reserve(iters.size_hint().0);
        for iter in iters {
            self.push(iter);
        }
    }
}

impl<IT, Item> FromIterator<Item> for VecStorage<IT>
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

/// Internal representation of the [`VecStorage`] that's actually used as the
/// [`MergeIter`](crate::MergeIter)'s [`Storage`](crate::internal::BaseStorage) backend.
pub struct InternalVecStorage<IT: Iterator> {
    storage: *mut PeekIter<IT>,
    heap: *mut *mut PeekIter<IT>,
    // Extra storage capacity over the initial_len
    extra_storage_cap: HalfUsize,
    // Extra heap capacity over the initial_len
    extra_heap_cap: HalfUsize,
    initial_len: usize,
    len: usize,
}

impl<IT: Iterator> InternalVecStorage<IT> {
    #[inline]
    const fn storage_cap(&self) -> usize {
        // SAFETY: can't overflow, capacity <= isize::MAX < usize::MAX
        // as conversion is safe, because the HalfUsize type is guaranteed to be smaller than usize
        unsafe { unchecked_add(self.initial_len, self.extra_storage_cap as usize) }
    }
    #[inline]
    const fn heap_cap(&self) -> usize {
        // SAFETY: can't overflow, capacity <= isize::MAX < usize::MAX
        // as conversion is safe, because the HalfUsize type is guaranteed to be smaller than usize
        unsafe { unchecked_add(self.initial_len, self.extra_heap_cap as usize) }
    }
}

unsafe impl<IT: Iterator> BaseStorage for InternalVecStorage<IT> {
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

impl<IT> Debug for InternalVecStorage<IT>
where
    IT: Iterator,
    PeekIter<<Self as BaseStorage>::IT>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InternalVecStorage")
            .field("len", &self.len)
            .field("initial_len", &self.initial_len)
            .field("heap_cap", &self.heap_cap())
            .field("storage_cap", &self.storage_cap())
            .field("storage", &debug_formatter(self))
            .finish_non_exhaustive()
    }
}

impl<IT: Iterator> Drop for InternalVecStorage<IT> {
    fn drop(&mut self) {
        let storage;
        let heap;
        unsafe {
            storage = Vec::from_raw_parts(self.storage, 0, self.storage_cap());
            heap = Vec::from_raw_parts(self.heap, 0, self.heap_cap());
        }
        crate::storage::StorageOps::clear(self);
        drop(heap);
        drop(storage);
    }
}

// SAFETY: InternalVecStorage is an owning container of two Vecs,
// one containing `PeekIter<IT>` and the other containing `*mut PeekIter<IT>`.
// It's safe for them to be send and sync, if the `Vec<PeekIter<IT>>` is send and sync
// respectively
unsafe impl<IT> Send for InternalVecStorage<IT>
where
    IT: Iterator,
    Vec<PeekIter<IT>>: Send,
{
}

// SAFETY: see above.
unsafe impl<IT> Sync for InternalVecStorage<IT>
where
    IT: Iterator,
    Vec<PeekIter<IT>>: Sync,
{
}

impl<IT> Clone for InternalVecStorage<IT>
where
    IT: Iterator,
    PeekIter<IT>: Clone,
{
    fn clone(&self) -> Self {
        let len = self.len;
        if len == 0 {
            let storage = Vec::new();
            let heap = Vec::new();
            // Create an empty storage
            return Self {
                extra_heap_cap: HalfUsize::try_from(heap.capacity())
                    .expect("Extra heap capacity is too large"),
                extra_storage_cap: HalfUsize::try_from(storage.capacity())
                    .expect("Extra storage capacity is too large"),
                storage: ManuallyDrop::new(storage).as_mut_ptr(),
                heap: ManuallyDrop::new(heap).as_mut_ptr(),
                initial_len: len,
                len,
            };
        }
        let mut storage: Vec<PeekIter<IT>> = Vec::with_capacity(len);
        let extra_storage_cap = HalfUsize::try_from(
            storage
                .capacity()
                .checked_sub(len)
                .expect("Storage capacity is smaller than requested"),
        )
        .expect("Extra storage capacity is too large");

        let mut heap: Vec<usize> = Vec::with_capacity(len);
        let extra_heap_cap = HalfUsize::try_from(
            heap.capacity()
                .checked_sub(len)
                .expect("Heap capacity is smaller than requested"),
        )
        .expect("Extra heap capacity is too large");

        if len == self.initial_len {
            // no holes in the storage, just clone all of the items
            storage.extend_from_slice(
                // Storage does not contain any uninit values
                unsafe { slice::from_raw_parts(self.storage, len) },
            );
            let storage = ManuallyDrop::new(storage).as_mut_ptr();
            // inner pointers are uninitialized
            let heap: *mut *mut PeekIter<IT> = ManuallyDrop::new(heap).as_mut_ptr().cast();
            for i in 0..len {
                unsafe {
                    heap.add(i).write(rebase_ptr(
                        self.storage,
                        // add offset of i'th element of the self.heap to the new storage
                        self.heap.add(i).read(),
                        storage,
                    ));
                }
            }
            return Self {
                storage,
                heap,
                extra_heap_cap,
                extra_storage_cap,
                len,
                initial_len: len,
            };
        }

        heap.extend(0..len);
        // Heap is a vec of indexes 0..len

        // Sort the heap in the order of the original storage
        heap.sort_unstable_by_key(|&pos|
            // SAFETY: self.heap is valid for reads from 0 to len
            unsafe { self.heap.add(pos).read() });

        // Now heap is a vec of indexes into the original heap,
        // such that self.heap[heap[N]] is the N'th live iterator in order of insertion

        // Filling the storage with cloned items, preserving the order of insertion
        storage.extend(heap.iter().map(|&offset|
            // SAFETY: self.heap is valid for reads from 0 to len, and only points to
            // live elements; heap consists only of indexes 0..len
            unsafe { (&*self.heap.add(offset).read()).clone() }));

        // This is a bit of a complex operation.
        // Right now heap[N] is the index into the original heap, i.e.
        // heap[3] == 5 means that pointer to (the original of the cloned) storage[3]
        // is at the self.heap[5].
        // So in the final heap `heap[5]` should be `*mut storage[3]`
        // In order to do this in-place we must perform a cyclic shift:
        // heap[pos] <- idx <- pos <- heap[pos]
        // But we need to differentiate between already rotated elements and new once,
        // so we're doing
        // heap[pos] <- idx | MARKER; idx <- pos <- heap[pos]
        // and checking for marker's presence in our pos values.

        const MARKER: usize = {
            let marker = 1_usize << (usize::BITS - 1);
            assert!((marker > isize::MAX as usize) && (marker & (isize::MAX as usize)) == 0);
            // MARKER is good to use as no valid vector offset can set the last bit (since Vec
            // can't hold more than isize::MAX bytes/non-ZST-elements)
            marker
        };

        for mut idx in 0..len {
            // SAFETY: idx in range 0..len
            let mut pos = *unsafe { heap.get_unchecked(idx) };
            if pos & MARKER != 0 {
                continue;
            }
            loop {
                pos = mem::replace(
                    // SAFETY: pos in range 0..len
                    unsafe { heap.get_unchecked_mut(pos) },
                    mem::replace(&mut idx, pos) | MARKER,
                );
                if pos & MARKER != 0 {
                    break;
                }
            }
        }
        // Right now heap contains correct indexes into storage (in the order of the
        // self.heap), or'ed with MARKER. We're replacing them with pointers to storage
        // and we're done!

        let storage = ManuallyDrop::new(storage).as_mut_ptr();
        // We just casted plain usizes to *mut PeekIter<IT>, but we're only reading their
        // addresses as usizes and re-initializing them with pointers to new storage
        let heap: *mut *mut PeekIter<IT> = ManuallyDrop::new(heap).as_mut_ptr().cast();
        for i in 0..len {
            // SAFETY: contents of heap are offsets in range 0..len, they are within the allocated
            // storage range
            unsafe {
                let p = heap.add(i);
                p.write(storage.add(ptr_to_usize(p.read()) & !MARKER));
            }
        }

        Self {
            storage,
            heap,
            extra_heap_cap,
            extra_storage_cap,
            len,
            initial_len: len,
        }
    }
}
