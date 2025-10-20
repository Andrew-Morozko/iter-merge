#![allow(clippy::redundant_static_lifetimes, unused)]
#![cfg_attr(coverage_nightly, coverage(off))]

pub(crate) extern crate std;

use std::{
    mem::ManuallyDrop,
    panic::{UnwindSafe, catch_unwind, panic_any},
    pin::pin,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

mod data;
#[cfg(feature = "alloc")]
pub mod order;
pub use data::TestData;
use data::*;

// Make sure that the iterator is actually consumed
#[inline]
pub fn consume<T>(item: T) {
    drop(core::hint::black_box(item));
}

#[cfg(feature = "alloc")]
use crate::VecStorage;
use crate::{
    ArrayStorage,
    comparators::{ByOrd, MaxFirst, tie_breaker},
};

fn assert_panics_with<F>(msg: &'static str, f: F)
where
    F: FnOnce() + UnwindSafe,
{
    let err = catch_unwind(f).unwrap_err();
    if err.downcast_ref() != Some(&msg) {
        panic_any(err);
    }
}

fn correct_on_cmp_panic<TD: TestData>(iters: &TD) {
    const PANIC_MSG: &'static str = "PanicyCmp panic";
    static CMP_CALLS: AtomicUsize = AtomicUsize::new(0);

    let s = ArrayStorage::<MAX_TEST_VEC_LEN, _>::from_iter(iters.as_iters());
    pin!(s)
        .into_builder()
        .min_by_func(|a, b| {
            CMP_CALLS.fetch_add(1, SeqCst);
            a.cmp(b)
        })
        .build()
        .for_each(consume);
    let max_num_cmp = CMP_CALLS.swap(0, SeqCst);

    let panicky_cmp = |panic_at| {
        move |a: &TD::Item, b: &TD::Item| {
            if CMP_CALLS.fetch_add(1, SeqCst) == panic_at {
                CMP_CALLS.store(0, SeqCst);
                panic_any(PANIC_MSG);
            }
            a.cmp(b)
        }
    };

    for panic_at in 0..max_num_cmp {
        #[cfg(feature = "alloc")]
        {
            assert_panics_with(PANIC_MSG, || {
                VecStorage::from_iter(iters.as_iters())
                    .into_builder()
                    .min_by_func(panicky_cmp(panic_at))
                    .build()
                    .into_vec();
            });
            assert_panics_with(PANIC_MSG, || {
                VecStorage::from_iter(iters.as_iters())
                    .into_builder()
                    .min_by_func(panicky_cmp(panic_at))
                    .build()
                    .for_each(consume);
            });
            assert_panics_with(PANIC_MSG, || {
                let mut s = ArrayStorage::with_capacity::<MAX_TEST_VEC_LEN>();
                s.extend(iters.as_iters());
                let s = pin!(s);
                s.into_builder()
                    .min_by_func(panicky_cmp(panic_at))
                    .build()
                    .into_vec();
            });
        }
        assert_panics_with(PANIC_MSG, || {
            let mut s = ArrayStorage::with_capacity::<MAX_TEST_VEC_LEN>();
            s.extend(iters.as_iters());
            let s = pin!(s);
            s.into_builder()
                .min_by_func(panicky_cmp(panic_at))
                .build()
                .for_each(consume);
        });
    }
}

struct PanickyDropIter<IT> {
    iter: ManuallyDrop<IT>,
    panic_in_drop: bool,
}

impl PanickyDropIter<()> {
    const PANIC_MSG: &'static str = "PanickyIter panic";
}

impl<IT: Iterator> PanickyDropIter<IT> {
    fn new(iter: IT, panic_in_drop: bool) -> Self {
        Self {
            iter: ManuallyDrop::new(iter),
            panic_in_drop,
        }
    }
}

impl<IT: Iterator> Iterator for PanickyDropIter<IT> {
    type Item = IT::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<IT> Drop for PanickyDropIter<IT> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.iter);
        }
        if self.panic_in_drop {
            panic_any(PanickyDropIter::PANIC_MSG);
        }
    }
}

fn correct_on_iter_drop_panic(iters: &impl TestData) {
    let make_iter = |panic_at: usize| {
        iters
            .as_iters()
            .enumerate()
            .map(move |(n, iter)| PanickyDropIter::new(iter, n == panic_at))
    };

    for panic_at in 0..iters.length() {
        #[cfg(feature = "alloc")]
        {
            assert_panics_with(PanickyDropIter::PANIC_MSG, || {
                let _ = VecStorage::from_iter(make_iter(panic_at))
                    .build()
                    .into_vec();
            });
            assert_panics_with(PanickyDropIter::PANIC_MSG, || {
                VecStorage::from_iter(make_iter(panic_at))
                    .build()
                    .for_each(consume);
            });
            assert_panics_with(PanickyDropIter::PANIC_MSG, || {
                let mut s = ArrayStorage::with_capacity::<MAX_TEST_VEC_LEN>();
                s.extend(make_iter(panic_at));
                let s = pin!(s);
                let _ = s.build().into_vec();
            });
        }
        assert_panics_with(PanickyDropIter::PANIC_MSG, || {
            let mut s = ArrayStorage::with_capacity::<MAX_TEST_VEC_LEN>();
            s.extend(make_iter(panic_at));
            let s = pin!(s);
            s.build().for_each(consume);
        });
    }
}

fn correct_on_next_panic(iters: &impl TestData) {
    static NEXT_CALLS: AtomicUsize = AtomicUsize::new(0);
    const PANIC_MSG: &'static str = "PanickyNext panic";

    let make_iter = |panic_at: usize| {
        iters.as_iters().map(move |mut iter| {
            core::iter::from_fn(move || {
                if NEXT_CALLS.fetch_add(1, SeqCst) == panic_at {
                    NEXT_CALLS.store(0, SeqCst);
                    panic_any(PANIC_MSG)
                }
                iter.next()
            })
        })
    };

    let max_next_calls = iters.length() + iters.item_count();

    for panic_at in 0..max_next_calls {
        #[cfg(feature = "alloc")]
        {
            assert_panics_with(PANIC_MSG, || {
                let _ = VecStorage::from_iter(make_iter(panic_at))
                    .build()
                    .into_vec();
            });
            assert_panics_with(PANIC_MSG, || {
                VecStorage::from_iter(make_iter(panic_at))
                    .build()
                    .for_each(consume);
            });
            assert_panics_with(PANIC_MSG, || {
                let mut s = ArrayStorage::with_capacity::<MAX_TEST_VEC_LEN>();
                s.extend(make_iter(panic_at));
                let s = pin!(s);
                let _ = s.build().into_vec();
            });
        }
        assert_panics_with(PANIC_MSG, || {
            let mut s = ArrayStorage::with_capacity::<MAX_TEST_VEC_LEN>();
            s.extend(make_iter(panic_at));
            let s = pin!(s);
            s.build().for_each(consume);
        });
    }
}

#[cfg(feature = "alloc")]
fn correct_on_clone_mid_consumption(iters: &impl TestData) {
    for consumed in 0..=iters.item_count() {
        let result = VecStorage::from_iter(iters.as_iters()).build().into_vec();

        let mut orig = VecStorage::from_iter(iters.as_iters()).build();

        let mut res_iter = result.into_iter();
        for item in res_iter.by_ref().take(consumed) {
            assert_eq!(item, orig.next().unwrap());
        }

        let mut copy = orig.clone();
        for item in res_iter {
            assert_eq!(item, orig.next().unwrap());
            assert_eq!(item, copy.next().unwrap());
        }
        assert!(orig.next().is_none() && copy.next().is_none());
    }
}

// Under Miri tests both for UB and for memory leaks
#[test]
fn cmp_panic() {
    TEST_VECTORS.iter().for_each(correct_on_cmp_panic);
}

#[test]
fn iter_drop_panic() {
    TEST_VECTORS.iter().for_each(correct_on_iter_drop_panic);
}

#[test]
fn next_panic() {
    TEST_VECTORS.iter().for_each(correct_on_next_panic);
}

#[cfg(feature = "alloc")]
#[test]
fn clone() {
    TEST_VECTORS
        .iter()
        .for_each(correct_on_clone_mid_consumption);
}

#[cfg(feature = "alloc")]
#[test]
fn correct_order() {
    use self::order::assert_correct_order;
    for data in TEST_VECTORS {
        assert_correct_order(data, ByOrd, tie_breaker::InsertionOrder);
        assert_correct_order(data, ByOrd, tie_breaker::ReverseInsertionOrder);
        assert_correct_order(data, ByOrd, tie_breaker::Unspecified);
        assert_correct_order(
            data,
            MaxFirst::new::<TestItemType>(ByOrd),
            tie_breaker::InsertionOrder,
        );
        assert_correct_order(
            data,
            MaxFirst::new::<TestItemType>(ByOrd),
            tie_breaker::ReverseInsertionOrder,
        );
        assert_correct_order(
            data,
            MaxFirst::new::<TestItemType>(ByOrd),
            tie_breaker::Unspecified,
        );
    }
}
