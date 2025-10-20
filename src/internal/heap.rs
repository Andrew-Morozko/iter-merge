use core::{cmp::Ordering, mem, ptr};

use crate::{
    comparators::Comparator,
    internal::{
        Hole, Item, Iter, PeekIter,
        nums::{unchecked_add, unchecked_mul, unchecked_sub},
        pointers::UniquePtr,
    },
    storage::Storage,
};

/// Min heap organized on storage `S` and ordered by `CMP`.
/// Heap structure:
/// 0 - min element
/// 1 - second min element, heap root
/// 2, 3 - children of the heap root
/// [idx*2, idx*2+1] - children of the idx element
#[derive(Debug, Clone)]
pub(crate) struct Heap<S, CMP> {
    pub(crate) comparator: CMP,
    pub(crate) storage: S,
}

impl<CMP, S> Heap<S, CMP>
where
    CMP: Comparator<Item<S>>,
    S: Storage,
{
    pub(crate) fn new(comparator: CMP, storage: S) -> Self {
        let mut res = Self {
            comparator,
            storage,
        };
        res.heapify_storage();
        res
    }

    #[inline]
    fn cmp(&self, a: &PeekIter<Iter<S>>, b: &PeekIter<Iter<S>>) -> Ordering {
        debug_assert!(!ptr::eq(a, b), "shouldn't ever compare the item to itself");
        self.comparator.compare(&a.item, &b.item)
    }

    fn heapify_storage(&mut self) {
        // This heapify process is done in two phases:
        // 1. First, we perform a bottom-up heapify on the range [1..], ensuring that the heap
        //    rooted at index 1 is a valid min-heap.
        // 2. Then, we explicitly fix the [0,1] pair so that the overall invariant [0] <= [1] holds.
        // This establishes that [1..] forms a valid heap and the minimum element is always at [0],
        // which is essential for the correctness of subsequent heap operations.
        if self.storage.len() <= 1 {
            return;
        }

        for n in (1..=(self.storage.len() / 2)).rev() {
            // SAFETY: n in range [1; self.storage.len() / 2], it's < self.storage.len()
            unsafe {
                self.sift_down_element(n);
            }
        }
        // SAFETY: len >= 2, therefore pointers are as safe as references
        unsafe {
            let first = self.storage.first();
            let second = self.storage.second();
            if self.cmp(&**first, &**second).is_gt() {
                ptr::swap_nonoverlapping(first, second, 1);
                self.sift_down_top();
            }
        }
    }

    /// Take an element at the top of the heap and move it down the heap,
    /// while its children are smaller.
    ///
    /// # Safety
    ///
    /// The caller must guarantee:
    /// * `self.storage.len() >= 2`
    /// * Heap [1; `self.storage.len()`) can be mutated and elements at these locations
    ///   can be accessed via reference (&). i.e.: no &mut to the [1; `self.storage.len()`)
    pub(crate) unsafe fn sift_down_top(&mut self) {
        // SAFETY: caller guarantees it's safe
        unsafe {
            self.sift_down_element(1);
        }
    }

    /// Take an element at `pos` and move it down the heap,
    /// while its children are smaller.
    ///
    /// # Safety
    ///
    /// The caller must guarantee:
    /// * `1 <= pos < self.storage.len()`
    ///   (therefore `self.storage.len()` >= 2)
    /// * Heap [pos; end) can be mutated and elements at these locations
    ///   can be accessed via reference (&). i.e.: no &mut to the [pos; end)
    #[inline] // only used in sift_down and heapify
    unsafe fn sift_down_element(&mut self, pos: usize) {
        let len = self.storage.len();
        #[allow(clippy::checked_conversions)]
        {
            debug_assert!(pos >= 1 && pos < len && len >= 2 && len <= isize::MAX as usize);
        }
        // SAFETY: The caller guarantees that pos < end <= self.storage.len().
        let mut hole = unsafe { Hole::new(self.storage.heap(), pos) };
        // hole.pos * 2; never overflows because self.storage.len() is <= isize::MAX
        let mut child = unsafe { unchecked_mul(hole.pos, 2) };
        // self.storage.len() is at least 2, so this never underflows
        let last_el = unsafe { unchecked_sub(len, 1) };
        while child < last_el {
            // SAFETY: child <= len - 2, so child + 1 never overflows
            let child2 = unsafe { unchecked_add(child, 1) };
            // find the smaller of the two children
            if self
                // SAFETY: child, child+1 are < len and != hole.pos
                .cmp(unsafe { &**hole.get(child) }, unsafe {
                    &**hole.get(child2)
                })
                .is_gt()
            {
                child = child2;
            }

            // if we are already in order, stop.
            if self
                // SAFETY: child is < len and != hole.pos, hole.elt is a valid item
                .cmp(unsafe { &**hole.elt }, unsafe { &**hole.get(child) })
                .is_le()
            {
                return;
            }
            // SAFETY: child != pos and is valid element
            unsafe {
                hole.move_to(child);
            }
            // hole.pos * 2; never overflows because self.storage.len() is <= isize::MAX
            child = unsafe { unchecked_mul(hole.pos, 2) };
        }
        if child == last_el {
            if self
                // SAFETY: child is < len and != hole.pos, hole.elt is a valid item
                .cmp(unsafe { &**hole.elt }, unsafe { &**hole.get(child) })
                .is_le()
            {
                return;
            }
            // SAFETY: child != pos and is valid element
            unsafe {
                hole.move_to(child);
            }
        }
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn into_vec(mut self) -> alloc::vec::Vec<Item<S>> {
        let mut res = alloc::vec::Vec::new();
        let mut hint_low = self.storage.len();
        if hint_low == 0 {
            return res;
        }
        self.storage
            .map_items(|it| hint_low = hint_low.saturating_add(it.iter.size_hint().0));
        res.reserve_exact(hint_low);

        // SAFETY: len >= 1, therefore pointer to first is valid. We won't create other pointers to
        //         the first element in this function, so it's unique.
        let mut first = unsafe { UniquePtr::new(*self.storage.first()) };
        if self.storage.len() >= 3 {
            // SAFETY: len >= 2, therefore pointer to second is valid. We won't create other
            //         pointers to the first element in this scope
            let mut second = unsafe { UniquePtr::new(*self.storage.second()) };
            loop {
                if let Some(item) = first.advance() {
                    res.push(item);
                    if self.cmp(&*first, &*second).is_le() {
                        // order is still correct
                        continue;
                    }
                    // second is now the smallest, and first needs to be on the
                    // heap and sifted down. Heap operations do not touch the first
                    // pointer, so it's valid for us to keep holding it as UniquePtr
                    unsafe {
                        self.storage
                            .second()
                            .write(mem::replace(&mut first, second).into_ptr());
                        self.storage.first().write(first.as_ptr());
                    }

                    // SAFETY: heap is not empty
                    // The only live mutable reference is to the first element
                    // self.sift_down() never touches the first element
                    // so it's ok to have that reference live
                    unsafe {
                        self.sift_down_top();
                    }
                    // update second reference
                    second = unsafe { UniquePtr::new(*self.storage.second()) };
                } else {
                    // Heap: [first, second, ..., last], and the first is exhausted
                    // Performing operations to get to:
                    // [second, last, ...]
                    // and then sifting down to get to
                    // [second, new_second, ..., last, ...]

                    // SAFETY: self.len() >= 3
                    unsafe {
                        // last replaces second
                        self.storage.second().write(self.storage.pop_last());
                        // second replaces first
                        self.storage.first().write(second.as_ptr());
                    };

                    let popped = mem::replace(&mut first, second);
                    res.push(
                        // SAFETY: heap updated, there's no way to get another instance of first
                        unsafe { popped.into_owning_ptr() }.into_last_item(),
                    );

                    if self.storage.len() == 2 {
                        break;
                    }
                    // SAFETY: if here - self.storage.len() is still >= 3,
                    // and second can't alias the first since the heap was modified
                    second = unsafe {
                        self.sift_down_top();
                        UniquePtr::new(*self.storage.second())
                    }
                }
            }
        }
        if self.storage.len() == 2 {
            // SAFETY: len >= 2, therefore pointers are as safe as references
            let mut second = unsafe { UniquePtr::new(*self.storage.second()) };
            // We are not updating the heap when there are only two iterators left. Heap remains in
            // the correct state for drop handling, just the order of items may be incorrect
            while let Some(item) = first.advance() {
                res.push(item);
                if self.cmp(&*first, &*second).is_gt() {
                    mem::swap(&mut first, &mut second);
                }
            }
            // first iterator is exhausted, second still has some items
            unsafe {
                self.storage.set_len(1);
            }

            // We haven't updated the heap during 2-iterator consumption, place the non-exhausted
            // iterator on the heap.
            // SAFETY: len() == 1
            unsafe {
                self.storage.first().write(second.as_ptr());
            }

            let popped = mem::replace(&mut first, second);
            // SAFETY: Now heap is in state [second], the only reference to popped is ours
            res.push(unsafe { popped.into_owning_ptr() }.into_last_item());
        }
        debug_assert!(self.storage.len() == 1);
        // SAFETY: storage.len() > 0 (storage.len() == 1)
        unsafe {
            self.storage.set_len(0);
        }

        // SAFETY: Now heap is empty, the only reference to first is ours.
        let PeekIter { item, iter } = unsafe { first.into_owning_ptr() }.read();
        res.push(item);
        res.extend(iter);
        res
    }

    pub(crate) fn pop_front_item(&mut self) -> Option<Item<S>> {
        Some(match self.storage.len() {
            2 => {
                let mut first = unsafe { UniquePtr::new(*self.storage.first()) };
                let second = unsafe { UniquePtr::new(*self.storage.second()) };
                if let Some(item) = first.advance() {
                    if self.cmp(&*first, &*second).is_gt() {
                        // SAFETY: len() == 2
                        unsafe {
                            self.storage.first().write(second.into_ptr());
                            self.storage.second().write(first.into_ptr());
                        }
                    }
                    item
                } else {
                    // SAFETY: len() >= 2, first is removed from heap
                    unsafe {
                        self.storage.set_len(1);
                    }
                    unsafe {
                        // second replaces first
                        self.storage.first().write(second.into_ptr());
                        // now to_pop is the only reference to this item
                        first.into_owning_ptr()
                    }
                    .into_last_item()
                }
            }
            1 => {
                let mut first = unsafe { UniquePtr::new(*self.storage.first()) };
                first.advance().unwrap_or_else(|| {
                    // SAFETY: len() == 1, first is removed from heap
                    unsafe {
                        self.storage.set_len(0);
                        // now to_pop is the only reference to this item
                        first.into_owning_ptr()
                    }
                    .into_last_item()
                })
            }
            0 => return None,
            _ => {
                // 3.. is not supported on MSRV
                let mut first = unsafe { UniquePtr::new(*self.storage.first()) };
                let second = unsafe { UniquePtr::new(*self.storage.second()) };
                if let Some(item) = first.advance() {
                    if self.cmp(&*first, &*second).is_gt() {
                        // SAFETY: len() >= 3
                        unsafe {
                            self.storage.first().write(second.into_ptr());
                            self.storage.second().write(first.into_ptr());
                            // SAFETY: no references to heap are live and len() >= 3
                            self.sift_down_top();
                        }
                    }
                    item
                } else {
                    let item = unsafe {
                        // last replaces first
                        self.storage.second().write(self.storage.pop_last());
                        // second replaces first
                        self.storage.first().write(second.into_ptr());
                        // first is no longer accessible from the heap
                        first.into_owning_ptr()
                    }
                    .into_last_item();

                    // SAFETY: no references to heap are live and len() >= 2
                    unsafe {
                        self.sift_down_top();
                    }
                    item
                    // SAFETY: len() >= 3, first is removed from heap
                }
            }
        })
    }

    pub(crate) fn pop_front_iter(&mut self) -> Option<PeekIter<Iter<S>>> {
        let item;
        unsafe {
            match self.storage.len() {
                3 => {
                    item = self
                        .storage
                        .first()
                        .replace(self.storage.second().replace(self.storage.pop_last()))
                        .read();
                }
                2 => item = self.storage.first().replace(self.storage.pop_last()).read(),
                1 => item = self.storage.pop_last().read(),
                0 => return None,
                _ => {
                    // 4.. is not supported on MSRV
                    item = self
                        .storage
                        .first()
                        .replace(self.storage.second().replace(self.storage.pop_last()))
                        .read();
                    self.sift_down_top();
                }
            }
        }
        Some(item)
    }
}
