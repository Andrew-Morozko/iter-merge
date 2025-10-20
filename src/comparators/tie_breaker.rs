//! Comparators that compare items by the addresses of the items.
//!
//! In context of this library it's suggested to use them as tie-breakers
//! that decide which item to yield in case the main comparator returns [`Ordering::Equal`].
//! Addresses of items reflect the order in which they were inserted in storage (
//! at least for provided [`VecStorage`](crate::VecStorage) and
//! [`ArrayStorage`](crate::ArrayStorage)), thus the naming of these comparisons.
//!
//! [`Unspecified`] tie-breaker always returns [`Ordering::Equal`]. This makes the
//! [`MergeIter`](crate::MergeIter) a bit faster, but the order of polled iterators with equal
//! items is unstable (may change if the initial iterator list is modified in any way)

use core::cmp::Ordering;

use crate::{comparators::Comparator, internal::pointers::addr_from_ref};

/// If two items are equal the item from earlier-inserted iterator will be yielded first
#[derive(Debug, Clone, Copy)]
pub struct InsertionOrder;

impl<T> Comparator<T> for InsertionOrder {
    #[inline]
    fn compare(&self, a: &T, b: &T) -> Ordering {
        addr_from_ref(a).cmp(&addr_from_ref(b))
    }
}

/// If two items are equal the item from later-inserted iterator will be yielded first
#[derive(Debug, Clone, Copy)]
pub struct ReverseInsertionOrder;

impl<T> Comparator<T> for ReverseInsertionOrder {
    #[inline]
    fn compare(&self, a: &T, b: &T) -> Ordering {
        addr_from_ref(b).cmp(&addr_from_ref(a))
    }
}

/// If two items are equal they are yielded in unspecified order. This improves
/// the performance a bit.
#[derive(Debug, Clone, Copy)]
pub struct Unspecified; //TODO: measure

impl<T> Comparator<T> for Unspecified {
    #[inline]
    fn compare(&self, _a: &T, _b: &T) -> Ordering {
        Ordering::Equal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tie_breaker() {
        let arr = [1, 2];
        assert!(InsertionOrder.compare(&arr[0], &arr[1]).is_lt());
        assert!(InsertionOrder.compare(&arr[1], &arr[0]).is_gt());
        assert!(ReverseInsertionOrder.compare(&arr[0], &arr[1]).is_gt());
        assert!(ReverseInsertionOrder.compare(&arr[1], &arr[0]).is_lt());
        assert!(Unspecified.compare(&arr[0], &arr[1]).is_eq());
        assert!(Unspecified.compare(&arr[1], &arr[0]).is_eq());
    }
}
