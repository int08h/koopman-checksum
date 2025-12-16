//! Benchmarks comparing different optimization strategies for Koopman checksums.
//!
//! Run with: `cargo bench --bench optimization_comparison`

use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup,
    BenchmarkId, Criterion, Throughput,
};
use std::time::Duration;

// ============================================================================
// Constants
// ============================================================================

const MODULUS_16: u32 = 65519;   // 2^16 - 17
const MODULUS_32: u64 = 4294967291; // 2^32 - 5

// ============================================================================
// Original Implementation (baseline)
// ============================================================================

fn koopman16_original(data: &[u8], initial_seed: u8) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % MODULUS_16;
    }

    sum = (sum << 8) % MODULUS_16;
    sum = (sum << 8) % MODULUS_16;

    sum as u16
}

fn koopman32_original(data: &[u8], initial_seed: u8) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u64) % MODULUS_32;
    }

    sum = (sum << 8) % MODULUS_32;
    sum = (sum << 8) % MODULUS_32;
    sum = (sum << 8) % MODULUS_32;
    sum = (sum << 8) % MODULUS_32;

    sum as u32
}

// ============================================================================
// Optimization 1: Fast Modular Reduction (2^k - c trick)
//
// For modulus = 2^k - c:
// x % (2^k - c) ≡ (x >> k) * c + (x & (2^k - 1)) (mod 2^k - c)
// ============================================================================

/// Fast reduction for modulus 65519 = 2^16 - 17
/// Input: x < 2^25 (after shift+add, max is 65518*256 + 255 = 16,772,863)
#[inline(always)]
fn fast_mod_65519(x: u32) -> u32 {
    // First reduction: x = hi * 17 + lo where hi = x >> 16, lo = x & 0xFFFF
    // Result < 17 * 256 + 65536 = 69888 (since x < 2^25 means hi < 512)
    let hi = x >> 16;
    let lo = x & 0xFFFF;
    let r = hi * 17 + lo;

    // Second reduction: r < 69888, so hi2 < 2
    // Result < 17 * 2 + 65536 = 65570
    let hi2 = r >> 16;
    let lo2 = r & 0xFFFF;
    let r2 = hi2 * 17 + lo2;

    // Final correction: r2 < 65570, might be >= 65519
    if r2 >= MODULUS_16 { r2 - MODULUS_16 } else { r2 }
}

/// Fast reduction for modulus 4294967291 = 2^32 - 5
/// Input: x < 2^41 (after shift+add)
#[inline(always)]
fn fast_mod_4294967291(x: u64) -> u64 {
    // First reduction
    let hi = x >> 32;
    let lo = x & 0xFFFFFFFF;
    let r = hi * 5 + lo;

    // r < 5 * 2^9 + 2^32 ≈ 2^32, might need one more
    // Actually max hi after shift is (2^32-1), so r < 5*(2^32-1) + 2^32 could overflow
    // But our input x < 2^41, so hi < 2^9, r < 5*512 + 2^32 = 2560 + 2^32

    // Second reduction just in case
    let hi2 = r >> 32;
    let lo2 = r & 0xFFFFFFFF;
    let r2 = hi2 * 5 + lo2;

    if r2 >= MODULUS_32 { r2 - MODULUS_32 } else { r2 }
}

fn koopman16_fast_mod(data: &[u8], initial_seed: u8) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;

    for &byte in &data[1..] {
        sum = fast_mod_65519((sum << 8) + byte as u32);
    }

    sum = fast_mod_65519(sum << 8);
    sum = fast_mod_65519(sum << 8);

    sum as u16
}

fn koopman32_fast_mod(data: &[u8], initial_seed: u8) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    for &byte in &data[1..] {
        sum = fast_mod_4294967291((sum << 8) + byte as u64);
    }

    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);

    sum as u32
}

// ============================================================================
// Optimization 2: Barrett Reduction
//
// Precompute k = ceil(2^n / modulus) and use:
// q = (x * k) >> n
// r = x - q * modulus
// ============================================================================

// For 65519: we want k such that (x * k) >> 32 ≈ x / 65519
// k = ceil(2^32 / 65519) = 65545
const BARRETT_16_K: u64 = 65545;
const BARRETT_16_SHIFT: u32 = 32;

#[inline(always)]
fn barrett_mod_65519(x: u32) -> u32 {
    let q = ((x as u64 * BARRETT_16_K) >> BARRETT_16_SHIFT) as u32;
    let r = x - q * MODULUS_16;
    // r might be >= modulus due to rounding, need correction
    if r >= MODULUS_16 { r - MODULUS_16 } else { r }
}

// For 4294967291: k = ceil(2^64 / 4294967291) but this overflows
// Use a 64-bit approximation: k = ceil(2^64 / m) computed carefully
// Actually we need 128-bit arithmetic for proper Barrett on 32-bit modulus with 64-bit input
// Let's use a simpler approach: k = 2^32 + 5 (since m = 2^32 - 5)
const BARRETT_32_K: u128 = (1u128 << 64) / (MODULUS_32 as u128) + 1;

#[inline(always)]
fn barrett_mod_4294967291(x: u64) -> u64 {
    // Use 128-bit arithmetic
    let q = ((x as u128 * BARRETT_32_K) >> 64) as u64;
    let r = x.wrapping_sub(q.wrapping_mul(MODULUS_32));
    // Correction
    if r >= MODULUS_32 { r - MODULUS_32 } else { r }
}

fn koopman16_barrett(data: &[u8], initial_seed: u8) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;

    for &byte in &data[1..] {
        sum = barrett_mod_65519((sum << 8) + byte as u32);
    }

    sum = barrett_mod_65519(sum << 8);
    sum = barrett_mod_65519(sum << 8);

    sum as u16
}

fn koopman32_barrett(data: &[u8], initial_seed: u8) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    for &byte in &data[1..] {
        sum = barrett_mod_4294967291((sum << 8) + byte as u64);
    }

    sum = barrett_mod_4294967291(sum << 8);
    sum = barrett_mod_4294967291(sum << 8);
    sum = barrett_mod_4294967291(sum << 8);
    sum = barrett_mod_4294967291(sum << 8);

    sum as u32
}

// ============================================================================
// Optimization 3: Delayed Reduction
//
// Process multiple bytes before applying modulo to reduce division count
// ============================================================================

fn koopman16_delayed(data: &[u8], initial_seed: u8) -> u16 {
    if data.is_empty() {
        return 0;
    }

    // Use u64 to allow accumulating multiple bytes
    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    // Process bytes, reducing every 2 bytes to prevent overflow
    // After 2 shifts without mod: sum < 65519 * 256 * 256 + 255*256 + 255 < 2^32
    let mut count = 0;
    for &byte in &data[1..] {
        sum = (sum << 8) + byte as u64;
        count += 1;
        if count == 2 {
            sum %= MODULUS_16 as u64;
            count = 0;
        }
    }

    // Final reduction if needed
    if count > 0 {
        sum %= MODULUS_16 as u64;
    }

    // Finalization
    sum = (sum << 8) % MODULUS_16 as u64;
    sum = (sum << 8) % MODULUS_16 as u64;

    sum as u16
}

fn koopman32_delayed(data: &[u8], initial_seed: u8) -> u32 {
    if data.is_empty() {
        return 0;
    }

    // Use u128 to allow accumulating multiple bytes
    let mut sum: u128 = (data[0] ^ initial_seed) as u128;

    // Process bytes, reducing every 3 bytes to prevent overflow
    let mut count = 0;
    for &byte in &data[1..] {
        sum = (sum << 8) + byte as u128;
        count += 1;
        if count == 3 {
            sum %= MODULUS_32 as u128;
            count = 0;
        }
    }

    if count > 0 {
        sum %= MODULUS_32 as u128;
    }

    // Finalization
    sum = (sum << 8) % MODULUS_32 as u128;
    sum = (sum << 8) % MODULUS_32 as u128;
    sum = (sum << 8) % MODULUS_32 as u128;
    sum = (sum << 8) % MODULUS_32 as u128;

    sum as u32
}

// ============================================================================
// Optimization 4: Loop Unrolling (with fast mod)
// ============================================================================

fn koopman16_unrolled(data: &[u8], initial_seed: u8) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;
    let rest = &data[1..];

    // Process 4 bytes at a time
    let chunks = rest.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        sum = fast_mod_65519((sum << 8) + chunk[0] as u32);
        sum = fast_mod_65519((sum << 8) + chunk[1] as u32);
        sum = fast_mod_65519((sum << 8) + chunk[2] as u32);
        sum = fast_mod_65519((sum << 8) + chunk[3] as u32);
    }

    for &byte in remainder {
        sum = fast_mod_65519((sum << 8) + byte as u32);
    }

    sum = fast_mod_65519(sum << 8);
    sum = fast_mod_65519(sum << 8);

    sum as u16
}

fn koopman32_unrolled(data: &[u8], initial_seed: u8) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;
    let rest = &data[1..];

    let chunks = rest.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        sum = fast_mod_4294967291((sum << 8) + chunk[0] as u64);
        sum = fast_mod_4294967291((sum << 8) + chunk[1] as u64);
        sum = fast_mod_4294967291((sum << 8) + chunk[2] as u64);
        sum = fast_mod_4294967291((sum << 8) + chunk[3] as u64);
    }

    for &byte in remainder {
        sum = fast_mod_4294967291((sum << 8) + byte as u64);
    }

    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);

    sum as u32
}

// ============================================================================
// Optimization 5: Combined - Delayed + Fast Mod
// ============================================================================

fn koopman16_delayed_fast(data: &[u8], initial_seed: u8) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    // We can delay reduction because fast_mod handles larger inputs
    // But we still need to prevent overflow of u64
    // With u64 and values < 2^32, we can do many iterations safely
    let mut count = 0;
    for &byte in &data[1..] {
        sum = (sum << 8) + byte as u64;
        count += 1;
        // Reduce every 2 bytes to keep sum manageable
        if count == 2 {
            sum = fast_mod_65519(sum as u32) as u64;
            count = 0;
        }
    }

    if count > 0 {
        sum = fast_mod_65519(sum as u32) as u64;
    }

    sum = fast_mod_65519((sum << 8) as u32) as u64;
    sum = fast_mod_65519((sum << 8) as u32) as u64;

    sum as u16
}

// ============================================================================
// Optimization 6: Delayed Fast for Koopman32 (using u64 only, no u128)
// ============================================================================

/// Fast reduction for modulus 4294967291 = 2^32 - 5
/// Input must be < 2^40 for this to work correctly
#[inline(always)]
fn fast_mod_32_small(x: u64) -> u64 {
    // x = hi * 2^32 + lo
    // x mod (2^32 - 5) = hi * 5 + lo (mod 2^32 - 5)
    let hi = x >> 32;
    let lo = x & 0xFFFFFFFF;
    let r = hi * 5 + lo;
    // r < 5 * 2^8 + 2^32 = 2^32 + 1280, need one more check
    if r >= MODULUS_32 { r - MODULUS_32 } else { r }
}

fn koopman32_delayed_fast(data: &[u8], initial_seed: u8) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    // Process one byte at a time with fast mod
    // The key insight: after modulo, sum < 2^32
    // After (sum << 8) + byte, sum < 2^40
    // fast_mod_32_small handles inputs < 2^40
    for &byte in &data[1..] {
        sum = fast_mod_32_small((sum << 8) + byte as u64);
    }

    // Finalization
    sum = fast_mod_32_small(sum << 8);
    sum = fast_mod_32_small(sum << 8);
    sum = fast_mod_32_small(sum << 8);
    sum = fast_mod_32_small(sum << 8);

    sum as u32
}

// ============================================================================
// Correctness Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[allow(dead_code)]
    const TEST_DATA: &[u8] = b"123456789";
    #[allow(dead_code)]
    const LONG_DATA: &[u8] = b"The quick brown fox jumps over the lazy dog";

    #[test]
    fn test_koopman16_variants_match() {
        for data in [TEST_DATA, LONG_DATA, &[0x12, 0x34, 0x56]] {
            let expected = koopman16_original(data, 0);
            assert_eq!(koopman16_fast_mod(data, 0), expected, "fast_mod mismatch");
            assert_eq!(koopman16_barrett(data, 0), expected, "barrett mismatch");
            assert_eq!(koopman16_delayed(data, 0), expected, "delayed mismatch");
            assert_eq!(koopman16_unrolled(data, 0), expected, "unrolled mismatch");
            assert_eq!(koopman16_delayed_fast(data, 0), expected, "delayed_fast mismatch");
        }
    }

    #[test]
    fn test_koopman32_variants_match() {
        for data in [TEST_DATA, LONG_DATA, &[0x12, 0x34, 0x56]] {
            let expected = koopman32_original(data, 0);
            assert_eq!(koopman32_fast_mod(data, 0), expected, "fast_mod mismatch");
            assert_eq!(koopman32_barrett(data, 0), expected, "barrett mismatch");
            assert_eq!(koopman32_delayed(data, 0), expected, "delayed mismatch");
            assert_eq!(koopman32_unrolled(data, 0), expected, "unrolled mismatch");
            assert_eq!(koopman32_delayed_fast(data, 0), expected, "delayed_fast mismatch");
        }
    }

    #[test]
    fn test_fast_mod_65519() {
        // Test the fast mod function against regular mod
        for x in [0u32, 1, 65518, 65519, 65520, 100000, 1000000, 16772863] {
            assert_eq!(fast_mod_65519(x), x % MODULUS_16, "fast_mod_65519 failed for {}", x);
        }
    }

    #[test]
    fn test_fast_mod_4294967291() {
        for x in [0u64, 1, MODULUS_32 - 1, MODULUS_32, MODULUS_32 + 1, 1_000_000_000_000] {
            assert_eq!(fast_mod_4294967291(x), x % MODULUS_32, "fast_mod_4294967291 failed for {}", x);
        }
    }
}

// ============================================================================
// Benchmarks
// ============================================================================

fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i & 0xFF) as u8).collect()
}

/// Configure benchmark timing: 0.5s warmup, 1.0s measurement
fn fast_config(group: &mut BenchmarkGroup<WallTime>) {
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));
}

fn bench_koopman16_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("Koopman16_Optimization");
    fast_config(&mut group);

    for size in [256, 1024, 4096] {
        let data = generate_test_data(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("original", size), &data, |b, data| {
            b.iter(|| koopman16_original(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("fast_mod", size), &data, |b, data| {
            b.iter(|| koopman16_fast_mod(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("barrett", size), &data, |b, data| {
            b.iter(|| koopman16_barrett(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("delayed", size), &data, |b, data| {
            b.iter(|| koopman16_delayed(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("unrolled", size), &data, |b, data| {
            b.iter(|| koopman16_unrolled(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("delayed_fast", size), &data, |b, data| {
            b.iter(|| koopman16_delayed_fast(black_box(data), 0))
        });
    }

    group.finish();
}

fn bench_koopman32_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("Koopman32_Optimization");
    fast_config(&mut group);

    for size in [256, 1024, 4096] {
        let data = generate_test_data(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("original", size), &data, |b, data| {
            b.iter(|| koopman32_original(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("fast_mod", size), &data, |b, data| {
            b.iter(|| koopman32_fast_mod(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("barrett", size), &data, |b, data| {
            b.iter(|| koopman32_barrett(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("delayed", size), &data, |b, data| {
            b.iter(|| koopman32_delayed(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("unrolled", size), &data, |b, data| {
            b.iter(|| koopman32_unrolled(black_box(data), 0))
        });

        group.bench_with_input(BenchmarkId::new("delayed_fast", size), &data, |b, data| {
            b.iter(|| koopman32_delayed_fast(black_box(data), 0))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_koopman16_variants, bench_koopman32_variants);
criterion_main!(benches);
