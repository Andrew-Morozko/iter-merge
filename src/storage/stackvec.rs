use stackvector::{Array, StackVec};

use super::{SealedToken, Storage};

impl<T, A: Array<Item = T>> Storage for StackVec<A> {
    type Item = T;

    #[inline(always)]
    fn new(_: SealedToken) -> Self {
        Self::new()
    }

    #[inline(always)]
    fn push(&mut self, value: Self::Item, _: SealedToken) {
        Self::push(self, value);
    }
    #[inline(always)]
    fn remove(&mut self, index: usize, _: SealedToken) -> Self::Item {
        select! {
            ();
            unsafe { core::hint::assert_unchecked(index < self.len()) }
        }
        Self::remove(self, index)
    }
    #[inline(always)]
    fn swap_remove(&mut self, index: usize, _: SealedToken) -> Self::Item {
        select! {
            ();
            unsafe { core::hint::assert_unchecked(index < self.len()) }
        }
        Self::swap_remove(self, index)
    }
    #[inline(always)]
    fn len(&self, _: SealedToken) -> usize {
        Self::len(self)
    }
    #[inline(always)]
    fn get(&self, index: usize, _: SealedToken) -> &Self::Item {
        select! {
            &self[index];
            unsafe { self.get_unchecked(index) }
        }
    }

    #[inline(always)]
    fn get_mut(&mut self, index: usize, _: SealedToken) -> &mut Self::Item {
        select! {
            &mut self[index];
            unsafe { self.get_unchecked_mut(index) }
        }
    }
    #[inline(always)]
    fn reserve_for<I: Iterator>(&mut self, _iter: &I, _: SealedToken) {
        // StackVec is fixed capacity, so we don't need to reserve
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let stack_vec: StackVec<[i32; 10]> = Storage::new(SealedToken); // capacity parameter is ignored
        assert_eq!(stack_vec.len(), 0);
        assert_eq!(stack_vec.capacity(), 10);
    }

    #[test]
    fn test_reserve_for() {
        let iter = 0..5;
        let mut stack_vec: StackVec<[i32; 10]> = Storage::new(SealedToken);
        Storage::reserve_for(&mut stack_vec, &iter, SealedToken);
        assert_eq!(stack_vec.len(), 0);
        assert_eq!(stack_vec.capacity(), 10);
    }

    #[test]
    fn test_push() {
        let mut stack_vec: StackVec<[i32; 10]> = Storage::new(SealedToken);
        stack_vec.extend([1, 2, 3]);
        Storage::push(&mut stack_vec, 42, SealedToken);
        assert_eq!(stack_vec.as_slice(), &[1, 2, 3, 42]);
    }

    #[test]
    fn test_remove() {
        let mut stack_vec: StackVec<[i32; 10]> = Storage::new(SealedToken);
        stack_vec.extend([1, 2, 3, 4, 5]);
        let removed = Storage::remove(&mut stack_vec, 2, SealedToken);
        assert_eq!(removed, 3);
        assert_eq!(stack_vec.as_slice(), &[1, 2, 4, 5]);
    }

    #[test]
    fn test_swap_remove() {
        let mut stack_vec: StackVec<[i32; 10]> = Storage::new(SealedToken);
        stack_vec.extend([1, 2, 3, 4, 5]);
        let removed = Storage::swap_remove(&mut stack_vec, 1, SealedToken);
        assert_eq!(removed, 2);
        assert_eq!(stack_vec.as_slice(), &[1, 5, 3, 4]); // Last element moved to position 1
    }

    #[test]
    fn test_len() {
        let mut stack_vec: StackVec<[i32; 10]> = Storage::new(SealedToken);
        stack_vec.extend([1, 2, 3, 4, 5]);
        assert_eq!(Storage::len(&stack_vec, SealedToken), 5);
    }

    #[test]
    fn test_get() {
        let mut stack_vec: StackVec<[i32; 10]> = Storage::new(SealedToken);
        stack_vec.extend([42, 100, 200]);
        assert_eq!(*Storage::get(&stack_vec, 0, SealedToken), 42);
        assert_eq!(*Storage::get(&stack_vec, 1, SealedToken), 100);
        assert_eq!(*Storage::get(&stack_vec, 2, SealedToken), 200);
    }

    #[test]
    fn test_get_mut() {
        let mut stack_vec: StackVec<[i32; 10]> = Storage::new(SealedToken);
        stack_vec.extend([42, 100, 200]);
        *Storage::get_mut(&mut stack_vec, 0, SealedToken) = 300;
        *Storage::get_mut(&mut stack_vec, 2, SealedToken) = 400;
        assert_eq!(stack_vec.as_slice(), &[300, 100, 400]);
    }
}
