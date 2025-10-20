use core::{mem::ManuallyDrop, ptr};

/// Hole represents a hole in a slice i.e., an index without valid value
/// (because it was moved from or duplicated).
/// In drop, `Hole` will restore the slice by filling the hole
/// position with the value that was originally removed.
pub(crate) struct Hole<T> {
    pub(crate) data: *mut T,
    pub(crate) pos: usize,
    pub(crate) elt: ManuallyDrop<T>,
}

impl<T> Hole<T> {
    /// Creates a new `Hole` at the given position in the data slice.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `pos < len` for the backing slice, and that no other
    /// mutable references or aliasing exist for the duration of the `Hole`'s lifetime.
    /// The element at `pos` is logically removed and must not be accessed except via the `Hole`.
    pub(crate) unsafe fn new(data: *mut T, pos: usize) -> Self {
        Self {
            data,
            pos,
            elt: ManuallyDrop::new(
                // SAFETY: caller upholds invariants described above.
                unsafe { data.add(pos).read() },
            ),
        }
    }

    pub(crate) unsafe fn get(&self, idx: usize) -> *mut T {
        debug_assert!(idx != self.pos, "Read of element in a hole");
        unsafe { self.data.add(idx) }
    }

    pub(crate) unsafe fn move_to(&mut self, new_pos: usize) {
        debug_assert!(new_pos != self.pos, "Moved Hole to the same position");
        unsafe {
            ptr::copy_nonoverlapping(self.data.add(new_pos), self.data.add(self.pos), 1);
            self.pos = new_pos;
        }
    }
}

impl<T> Drop for Hole<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.data
                .add(self.pos)
                .write(ManuallyDrop::take(&mut self.elt));
        }
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use alloc::rc::Rc;
    use core::{array, cell::Cell};

    use super::Hole;
    struct Droppy {
        value: usize,
        drop_count: Rc<Cell<usize>>,
    }

    impl Drop for Droppy {
        fn drop(&mut self) {
            self.drop_count.set(self.drop_count.get() + 1);
        }
    }

    #[test]
    fn hole() {
        const ITEMS: usize = 4;
        let drops: [_; ITEMS] = array::from_fn(|_| Rc::new(Cell::new(0)));
        let mut items: [_; ITEMS] = array::from_fn(|idx| Droppy {
            value: idx,
            drop_count: Rc::clone(&drops[idx]),
        });

        let ptr = items.as_mut_ptr();
        let mut hole = unsafe { Hole::new(ptr, 1) };
        unsafe {
            hole.move_to(3);
            hole.move_to(0);
        };
        drop(hole);
        // After drop of hole, the array is reordered as expected
        assert!(items.iter().map(|it| it.value).eq([1, 3, 2, 0]));
        // and no elemenets were dropped
        assert!(drops.iter().all(|drop_count| drop_count.get() == 0));
        drop(items);
        assert!(drops.iter().all(|drop_count| drop_count.get() == 1));
    }
}
