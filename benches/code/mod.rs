use criterion::{BenchmarkId, Criterion, criterion_group};
use iter_merge::Merged;
use itertools::kmerge_by;
use rand::prelude::*;

fn closest_divisible<I>(iter: I, target: usize) -> usize
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
    ((target as f64) / (lcm as f64)).round() as usize * lcm
}

fn bench_itertools(c: &mut Criterion) {
    let iter_counts = [8, 32, 128, 256, 512, 1024, 1536, 2048, 3072, 4096];
    let n_els = closest_divisible(iter_counts.iter().copied(), 2_usize.pow(20));
    let mut rng = StdRng::seed_from_u64(0);
    let mut vec = rng.clone().random_iter().take(n_els).collect::<Vec<u64>>();

    let mut group = c.benchmark_group("Random items VS Itertools");
    for n_iters in iter_counts.iter().copied() {
        let it_len = n_els / n_iters;
        group.bench_function(BenchmarkId::new("IterMerge", n_iters), |b| {
            b.iter(|| {
                Merged::new(
                    (0..n_iters)
                        .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
                )
                .arbitrary_tie_breaking()
                .build()
                .into_vec()
            });
        });
        group.bench_function(BenchmarkId::new("Itertools kmerge_by", n_iters), |b| {
            b.iter(|| {
                kmerge_by(
                    (0..n_iters)
                        .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
                    |a: &u64, b: &u64| a < b,
                )
                .collect::<Vec<_>>()
            });
        });
    }
    group.finish();

    vec.clear();
    vec.extend(0..(n_els as u64));

    // 1% of elements is out of order
    for _ in 0..(n_els / 200) {
        vec.swap(rng.random_range(0..n_els), rng.random_range(0..n_els));
    }

    let mut group = c.benchmark_group("Partially ordered VS Itertools");
    for n_iters in iter_counts.iter().copied() {
        let it_len = n_els / n_iters;
        group.bench_function(BenchmarkId::new("IterMerge", n_iters), |b| {
            b.iter(|| {
                Merged::new(
                    (0..n_iters)
                        .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
                )
                .arbitrary_tie_breaking()
                .build()
                .into_vec()
            });
        });
        group.bench_function(BenchmarkId::new("Itertools kmerge_by", n_iters), |b| {
            b.iter(|| {
                kmerge_by(
                    (0..n_iters)
                        .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
                    |a: &u64, b: &u64| a < b,
                )
                .collect::<Vec<_>>()
            });
        });
    }
    group.finish();

    vec.clear();
    vec.extend(0..(n_els as u64));

    let mut group = c.benchmark_group("Fully ordered VS Itertools");
    for n_iters in iter_counts.iter().copied() {
        let it_len = n_els / n_iters;
        group.bench_function(BenchmarkId::new("IterMerge", n_iters), |b| {
            b.iter(|| {
                Merged::new(
                    (0..n_iters)
                        .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
                )
                .arbitrary_tie_breaking()
                .build()
                .into_vec()
            });
        });
        group.bench_function(BenchmarkId::new("Itertools kmerge_by", n_iters), |b| {
            b.iter(|| {
                kmerge_by(
                    (0..n_iters)
                        .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
                    |a: &u64, b: &u64| a < b,
                )
                .collect::<Vec<_>>()
            });
        });
    }
    group.finish();
}

fn bench_configs(c: &mut Criterion) {
    const N_ITERS: usize = 64;
    let n_els = 2_usize.pow(20);
    let vec = StdRng::seed_from_u64(0)
        .random_iter()
        .take(n_els)
        .collect::<Vec<u64>>();

    let mut group = c.benchmark_group(format!(
        "Configs ({n_els} items; {N_ITERS} iters{})",
        if cfg!(feature = "forbid_unsafe") {
            "; forbid_unsafe"
        } else {
            ""
        }
    ));

    let it_len = n_els / N_ITERS;
    group.bench_function("Arbitrary", |b| {
        b.iter(|| {
            Merged::new(
                (0..N_ITERS)
                    .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
            )
            .arbitrary_tie_breaking()
            .build()
            .into_vec()
        });
    });
    group.bench_function("Stable", |b| {
        b.iter(|| {
            Merged::new(
                (0..N_ITERS)
                    .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
            )
            .build()
            .into_vec()
        });
    });
    #[cfg(feature = "stackvec_storage")]
    group.bench_function("Arbitrary, stackvec", |b| {
        b.iter(|| {
            Merged::new_stackvec::<N_ITERS>(
                (0..N_ITERS)
                    .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
            )
            .arbitrary_tie_breaking()
            .build()
            .into_vec()
        });
    });
    #[cfg(feature = "stackvec_storage")]
    group.bench_function("Stable, stackvec", |b| {
        b.iter(|| {
            Merged::new_stackvec::<N_ITERS>(
                (0..N_ITERS)
                    .map(|iter_n| vec.iter().skip(iter_n * it_len).take(it_len).copied()),
            )
            .build()
            .into_vec()
        });
    });
    group.finish();
}

#[cfg(not(feature = "forbid_unsafe"))]
criterion_group!(benches, bench_itertools, bench_configs);
#[cfg(feature = "forbid_unsafe")]
criterion_group!(benches, bench_configs);
