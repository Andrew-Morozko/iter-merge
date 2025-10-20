use std::{hint::black_box, pin::pin};

use criterion::{BenchmarkId, Criterion, criterion_group};
use iter_merge::{ArrayStorage, VecStorage, comparators::tie_breaker};
use itertools::kmerge;
use rand::prelude::*;

#[inline(always)]
fn consume<T>(item: T) {
    drop(black_box(item));
}

fn next_divisible_by_all<I>(iter: I, target: usize) -> usize
where
    I: Iterator<Item = usize>,
{
    fn gcd(mut a: usize, mut b: usize) -> usize {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }
    fn lcm(a: usize, b: usize) -> usize {
        a.checked_mul(b).unwrap() / gcd(a, b)
    }

    let lcm = iter.fold(1, lcm);
    target.next_multiple_of(lcm)
}

fn make_iters(
    n_iters: usize, vec: &'_ [u64],
) -> impl Iterator<Item = impl Iterator<Item = u64> + '_> + '_ {
    let it_len = vec.len() / n_iters;
    (0..n_iters).map(move |iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied())
}

fn bench_itertools(c: &mut Criterion) {
    let iter_counts = [8, 64, 256, 1024, 2048, 4096];
    let mut rng = StdRng::seed_from_u64(0);
    let mut vec = rng
        .clone()
        .random_iter()
        .take(next_divisible_by_all(
            iter_counts.iter().copied(),
            2_usize.pow(20),
        ))
        .collect::<Vec<u64>>();

    let mut group = c.benchmark_group("Random, by single element");
    for &n_iters in &iter_counts {
        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("IterMerge", n_iters), |b| {
            b.iter(|| {
                VecStorage::from_iter(make_iters(n_iters, &vec))
                    .into_builder()
                    .tie_breaker(tie_breaker::Unspecified)
                    .build()
                    .for_each(consume)
            });
        });
        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("Itertools kmerge", n_iters), |b| {
            b.iter(|| kmerge(make_iters(n_iters, &vec)).for_each(consume));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("Random");
    for &n_iters in &iter_counts {
        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("IterMerge", n_iters), |b| {
            b.iter(|| {
                VecStorage::from_iter(make_iters(n_iters, &vec))
                    .into_builder()
                    .tie_breaker(tie_breaker::Unspecified)
                    .build()
                    .into_vec()
            });
        });
        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("Itertools kmerge", n_iters), |b| {
            b.iter(|| {
                kmerge(
                    make_iters(n_iters, &vec), // |a: &u64, b: &u64| a < b,
                )
                .collect::<Vec<_>>()
            });
        });
    }
    group.finish();

    let mut sorted_vec = vec.clone();
    sorted_vec.sort();

    let mut group = c.benchmark_group("Worst case");
    for &n_iters in &iter_counts {
        let it_len = sorted_vec.len() / n_iters;
        vec.clear();
        (0..n_iters).for_each(|iter_n| {
            vec.extend(sorted_vec.iter().skip(iter_n).step_by(it_len).copied());
        });
        vec = black_box(vec);
        // now iterators constantly switch places

        group.bench_function(BenchmarkId::new("IterMerge", n_iters), |b| {
            b.iter(|| {
                VecStorage::from_iter(make_iters(n_iters, &vec))
                    .into_builder()
                    .tie_breaker(tie_breaker::Unspecified)
                    .build()
                    .into_vec()
            });
        });

        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("Itertools kmerge", n_iters), |b| {
            b.iter(|| kmerge(make_iters(n_iters, &vec)).collect::<Vec<_>>());
        });
    }
    group.finish();

    vec = sorted_vec;

    let mut group = c.benchmark_group("Fully ordered");
    for &n_iters in &iter_counts {
        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("IterMerge", n_iters), |b| {
            b.iter(|| {
                VecStorage::from_iter(make_iters(n_iters, &vec))
                    .into_builder()
                    .tie_breaker(tie_breaker::Unspecified)
                    .build()
                    .into_vec()
            });
        });
        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("Itertools kmerge", n_iters), |b| {
            b.iter(|| kmerge(make_iters(n_iters, &vec)).collect::<Vec<_>>());
        });
    }
    group.finish();

    {
        // 1% of elements is out of order
        let mut indexes = Vec::from_iter(0..vec.len());
        for _ in 0..(vec.len() / 200) {
            vec.swap(
                indexes.swap_remove(rng.random_range(0..indexes.len())),
                indexes.swap_remove(rng.random_range(0..indexes.len())),
            );
        }
    }

    let mut group = c.benchmark_group("Partially ordered");
    for &n_iters in &iter_counts {
        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("IterMerge", n_iters), |b| {
            b.iter(|| {
                VecStorage::from_iter(make_iters(n_iters, &vec))
                    .into_builder()
                    .tie_breaker(tie_breaker::Unspecified)
                    .build()
                    .into_vec()
            });
        });
        vec = black_box(vec);
        group.bench_function(BenchmarkId::new("Itertools kmerge", n_iters), |b| {
            b.iter(|| kmerge(make_iters(n_iters, &vec)).collect::<Vec<_>>());
        });
    }
    group.finish();
}

fn bench_configs(c: &mut Criterion) {
    const N_ITERS: usize = 256;
    let n_els = 2_usize.pow(20);
    let mut vec = StdRng::seed_from_u64(0)
        .random_iter()
        .take(n_els)
        .collect::<Vec<u64>>();

    let mut group = c.benchmark_group(format!("Configs ({n_els} items; {N_ITERS} iters)",));

    vec = black_box(vec);
    group.bench_function("Arbitrary", |b| {
        b.iter(|| {
            VecStorage::from_iter(make_iters(N_ITERS, &vec))
                .into_builder()
                .tie_breaker(tie_breaker::Unspecified)
                .build()
                .into_vec()
        });
    });
    vec = black_box(vec);
    group.bench_function("Stable", |b| {
        b.iter(|| {
            VecStorage::from_iter(make_iters(N_ITERS, &vec))
                .into_builder()
                .tie_breaker(tie_breaker::Unspecified)
                .build()
                .into_vec()
        });
    });
    vec = black_box(vec);
    group.bench_function("Arbitrary, stack", |b| {
        b.iter(|| {
            let mut s = ArrayStorage::with_capacity::<N_ITERS>();
            s.extend(make_iters(N_ITERS, &vec));
            let s = pin!(s);
            s.into_builder()
                .tie_breaker(tie_breaker::Unspecified)
                .build()
                .into_vec()
        });
    });
    vec = black_box(vec);
    group.bench_function("Stable, stack", |b| {
        b.iter(|| {
            let mut s = ArrayStorage::with_capacity::<N_ITERS>();
            s.extend(make_iters(N_ITERS, &vec));
            let s = pin!(s);
            s.build().into_vec()
        });
    });
    group.finish();
}

fn collect(c: &mut Criterion) {
    const N_ITERS: usize = 256;
    let n_els = 2_usize.pow(20);
    let mut vec = StdRng::seed_from_u64(0)
        .random_iter()
        .take(n_els)
        .collect::<Vec<u64>>();

    let mut group = c.benchmark_group("consumption modes");

    vec = black_box(vec);
    group.bench_function("into_vec", |b| {
        b.iter(|| {
            VecStorage::from_iter(make_iters(N_ITERS, &vec))
                .into_builder()
                .tie_breaker(tie_breaker::Unspecified)
                .build()
                .into_vec()
        });
    });

    vec = black_box(vec);
    group.bench_function("collect", |b| {
        b.iter(|| {
            VecStorage::from_iter(make_iters(N_ITERS, &vec))
                .into_builder()
                .tie_breaker(tie_breaker::Unspecified)
                .build()
                .collect::<Vec<_>>()
        });
    });

    vec = black_box(vec);
    group.bench_function("next", |b| {
        b.iter(|| {
            VecStorage::from_iter(make_iters(N_ITERS, &vec))
                .into_builder()
                .tie_breaker(tie_breaker::Unspecified)
                .build()
                .for_each(consume)
        });
    });
    group.finish();
}

criterion_group!(benches, bench_itertools, bench_configs, collect);
