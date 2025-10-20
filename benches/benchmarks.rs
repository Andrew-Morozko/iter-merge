#[cfg(benchmarking)]
mod code;
#[cfg(benchmarking)]
criterion::criterion_main!(code::benches);

#[cfg(not(benchmarking))]
fn main() {
    panic!(
        "Benchmarking disabled! Run benchmarks with:
    RUSTFLAGS='-C target-cpu=native --cfg benchmarking' cargo bench --bench benchmarks"
    );
}
