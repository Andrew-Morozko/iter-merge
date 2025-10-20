#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate iter_merge;
use iter_merge::{VecStorage, ArrayStorage};
use std::hint::black_box;
use std::pin::pin;

fn consume<T>(item: T){
    drop(black_box(item))
}

fuzz_target!(|data: Vec<Vec<i8>>| {
    // fuzzed code goes here
    VecStorage::from_iter(data.iter().map(|it| it.iter().copied()))
        .build()
        .for_each(consume);
    const CAP: usize = 10;
    let s = ArrayStorage::<CAP, _>::from_iter(
        data.iter().take(CAP).map(|it| it.iter().copied()),
    );
    let s = pin!(s);
    s.build().for_each(consume);
});
