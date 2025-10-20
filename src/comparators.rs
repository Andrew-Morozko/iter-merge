//! Defines comparators for [`MergeIter`](crate::MergeIter)
//!
//! Users of this crate may implement [`Comparator`] trait to create a custom comparator
//! or use [`ByOrd`] in builder functions [`{min|max}_by`](crate::merge_iter::Builder::min_by)
//! to compare items using [`Ord`] trait.
//!
//! Comparators can be chained by using [`Chain::new`].
//!
//! The rest of the structures here have no public constructors, they are constructed by various
//! [`Builder`](crate::merge_iter::Builder) methods.

use core::cmp::Ordering;

pub mod tie_breaker;

/// Trait used to compare elements of [`MergeIter`](crate::MergeIter)
///
/// Implementations should produce a consistent total ordering, see [`Ord`]
/// documentation for details.
///
/// Producing non-total or inconsistent ordering may result in incorrect behavior
/// (i.e. items are yielded in a wrong order) but will not result in UB.
pub trait Comparator<T>: Sized {
    /// Compares two elements and returns an [`Ordering`]
    fn compare<'a>(&self, a: &'a T, b: &'a T) -> Ordering;
}

impl<T, C> Comparator<T> for &C
where
    C: Comparator<T>,
{
    #[inline]
    fn compare<'a>(&self, a: &'a T, b: &'a T) -> Ordering {
        C::compare(self, a, b)
    }
}

/// Wrapper that reverses a comparator.
///
/// Our internal data stuctures are all min-first, so to get
/// max-first we're just inverting the order of operands passed to
/// comparators.
#[derive(Debug, Clone)]
pub struct MaxFirst<C>(pub(crate) C);

impl<C> MaxFirst<C> {
    #[inline]
    #[doc(hidden)]
    pub const fn new<T>(comparator: C) -> Self
    where
        C: Comparator<T>,
    {
        Self(comparator)
    }
}

impl<T, C> Comparator<T> for MaxFirst<C>
where
    C: Comparator<T>,
{
    #[inline]
    fn compare(&self, a: &T, b: &T) -> Ordering {
        self.0.compare(b, a)
    }
}

/// Calls the second comparator if the first one returns [`Ordering::Equal`].
#[derive(Debug, Clone)]
pub struct Chain<C1, C2> {
    first: C1,
    next: C2,
}

impl<C1, C2> Chain<C1, C2> {
    /// If the first comparator returns [`Ordering::Equal`] - compare
    /// elements using `next`.
    ///
    /// Similar to [`Ordering::then_with`]
    #[inline]
    pub const fn new<T>(first: C1, next: C2) -> Self
    where
        C1: Comparator<T>,
        C2: Comparator<T>,
        T:,
    {
        Self { first, next }
    }
}

impl<T, C1, C2> Comparator<T> for Chain<C1, C2>
where
    C1: Comparator<T>,
    C2: Comparator<T>,
    T:,
{
    #[inline]
    fn compare<'a>(&self, a: &'a T, b: &'a T) -> Ordering {
        match self.first.compare(a, b) {
            Ordering::Equal => self.next.compare(a, b),
            other => other,
        }
    }
}

/// Comparator that uses [`Ord`] to compare items, default for the [`MergeIter`](crate::MergeIter).
///
/// # Example
/// Max-first merge:
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// use iter_merge::{VecStorage, comparators::ByOrd};
/// let res = VecStorage::from_iter([vec![3, 2], vec![4, 1]])
///     .into_builder()
///     .max_by(ByOrd)
///     .build()
///     .into_vec();
/// assert_eq!(res, vec![4, 3, 2, 1]);
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ByOrd;

impl<T: Ord> Comparator<T> for ByOrd {
    #[inline]
    fn compare(&self, a: &T, b: &T) -> Ordering {
        Ord::cmp(a, b)
    }
}

/// Comparator that uses a function to compare items
///
/// Construct via [`{min|max}_by_func`](crate::merge_iter::Builder::min_by_func)
#[derive(Debug, Clone)]
pub struct ByFunc<F>(pub(crate) F);

impl<T, F> Comparator<T> for ByFunc<F>
where
    F: Fn(&T, &T) -> Ordering,
    T:,
{
    // Leaving decision to inline this to the compiler because F can be long
    fn compare(&self, a: &T, b: &T) -> Ordering {
        self.0(a, b)
    }
}

/// Comparator that uses a key to compare items
///
/// Construct via [`{min|max}_by_key`](crate::merge_iter::Builder::min_by_key)
#[derive(Debug, Clone)]
pub struct ByKey<F>(pub(crate) F);

impl<T, F, K> Comparator<T> for ByKey<F>
where
    F: Fn(&T) -> K,
    K: Ord,
    T:,
{
    // Leaving decision to inline this to the compiler because F can be long
    fn compare(&self, a: &T, b: &T) -> Ordering {
        self.0(a).cmp(&self.0(b))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn comparators() {
        let [a, b] = [1_i32, 2];
        assert!(Comparator::compare(&ByOrd, &a, &b).is_lt());
        assert!(Comparator::compare(&MaxFirst(ByOrd), &a, &b).is_gt());
        assert!(
            Comparator::compare(
                &ByFunc(|a: &i32, b: &i32| {
                    assert!(*a == 1);
                    assert!(*b == 2);
                    Ordering::Equal
                }),
                &a,
                &b
            )
            .is_eq()
        );
        assert!(
            Comparator::compare(
                &MaxFirst(ByFunc(|a: &i32, b: &i32| {
                    assert!(*a == 2);
                    assert!(*b == 1);
                    Ordering::Equal
                })),
                &a,
                &b
            )
            .is_eq()
        );

        assert!(
            Comparator::compare(
                &ByKey(|v: &i32| {
                    assert!(*v == 1 || *v == 2);
                    0
                }),
                &a,
                &b
            )
            .is_eq()
        );
    }
}
