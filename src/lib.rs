//! A high-performance iterator merging library.
//!
//! This crate provides [`MergeIter`] and a builder API to merge items from many iterators
//! according to a comparator. By default, it performs a min-merge by [`Ord`], breaking ties
//! by insertion order. It's `no_std`, with `Vec`-requiring functions behind the `alloc` feature.
//!
//! # Quick start
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use iter_merge::merge;
//!
//! let a = vec![1, 3, 5];
//! let b = vec![2, 4, 6];
//! let merged = merge([a, b]).into_vec();
//! assert_eq!(merged, vec![1, 2, 3, 4, 5, 6]);
//! # }
//! ```
//!
//! Note that only the next item in each iterator is considered.
//! If the input iterators are not sorted, the result won't be sorted either:
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use iter_merge::merge;
//!
//! let merged = merge([vec![2, 1, 5], vec![4, 3, 6]]).into_vec();
//! assert_eq!(merged, vec![2, 1, 4, 3, 5, 6]);
//! # }
//! ```
//!
//! ## Array storage
//!
//! ```
//! use core::pin::pin;
//!
//! use iter_merge::{ArrayStorage, MergeIter};
//!
//! // First create a storage with some fixed capacity
//! let mut storage = ArrayStorage::<2, _>::from_iter([[1, 3, 5]]);
//! // You can modify it by adding iterators of the same type
//! storage.push([2, 4, 6]);
//!
//! // In order to construct a MergeIter you need to pin that storage.
//! // You won't be able to modify it once you've pinned it.
//! let storage = pin!(storage);
//! let mut merge = storage.build();
//! assert!(merge.eq([1, 2, 3, 4, 5, 6]));
//! ```
//!
//! # Custom comparator
//!
//! Use the builder to specify custom ordering (min/max by comparison function, by key, or by Ord).
//! Implement a custom [`comparator`](crate::comparators::Comparator) for even more control.
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use iter_merge::{MergeIter, VecStorage};
//!
//! // Merge by descending absolute value
//! let res = VecStorage::from_iter([vec![-3_i32, -1], vec![2, -2]])
//!     .into_builder()
//!     .max_by_key(|&x| x.abs())
//!     .build()
//!     .into_vec();
//! assert_eq!(res, vec![-3, 2, -2, -1]);
//! # }
//! ```
//!
//! # Peeking and conditional consumption
//! [`MergeIter`] provides the same methods as [`iter::Peekable`](core::iter::Peekable):
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use iter_merge::merge;
//!
//! let mut it = merge([vec![1, 1, 2], vec![1, 3]]);
//! assert_eq!(it.peek(), Some(&1));
//! // consume all 1s
//! while let Some(1) = it.next_if_eq(&1) {}
//! assert_eq!(it.next(), Some(2));
//! # }
//! ```
//!
//! # Performance
//!
//! It's 1.45-1.65x faster than [`itertools::kmerge`] in my benchmarks and scales as
//! `O(item_count + log₂(iterator_count))`
//!
//! <details>
//!   <summary>Benchmark details</summary>
//!   <p>
//!     Benchmarks were run on a fresh Ubuntu install on dedicated
//!     Intel E-1270v3 (4 cores; 3.5GHz) server with maximal optimizations
//!     (opt-level=3, lto=true, codegen-units=1, target-cpu=native)
//!   </p>
//!   <p>
//!     Iterators were over random u64's, and iterator itself had a size of 32 bytes long
//!     (for reference Vec::into_iter has the same size). Expect bigger performance
//!     improvements compared with <code>itertools::kmerge</code> when merging larger
//!     iterators and/or iterators over large values, since the key difference between these
//!     libraries is the use of pointer indirection in our heap, so we don't need to move
//!     the pair of <code>(peeked_item, iterator)</code>, only pointers to them.
//!   </p>
//! </details>
//!
//! For detailed performance numbers in various scenarios, run the included benchmarks in the
//! `benches/` directory.
//!
//! [`itertools::kmerge`]: https://docs.rs/itertools/0.14.0/itertools/trait.Itertools.html#method.kmerge
//!
//! # Crate Features
//! - `alloc`: Enables heap-allocated storage with [`VecStorage`] and methods like
//!   [`MergeIter::into_vec`]
#![no_std]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(not(feature = "alloc"), allow(unused))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod comparators;
pub mod merge_iter;
pub mod storage;

pub use merge_iter::MergeIter;
pub use storage::ArrayStorage;
#[cfg(feature = "alloc")]
pub use storage::VecStorage;

#[cfg(feature = "alloc")]
mod convenience;
#[cfg(feature = "alloc")]
pub use convenience::*;

pub mod internal;

#[cfg(any(fuzzing, test))]
#[doc(hidden)]
pub mod tests;


#[doc(hidden)]
#[cfg_attr(feature="alloc", doc = include_str!("../README.md"))]
struct _ReadmeTest;