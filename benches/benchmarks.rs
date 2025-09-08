#[cfg(benchmarking)]
mod code;
#[cfg(benchmarking)]
criterion::criterion_main!(code::benches);

#[cfg(not(benchmarking))]
fn main() {
    panic!(
        r"Benchmarking disabled! Run bechmarks with:
    RUSTFLAGS='-C target-cpu=native --cfg benchmarking' cargo bench"
    );
}
