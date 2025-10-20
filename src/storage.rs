//! Storage backends

pub(crate) mod array;
pub use array::*;
#[cfg(feature = "alloc")]
pub(crate) mod vec;
use core::fmt::Debug;

#[cfg(feature = "alloc")]
pub use vec::*;

use crate::{
    comparators::{ByOrd, tie_breaker},
    internal::{PeekIter, StorageOps},
    merge_iter::DefaultBuilder,
};

/// Marker trait for [`MergeIter`](crate::MergeIter) storage.
pub trait Storage: StorageOps + Sized {
    /// Create a new builder with default parameters for this storage
    #[inline]
    fn into_builder(self) -> DefaultBuilder<Self> {
        DefaultBuilder::new(self, ByOrd, tie_breaker::InsertionOrder)
    }
}

impl<S: StorageOps + Sized> Storage for S {}

struct DebugFormatter<'a, S>(&'a S);

impl<S> Debug for DebugFormatter<'_, S>
where
    S: Storage,
    PeekIter<S::IT>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_list();
        self.0.map_items(|it| {
            d.entry(it);
        });
        d.finish()
    }
}

/// Utility that returns a [`Debug`] formatter of the items in [`Storage`]
///
/// Items are shown using standard [`debug_list`](core::fmt::Formatter::debug_list) format.
/// See [this `Debug` impl](crate::storage::InternalArrayStorage::fmt) for an example.
#[inline]
pub fn debug_formatter<S>(storage: &'_ S) -> impl Debug + '_
where
    S: Storage,
    PeekIter<S::IT>: Debug,
{
    DebugFormatter(storage)
}
