#![cfg(fuzzing)]
#![cfg_attr(fuzzing, feature(coverage_attribute))]

#[cfg(not(debug_assertions))]
compile_error!("incorrect fuzzing configuration, use fuzz profile");

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

#[test]
fn fuzz_merge_correctness() {
    fn fuzz(input: &Wrapper) {
        test_all_merges(&input.0)
    }
    let result = fuzzcheck::fuzz_test(fuzz).default_options().launch();
    assert!(!result.found_test_failure);
}

// Make sure that the iterator is actually consumed
fn consume<I>(iter: I)
where
    I: Iterator,
{
    for item in iter {
        std::hint::black_box(item);
    }
}

#[test]
fn fuzz_merge() {
    use iter_merge::Merged;
    fn fuzz(input: &Wrapper) {
        consume(Merged::new(&input.0).build());
        consume(Merged::new(&input.0).arbitrary_tie_breaking().build());
    }
    let result = fuzzcheck::fuzz_test(fuzz).default_options().launch();
    assert!(!result.found_test_failure);
}
