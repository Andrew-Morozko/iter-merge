use alloc::vec::Vec;

use super::Storage;

impl<T> Storage for Vec<T> {
    type Item = T;
    #[inline(always)]
    fn new() -> Self {
        Self::new()
    }
    #[inline(always)]
    fn push(&mut self, value: Self::Item) {
        Self::push(self, value);
    }
    #[inline(always)]
    fn remove(&mut self, index: usize) -> Self::Item {
        select!{
            ();
            unsafe { core::hint::assert_unchecked(index < self.len()) }
        }
        Self::remove(self, index)
    }
    #[inline(always)]
    fn swap_remove(&mut self, index: usize) -> Self::Item {
        select!{
            ();
            unsafe { core::hint::assert_unchecked(index < self.len()) }
        }
        Self::swap_remove(self, index)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        Self::len(self)
    }
    #[inline(always)]
    fn get(&self, index: usize) -> &Self::Item {
        select!{
            &self[index];
            unsafe { self.get_unchecked(index) }
        }
    }

    #[inline(always)]
    fn get_mut(&mut self, index: usize) -> &mut Self::Item {
        select!{
            &mut self[index];
            unsafe { self.get_unchecked_mut(index) }
        }
    }
    #[inline(always)]
    fn reserve_for<I: Iterator>(&mut self, iter: &I) {
        Self::reserve(self, iter.size_hint().0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_new() {
        let vec: Vec<i32> = Storage::new();
        assert_eq!(vec.capacity(), 0);
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_reserve_for() {
        let iter = 0..5;
        let mut stack_vec: Vec<i32> = Storage::new();
        assert_eq!(stack_vec.capacity(), 0);
        Storage::reserve_for(&mut stack_vec, &iter);
        assert_eq!(stack_vec.len(), 0);
        assert!(stack_vec.capacity() >= 5);
    }

    #[test]
    fn test_push() {
        let mut vec = vec![1, 2, 3];
        Storage::push(&mut vec, 42);
        assert_eq!(vec, vec![1, 2, 3, 42]);
    }

    #[test]
    fn test_remove() {
        let mut vec = vec![1, 2, 3, 4, 5];
        let removed = Storage::remove(&mut vec, 2);
        assert_eq!(removed, 3);
        assert_eq!(vec, vec![1, 2, 4, 5]);
    }

    #[test]
    fn test_swap_remove() {
        let mut vec = vec![1, 2, 3, 4, 5];
        let removed = Storage::swap_remove(&mut vec, 1);
        assert_eq!(removed, 2);
        assert_eq!(vec, vec![1, 5, 3, 4]); // Last element moved to position 1
    }

    #[test]
    fn test_len() {
        let vec = vec![1, 2, 3, 4, 5];
        assert_eq!(Storage::len(&vec), 5);
    }

    #[test]
    fn test_get() {
        let vec = vec![42, 100, 200];
        assert_eq!(*Storage::get(&vec, 0), 42);
        assert_eq!(*Storage::get(&vec, 1), 100);
        assert_eq!(*Storage::get(&vec, 2), 200);
    }

    #[test]
    fn test_get_mut() {
        let mut vec = vec![42, 100, 200];
        *Storage::get_mut(&mut vec, 0) = 300;
        *Storage::get_mut(&mut vec, 2) = 400;
        assert_eq!(vec, vec![300, 100, 400]);
    }
}
