# iter-merge

[![Crates.io](https://img.shields.io/crates/v/iter-merge)](https://crates.io/crates/iter-merge)
[![Documentation](https://docs.rs/iter-merge/badge.svg)](https://docs.rs/iter-merge)
[![CI Test Status](https://github.com/Andrew-Morozko/iter-merge/actions/workflows/Tests.yml/badge.svg)](https://github.com/Andrew-Morozko/iter-merge/actions/workflows/Tests.yml)

A high-performance iterator merging library for Rust that efficiently combines multiple iterators into a single iterator, yielding smallest item first.

Note: only compares the first items across provided iterators. The output would be sorted only if the iterators themselves are sorted.

## Features

- Optimized for small to medium numbers of iterators
- Lazy iterator consumption: each `.next()` advances only one iterator
- Support for custom comparison functions
- Multiple storage backends: `Vec` and `StackVec` (for no-std and no-alloc compatibility)
- Optionally `forbid_unsafe`
- Dynamic iterator addition after construction
- Additional [`Peekable`](https://doc.rust-lang.org/std/iter/struct.Peekable.html) methods

## Usage

```rust
use iter_merge::Merged;

let iter1 = vec![1, 3, 5, 7];
let iter2 = vec![2, 4, 6, 8];

let mut merged = Merged::new([iter1, iter2]).build();
let result = merged.into_vec();

assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8]);
```

### Custom Comparison

```rust
use iter_merge::Merged;

let mut merged = Merged::new([
    vec![9, 6, 3],
    vec![8, 5, 2],
    vec![7, 4, 1],
])
.with_cmp(|a, b| b.cmp(a))
.build();

let result = merged.into_vec();
assert_eq!(result, vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
```

### Dynamic Iterator Addition

```rust
use iter_merge::Merged;

let mut merged = Merged::new([vec![0, 1]]).build();
assert_eq!(merged.next(), Some(0));

merged.add_iter(vec![2]);
merged.add_iters([vec![3], vec![4]]);

let result = merged.into_vec();
assert_eq!(result, vec![1, 2, 3, 4]);
```

## Performance

This library is optimized for merging small to medium numbers of iterators. When tested
with ~1 000 000 random `u64`s:

- Up to ~123 iterators: 2x faster than `itertools::kmerge`
- Up to ~355 iterators: 1.5x faster than `itertools::kmerge`
- Break-even point: ~682 iterators, after that the `O(iterator_count²)` starts to dominate
- Performs better when an iterator contains a run of the min items
- Arbitrary tie-breaking: 23% faster than stable tie-breaking
- Unsafe optimizations: 49% faster than safe-only mode

## Features

The crate supports several feature flags:

- `vec_storage` (default): Enable heap-allocated storage with `Vec`, requires `alloc`
- `stackvec_storage`: Enable stack-allocated storage with `StackVec`
- `forbid_unsafe`: Disable all unsafe code

## Testing

Run the fuzz test:
* Uncomment `For fuzzing` blocks in the [Cargo.toml](Cargo.toml) or run `./toggle_fuzz_config.sh`
* Run tests:
  ```bash
  cargo +nightly fuzzcheck fuzz_merge --test fuzz --profile fuzz
  cargo +nightly fuzzcheck fuzz_merge_correctness --test fuzz --profile fuzz
  ```

Run benchmarks:

```bash
RUSTFLAGS='-C target-cpu=native --cfg benchmarking'
cargo bench
```
(Benchmarking dependencies are behind `benchmarking` cfg option to keep MSRV low)

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