use core::{cmp::Ordering, marker::PhantomData};

#[cfg(feature = "vec_storage")]
use alloc::vec::Vec;

#[cfg(feature = "stackvec_storage")]
use stackvector::{Array, StackVec};

use super::MergedIter;

/// A builder for creating a merging iterator.
///
/// The `Merged` type provides an interface for configuring and creating [`MergedIter`] iterators.
/// It allows you to specify:
/// - Whether tie-breaking should be stable or arbitrary
/// - Custom comparison functions
/// - Storage backends (heap-allocated `Vec` or stack-allocated arrays)
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
/// let mut merged = Merged::new([iter1, iter2])
///     .arbitrary_tie_breaking() // for better performance
///     .build();
/// let result = merged.into_vec();
///
/// assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
/// # }
/// ```
///
/// [`MergedIter`]: crate::MergedIter
#[derive(Debug)]
pub struct Merged<const STABLE_TIE_BREAKING: bool, Cmp, Storage, IterIter> {
    pub(crate) iters: IterIter,
    pub(crate) cmp: Cmp,
    pub(crate) _p: PhantomData<Storage>,
}

impl<IterIter, Iter> Merged<true, (), (), IterIter>
where
    IterIter: IntoIterator<Item = Iter>,
    Iter: IntoIterator,
{
    /// Creates a new [`Merged`] for merging iterators.
    ///
    /// This is the entry point for creating a merge iterator. By default, it uses:
    /// - Stable tie-breaking (items from earlier iterators are yielded first when equal)
    /// - Standard ordering comparison `Ord::cmp` (if items implement `Ord`)
    /// - `Vec` storage for the internal iterator state
    ///
    /// # Arguments
    ///
    /// * `iters` - A collection of iterators to merge. This can be any type that implements
    ///   `IntoIterator<Item = Iter>` where `Iter` is the iterator type to merge.
    ///
    /// # Returns
    ///
    /// Returns a new [`Merged`].
    ///
    /// # Note
    ///
    /// This method creates a builder with stable tie-breaking by default. If you need
    /// arbitrary tie-breaking for better performance, use [`arbitrary_tie_breaking()`] after
    /// calling this method.
    ///
    /// [`arbitrary_tie_breaking()`]: Merged::arbitrary_tie_breaking
    #[cfg(feature = "vec_storage")]
    #[allow(clippy::type_complexity)]
    pub const fn new(iters: IterIter) -> Merged<true, (), Vec<(Iter::Item, Iter::IntoIter)>, IterIter> {
        Merged {
            iters,
            cmp: (),
            _p: PhantomData,
        }
    }

    /// Creates a new [`Merged`] for merging iterators that uses stackvec storage.
    ///
    /// See [`new`](crate::Merged::new) for the details.
    ///
    /// # Generic parameters
    ///
    /// * `Arr` - The array type that will be used for storage. This should be sized to
    ///   accommodate the maximum number of iterators you expect to merge, otherwise [building]
    ///   the iterator would panic.
    ///
    /// # Examples
    /// ```
    /// use iter_merge::Merged;
    /// # use core::array;
    ///
    /// let iter1 = [1, 3, 5];
    /// let iter2 = [2, 4, 6];
    ///
    /// let mut merged = Merged::new_stackvec::<2>([iter1, iter2])
    ///     .build();
    /// let result: [i32; 6] = array::from_fn(|_| merged.next().unwrap());
    ///
    /// assert_eq!(result, [1, 2, 3, 4, 5, 6]);
    /// ```
    ///
    /// [building]: crate::Merged::build
    #[cfg(feature = "stackvec_storage")]
    #[allow(clippy::type_complexity)]
    pub const fn new_stackvec<const N: usize>(iters: IterIter) -> Merged<true, (), StackVec<[(Iter::Item, Iter::IntoIter); N]>, IterIter> where
        [(Iter::Item, Iter::IntoIter); N]: Array<Item = (Iter::Item, Iter::IntoIter)>
    {
        Merged {
            iters,
            cmp: (),
            _p: PhantomData,
        }
    }
}

impl<const STABLE_TIE_BREAKING: bool, Cmp, Storage, IterIter, Iter>
    Merged<STABLE_TIE_BREAKING, Cmp, Storage, IterIter>
where
    IterIter: IntoIterator<Item = Iter>,
    Iter: IntoIterator,
{

    /// Sets a custom comparison function for merging.
    ///
    /// By default, the merge uses the `Ord::cmp` (if items implement `Ord`). This method allows you
    /// to specify a custom comparison function to control the order in which items are merged.
    ///
    /// # Arguments
    ///
    /// * `cmp` - A function or closure that compares two items and returns an [`Ordering`].
    ///   The function should be consistent and transitive for proper merge behavior.
    ///
    /// # Returns
    ///
    /// Returns a new [`Merged`] with the provided comparison function.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    ///
    /// let iter1 = vec![5, 3, 1];
    /// let iter2 = vec![6, 4, 2];
    /// let mut merged = Merged::new([iter1, iter2])
    ///     .with_cmp(|a, b| b.cmp(a)) // reverse order
    ///     .build();
    /// let result = merged.into_vec();
    /// assert_eq!(result, vec![6, 5, 4, 3, 2, 1]);
    /// # }
    /// ```
    ///
    /// ```
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    ///
    /// // Custom comparison for case-insensitive string sorting
    /// let iter1 = vec!["Apple", "banana"];
    /// let iter2 = vec!["Cherry", "date"];
    /// let mut merged = Merged::new([iter1, iter2])
    ///     .with_cmp(|a, b| a.to_lowercase().cmp(&b.to_lowercase()))
    ///     .build();
    /// let result = merged.into_vec();
    /// assert_eq!(result, vec!["Apple", "banana", "Cherry", "date"]);
    /// # }
    /// ```
    ///
    /// [`Ordering`]: std::cmp::Ordering
    /// [`Merged`]: crate::Merged
    pub fn with_cmp<F: Fn(&Iter::Item, &Iter::Item) -> Ordering>(
        self, cmp: F,
    ) -> Merged<STABLE_TIE_BREAKING, F, Storage, IterIter> {
        let Self { iters, .. } = self;
        Merged {
            iters,
            cmp,
            _p: PhantomData,
        }
    }
}



impl<Cmp, Storage, IterIter> Merged<true, Cmp, Storage, IterIter> {
    /// Enables arbitrary tie-breaking for the merge operation.
    ///
    /// When items from different iterators are equal according to the comparison function,
    /// arbitrary tie-breaking allows the merge to yield them in any order. This provides
    /// better performance but does not guarantee a predictable ordering for equal items.
    ///
    /// # Returns
    ///
    /// Returns a new [`Merged`] with arbitrary tie-breaking enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "vec_storage")]
    /// # {
    /// use iter_merge::Merged;
    /// use std::cmp::Ordering;
    ///
    /// let iter1 = vec![(0, 0)];
    /// let iter2 = vec![(0, 1)];
    ///
    /// let mut merged = Merged::new([iter1, iter2])
    ///     // We have 2 distinct elements that compare equal
    ///     .with_cmp(|a, b| a.0.cmp(&b.0))
    ///     .arbitrary_tie_breaking()
    ///     .build();
    /// let result = merged.into_vec();
    ///
    /// assert_eq!(result.len(), 2);
    /// // The order of elements is undefined because of `arbitrary_tie_breaking`
    /// assert!(result.contains(&(0, 0)));
    /// assert!(result.contains(&(0, 1)));
    /// # }
    /// ```
    pub fn arbitrary_tie_breaking(self) -> Merged<false, Cmp, Storage, IterIter> {
        let Self { iters, cmp, .. } = self;
        Merged {
            iters,
            cmp,
            _p: PhantomData,
        }
    }
}


impl<Cmp, Storage, IterIter> Merged<false, Cmp, Storage, IterIter> {
    /// Enables stable tie-breaking for the merge operation.
    ///
    /// When items from different iterators are equal according to the comparison function,
    /// stable tie-breaking ensures that items from earlier iterators are yielded first.
    /// This provides a predictable ordering but comes with a slight performance cost.
    ///
    /// This method is provided just for completeness, since stable tie-breaking is the default.
    ///
    /// # Returns
    ///
    /// Returns a new [`Merged`] with stable tie-breaking enabled.
    pub fn stable_tie_breaking(self) -> Merged<true, Cmp, Storage, IterIter> {
        let Self { iters, cmp, .. } = self;
        Merged {
            iters,
            cmp,
            _p: PhantomData,
        }
    }
}

#[expect(private_bounds)]
impl<const STABLE_TIE_BREAKING: bool, Storage, IterIter, Iter>
    Merged<STABLE_TIE_BREAKING, (), Storage, IterIter>
where
    IterIter: IntoIterator<Item = Iter>,
    Iter: IntoIterator,
    Storage: crate::storage::Storage<Item = (Iter::Item, Iter::IntoIter)>,
    Iter::Item: Ord,
{
    /// Builds a merged iterator using the default ordering ([`Ord`]) for the item type.
    ///
    /// [`Ord`]: core::cmp::Ord
    pub fn build(
        self,
    ) -> MergedIter<STABLE_TIE_BREAKING, Storage, impl Fn(&Iter::Item, &Iter::Item) -> Ordering> {
        let Self { iters, .. } = self;
        Merged {
            iters,
            cmp: Ord::cmp,
            _p: PhantomData,
        }
        .build()
    }
}

#[expect(private_bounds)]
impl<const STABLE_TIE_BREAKING: bool, Cmp, Storage, IterIter, Iter>
    Merged<STABLE_TIE_BREAKING, Cmp, Storage, IterIter>
where
    IterIter: IntoIterator<Item = Iter>,
    Iter: IntoIterator,
    Storage: crate::storage::Storage<Item = (Iter::Item, Iter::IntoIter)>,
    Cmp: Fn(&Iter::Item, &Iter::Item) -> Ordering,
{
    /// Builds a merged iterator using the provided comparison function.
    ///
    /// This method constructs a [`MergedIter`] iterator that merges all input iterators,
    /// comparing items using the custom comparison function provided to the builder.
    ///
    /// [`MergedIter`]: crate::MergedIter
    pub fn build(self) -> MergedIter<STABLE_TIE_BREAKING, Storage, Cmp> {
        let Self { iters, cmp, .. } = self;
        let mut merged = MergedIter::new(cmp);
        merged.add_iters(iters);
        merged
    }
}
