//! A high-performance iterator merging library.
//!
//! This crate provides efficient merging of multiple iterators into a single iterator.
//! It supports both stable and arbitrary tie-breaking, custom comparison functions, and different
//! storage backends for optimal performance in various scenarios.
//!
//! # Quick Start
//!
//! ```
//! # #[cfg(feature = "vec_storage")]
//! # {
//! use iter_merge::Merged;
//!
//! let vec1 = vec![1, 3, 5];
//! let vec2 = vec![2, 4, 6];
//!
//! let mut merged = Merged::new([vec1.iter().copied(), vec2.iter().copied()]).build();
//! let result = merged.into_vec();
//!
//! assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
//! # }
//! ```
//!
//! Note that only the next item in each iterator is considered.
//! If the input iterators are not sorted, the result won't be sorted either:
//!
//! ```
//! # #[cfg(feature = "vec_storage")]
//! # {
//! use iter_merge::Merged;
//!
//! let result = Merged::new([vec![2, 1, 5], vec![4, 3, 6]])
//!     .build()
//!     .into_vec();
//!
//! assert_eq!(result, vec![2, 1, 4, 3, 5, 6]);
//! # }
//! ```
//!
//! # Performance
//!
//! This library is designed for high performance, especially when merging a small number of iterators.
//! It scales as `O(iterator_count² + item_count)`, while [`itertools::kmerge`] scales as
//! `O(iterator_count + item_count)` with a higher constant term.
//!
//! * Up to ~123 random iterators it's 2x faster than kmerge, at ~355 iterators it's 1.5x faster,
//!   the break-even point is ~682 iterators, and at ~1363 iterators it's 2x slower.
//! * If 1% of data is out of order (randomly swapped) the break-even point is ~1073 iterators
//! * If the data is fully sorted the break-even point is ~2643 iterators
//! * `arbitrary_tie_breaking()` is 1.23x faster than (default) `stable_tie_breaking`
//! * Default implementation (with unsafe code) is 1.49x faster than fully safe implementation
//!   (when `forbid_unsafe` feature is active)
//!
//! <details>
//!   <summary>Exact benchmark parameters</summary>
//!   <ul>
//!     <li>
//!         Benchmarks were run on a fresh Ubuntu install on dedicated
//!         Intel E-1270v3 (4 cores; 3.5GHz) server with maximal optimizations
//!         (opt-level=3, lto=true, codegen-units=1, target-cpu=native)
//!     </li>
//!     <li><code>item_count</code> is <code>1 044 480</code> for comparisons with kmerge</li>
//!     <li><code>arbitrary_tie_breaking()</code> is enabled, since this matches the kmerge</li>
//!     <li>Exact iterator counts were interpolated</li>
//!     <li>
//!         <code>arbitrary_tie_breaking()</code> and <code>forbid_unsafe</code> performance
//!         was evaluated with <code>1 048 576</code> items and <code>64</code> iterators
//!     </li>
//!   </ul>
//! </details>
//!
//! Unlike `kmerge`, which eagerly collects items into a min-heap, this library pulls items from
//! input iterators *lazily* — items are only fetched as needed, and only the iterator containing
//! the smallest item is advanced at each `.next()` call.
//!
//! For detailed performance numbers and scenarios, run the included benchmarks in the `benches/`
//! directory.
//!
//! [`itertools::kmerge`]: https://docs.rs/itertools/0.14.0/itertools/trait.Itertools.html#method.kmerge
//!
//! # Crate Features
//!
//! - `vec_storage`: Enables heap-allocated storage with `Vec` (enabled by default)
//! - `stackvec_storage`: Enables stack-allocated storage with `with_stackvec_storage()`
//! - `forbid_unsafe`: Prevents the use of unsafe code throughout the crate
//!
//! See the documentation for individual types and methods for more detailed examples.

#![no_std]
#![cfg_attr(feature = "forbid_unsafe", forbid(unsafe_code))]
#![cfg_attr(fuzzing, feature(coverage_attribute))]

#[cfg(feature = "vec_storage")]
extern crate alloc;

mod builder;
mod iter;
mod storage;

pub use builder::Merged;
pub use iter::MergedIter;

#[doc(hidden)]
#[cfg_attr(feature="vec_storage", doc = include_str!("../README.md"))]
struct _ReadmeTest;
