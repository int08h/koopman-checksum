//! Benchmarks for Koopman checksum implementations.
//!
//! Run with: `cargo bench`

use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId,
    Criterion, Throughput,
};
use koopman_checksum::*;
use std::time::Duration;

fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i & 0xFF) as u8).collect()
}

/// Speed up benchmark runs by reducing measurement time and warm-up time.
fn fast_config(group: &mut BenchmarkGroup<WallTime>) {
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));
}

fn bench_koopman8(c: &mut Criterion) {
    let mut group = c.benchmark_group("Koopman8");
    fast_config(&mut group);

    for size in [64, 256, 1024, 4096].iter() {
        let data = generate_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("checksum", size), &data, |b, data| {
            b.iter(|| koopman8(black_box(data), 0))
        });
    }

    group.finish();
}

fn bench_koopman16(c: &mut Criterion) {
    let mut group = c.benchmark_group("Koopman16");
    fast_config(&mut group);

    for size in [64, 256, 1024, 4096, 16384, 65536].iter() {
        let data = generate_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("checksum", size), &data, |b, data| {
            b.iter(|| koopman16(black_box(data), 0))
        });
    }

    group.finish();
}

fn bench_koopman32(c: &mut Criterion) {
    let mut group = c.benchmark_group("Koopman32");
    fast_config(&mut group);

    for size in [64, 256, 1024, 4096, 16384, 65536].iter() {
        let data = generate_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("checksum", size), &data, |b, data| {
            b.iter(|| koopman32(black_box(data), 0))
        });
    }

    group.finish();
}

fn bench_koopman8p(c: &mut Criterion) {
    let mut group = c.benchmark_group("Koopman8P");
    fast_config(&mut group);

    for size in [64, 256, 1024, 4096].iter() {
        let data = generate_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("checksum", size), &data, |b, data| {
            b.iter(|| koopman8p(black_box(data), 0))
        });
    }

    group.finish();
}

fn bench_koopman16p(c: &mut Criterion) {
    let mut group = c.benchmark_group("Koopman16P");
    fast_config(&mut group);

    for size in [64, 256, 1024, 4096].iter() {
        let data = generate_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("checksum", size), &data, |b, data| {
            b.iter(|| koopman16p(black_box(data), 0))
        });
    }

    group.finish();
}

fn bench_koopman32p(c: &mut Criterion) {
    let mut group = c.benchmark_group("Koopman32P");
    fast_config(&mut group);

    for size in [64, 256, 1024, 4096].iter() {
        let data = generate_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("checksum", size), &data, |b, data| {
            b.iter(|| koopman32p(black_box(data), 0))
        });
    }

    group.finish();
}

fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("Streaming");
    fast_config(&mut group);

    let data = generate_test_data(4096);
    group.throughput(Throughput::Bytes(4096));

    group.bench_function("one_shot", |b| b.iter(|| koopman16(black_box(&data), 0)));

    group.bench_function("streaming_single_update", |b| {
        b.iter(|| {
            let mut hasher = Koopman16::new();
            hasher.update(black_box(&data));
            hasher.finalize()
        })
    });

    group.bench_function("streaming_chunked_64", |b| {
        b.iter(|| {
            let mut hasher = Koopman16::new();
            for chunk in data.chunks(64) {
                hasher.update(black_box(chunk));
            }
            hasher.finalize()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_koopman8,
    bench_koopman16,
    bench_koopman32,
    bench_koopman8p,
    bench_koopman16p,
    bench_koopman32p,
    bench_streaming,
);

criterion_main!(benches);
