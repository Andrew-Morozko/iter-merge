#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate iter_merge;
use iter_merge::comparators::{tie_breaker, ByOrd};
fuzz_target!(|data: Vec<Vec<i8>>| {
    // fuzzed code goes here
    iter_merge::tests::order::assert_correct_order(
        &data,
        ByOrd,
        tie_breaker::InsertionOrder
    );
    iter_merge::tests::order::assert_correct_order(
        &data,
        ByOrd,
        tie_breaker::Unspecified
    );
});
