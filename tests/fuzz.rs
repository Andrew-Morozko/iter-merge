#![cfg(fuzzing)]
#![feature(coverage_attribute)]

use fuzzcheck::DefaultMutator;
use serde::{Deserialize, Serialize};

mod helpers;

use helpers::test_all_merges;

#[derive(Clone, Debug, Serialize, Deserialize, DefaultMutator)]
struct Wrapper(pub Vec<Vec<i8>>);

impl Default for Wrapper {
    fn default() -> Self {
        Wrapper(Vec::new())
    }
}

fn test_merge_correctness(input: &Wrapper) {
    test_all_merges(&input.0)
}

#[test]
fn fuzz_merge_correctness() {
    let result = fuzzcheck::fuzz_test(test_merge_correctness)
        .default_options()
        .launch();
    assert!(!result.found_test_failure);
}

