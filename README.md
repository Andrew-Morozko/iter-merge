# iter-merge

[![Crates.io](https://img.shields.io/crates/v/iter-merge)](https://crates.io/crates/iter-merge)
[![Documentation](https://docs.rs/iter-merge/badge.svg)](https://docs.rs/iter-merge)
[![CI Test Status](https://github.com/Andrew-Morozko/iter-merge/actions/workflows/Tests.yml/badge.svg)](https://github.com/Andrew-Morozko/iter-merge/actions/workflows/Tests.yml)

A high-performance iterator merging library for Rust that efficiently combines multiple iterators into a single iterator, yielding smallest item first.

Note: only compares the first items across provided iterators. The output would be sorted only if the iterators themselves are sorted.

## Features

- Lazy iterator consumption: each `.next()` advances only one iterator
- Support for custom comparison functions
- Multiple storage backends: `Vec` and `Array` (for no-std and no-alloc compatibility)
- Additional [`Peekable`](https://doc.rust-lang.org/std/iter/struct.Peekable.html) methods

## Usage

```rust
use iter_merge::merge;

let iter1 = vec![1, 3, 5, 7];
let iter2 = vec![2, 4, 6, 8];

let result = merge([iter1, iter2]).into_vec();

assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8]);
```

### Custom Comparison

```rust
use iter_merge::{VecStorage, comparators::ByOrd};

let mut merged = VecStorage::from_iter([
    vec![9, 6, 3],
    vec![8, 5, 2],
    vec![7, 4, 1],
])
.into_builder()
.max_by(ByOrd)
.build();

let result = merged.into_vec();
assert_eq!(result, vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
```

## Performance

It's 1.45-1.65x faster than `itertools::kmerge` in my benchmarks and scales as `O(item_count + log₂(iterator_count))` (just as well as `itertools::kmerge`)

## Features

The crate supports following feature flags:
- `alloc`: Enables heap-allocated storage with `VecStorage` and methods like `MergeIter::into_vec`

## Testing

Fuzzing:
Use [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz)

```bash
cargo +nightly fuzz run fuzz_correctness
cargo +nightly fuzz run fuzz_usage
```

Miri:
```bash
cargo +nightly miri test
```

Benchmarks:
```bash
RUSTFLAGS='-C target-cpu=native --cfg benchmarking' cargo bench --bench benchmarks
```
(Benchmarking dependencies are behind `benchmarking` cfg option)

## MSRV
The minimum supported Rust version for this crate is 1.68.0.
This may change in a major release, but it is unlikely.

## Contributing

Contributions are welcome. For major changes, please open an issue first to discuss what you would like to change.

## Related Projects

- [`itertools`](https://crates.io/crates/itertools) - General iterator utilities (includes `kmerge`)

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>