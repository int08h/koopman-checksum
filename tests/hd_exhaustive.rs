//! Exhaustive error detection verification tests for Koopman checksums.
//!
//! ## Understanding Hamming Distance (HD)
//!
//! - **HD=3**: Detects all 1-bit and 2-bit errors (but NOT all 3-bit errors)
//! - **HD=4**: Detects all 1-bit, 2-bit, and 3-bit errors (but NOT all 4-bit errors)
//!
//! These tests exhaustively verify these detection guarantees by testing all
//! possible error patterns in the data portion.
//!
//! # Test Organization
//!
//! - **8-bit tests**: Test ALL lengths from 1 byte up to max length, complete in seconds
//! - **16-bit tests**: Test at max length only, complete in hours to days
//!
//! Each test runs with both all-zero data and non-zero pattern data.
//!
//! # Available Tests
//!
//! | Name | Description | Run Time (AMD 9950X) |
//! |------|-------------|------|
//! | `koopman8_hd3_exhaustive` | koopman8 all lengths 1-13, verifies all 1-2 bit errors detected | seconds |
//! | `koopman8p_hd4_exhaustive` | koopman8p all lengths 1-5, verifies all 1-3 bit errors detected | seconds |
//! | `koopman16_hd3_exhaustive` | koopman16 at 4092 bytes, verifies all 1-2 bit errors detected | ~1 day |
//! | `koopman16p_hd4_exhaustive` | koopman16p at 2044 bytes, verifies all 1-3 bit errors detected | week+ |
//! | `hd_quick_sanity` | Quick sanity check of all variants | instant |
//!
//! # Running Tests
//!
//! ```bash
//! # Run all tests (warning: 16-bit tests take hours/days)
//! cargo test --release --test hd_exhaustive -- --nocapture
//!
//! # Run only 8-bit exhaustive tests (fast)
//! cargo test --release --test hd_exhaustive -- koopman8 --nocapture
//!
//! # Run specific test by name
//! cargo test --release --test hd_exhaustive -- koopman16_hd3_exhaustive --nocapture
//! ```

use koopman_checksum::{koopman8, koopman8p, koopman16, koopman16p};
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Maximum data length for koopman8 to detect all 1-2 bit errors
const MAX_LEN_8: usize = 13;

/// Maximum data length for koopman16 to detect all 1-2 bit errors
const MAX_LEN_16: usize = 4092;

/// Maximum data length for koopman8p to detect all 1-3 bit errors
const MAX_LEN_8P: usize = 5;

/// Maximum data length for koopman16p to detect all 1-3 bit errors
const MAX_LEN_16P: usize = 2044;

/// Generate all-zero test data of given length.
fn generate_zeros(len: usize) -> Vec<u8> {
    vec![0; len]
}

/// Generate pattern test data of given length.
fn generate_pattern(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i.wrapping_mul(7).wrapping_add(13)) as u8).collect()
}

/// Flip a single bit in the data at the given bit position.
#[inline]
fn flip_bit(data: &mut [u8], bit_pos: usize) {
    let byte_idx = bit_pos / 8;
    let bit_idx = bit_pos % 8;
    data[byte_idx] ^= 1 << bit_idx;
}

/// Verify all 1-bit errors in data are detected.
fn verify_1bit<F, C>(name: &str, seed: u8, data: &[u8], checksum_fn: &F) -> bool
where
    F: Fn(&[u8], u8) -> C,
    C: Eq + std::fmt::Debug,
{
    let original = checksum_fn(data, seed);
    let total_bits = data.len() * 8;

    for bit in 0..total_bits {
        let mut corrupted = data.to_vec();
        flip_bit(&mut corrupted, bit);
        if checksum_fn(&corrupted, seed) == original {
            eprintln!(
                "{} FAILED: seed={:#04x}, 1-bit error at bit {} not detected",
                name, seed, bit
            );
            return false;
        }
    }
    true
}

/// Verify all 2-bit errors in data are detected.
fn verify_2bit<F, C>(
    name: &str,
    seed: u8,
    data: &[u8],
    checksum_fn: &F,
    progress: &AtomicU64,
) -> bool
where
    F: Fn(&[u8], u8) -> C,
    C: Eq + std::fmt::Debug,
{
    let original = checksum_fn(data, seed);
    let total_bits = data.len() * 8;

    for bit1 in 0..total_bits {
        for bit2 in (bit1 + 1)..total_bits {
            let mut corrupted = data.to_vec();
            flip_bit(&mut corrupted, bit1);
            flip_bit(&mut corrupted, bit2);
            if checksum_fn(&corrupted, seed) == original {
                eprintln!(
                    "{} FAILED: seed={:#04x}, 2-bit error at bits {},{} not detected",
                    name, seed, bit1, bit2
                );
                return false;
            }
        }
        progress.fetch_add((total_bits - bit1 - 1) as u64, Ordering::Relaxed);
    }
    true
}

/// Verify all 3-bit errors in data are detected.
fn verify_3bit<F, C>(
    name: &str,
    seed: u8,
    data: &[u8],
    checksum_fn: &F,
    progress: &AtomicU64,
) -> bool
where
    F: Fn(&[u8], u8) -> C,
    C: Eq + std::fmt::Debug,
{
    let original = checksum_fn(data, seed);
    let total_bits = data.len() * 8;

    for bit1 in 0..total_bits {
        for bit2 in (bit1 + 1)..total_bits {
            for bit3 in (bit2 + 1)..total_bits {
                let mut corrupted = data.to_vec();
                flip_bit(&mut corrupted, bit1);
                flip_bit(&mut corrupted, bit2);
                flip_bit(&mut corrupted, bit3);
                if checksum_fn(&corrupted, seed) == original {
                    eprintln!(
                        "{} FAILED: seed={:#04x}, 3-bit error at bits {},{},{} not detected",
                        name, seed, bit1, bit2, bit3
                    );
                    return false;
                }
            }
        }
        let remaining = total_bits - bit1 - 1;
        if remaining >= 2 {
            progress.fetch_add((remaining * (remaining - 1) / 2) as u64, Ordering::Relaxed);
        }
    }
    true
}

/// Run an HD test with progress reporting.
fn run_hd_test<F, C>(
    name: &str,
    data: &[u8],
    pattern_name: &str,
    max_errors: usize,
    checksum_fn: F,
) where
    F: Fn(&[u8], u8) -> C + Send + Sync,
    C: Eq + std::fmt::Debug + Send,
{
    let data_len = data.len();
    let total_bits = data_len * 8;

    let c1 = total_bits as u64;
    let c2 = (total_bits as u64 * (total_bits as u64 - 1)) / 2;
    let c3 = if total_bits >= 3 {
        (total_bits as u64 * (total_bits as u64 - 1) * (total_bits as u64 - 2)) / 6
    } else {
        0
    };

    let tests_per_seed = match max_errors {
        1 => c1,
        2 => c1 + c2,
        3 => c1 + c2 + c3,
        _ => panic!("max_errors must be 1, 2, or 3"),
    };
    let total_tests = tests_per_seed * 256;

    println!("\n=== {} HD={} Test ({}) ===", name, max_errors + 1, pattern_name);
    println!("Data length: {} bytes ({} bits)", data_len, total_bits);
    println!(
        "Error patterns: 1-bit={}, 2-bit={}{}",
        c1,
        c2,
        if max_errors >= 3 {
            format!(", 3-bit={}", c3)
        } else {
            String::new()
        }
    );
    println!(
        "Total tests: {} ({:.2}B)",
        total_tests,
        total_tests as f64 / 1e9
    );

    let start = Instant::now();
    let failed = AtomicU64::new(0);
    let completed_seeds = AtomicU64::new(0);
    let tests_completed = AtomicU64::new(0);

    (0u8..=255).into_par_iter().for_each(|seed| {
        // 1-bit errors
        if !verify_1bit(name, seed, data, &checksum_fn) {
            failed.fetch_add(1, Ordering::Relaxed);
            return;
        }
        tests_completed.fetch_add(c1, Ordering::Relaxed);

        // 2-bit errors
        if max_errors >= 2 {
            if !verify_2bit(name, seed, data, &checksum_fn, &tests_completed) {
                failed.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }

        // 3-bit errors
        if max_errors >= 3 {
            if !verify_3bit(name, seed, data, &checksum_fn, &tests_completed) {
                failed.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }

        let done = completed_seeds.fetch_add(1, Ordering::Relaxed) + 1;
        if done % 8 == 0 || done == 256 {
            let tests_done = tests_completed.load(Ordering::Relaxed);
            let pct = 100.0 * tests_done as f64 / total_tests as f64;
            let elapsed = start.elapsed().as_secs_f64();
            let rate = tests_done as f64 / elapsed / 1e6;
            println!(
                "  {}/256 seeds ({:.1}%), {:.1}M tests/sec",
                done, pct, rate
            );
        }
    });

    let elapsed = start.elapsed();
    let fail_count = failed.load(Ordering::Relaxed);

    if fail_count == 0 {
        println!(
            "{} HD={} ({}): PASSED in {:.2}s ({:.1}M tests/sec)",
            name,
            max_errors + 1,
            pattern_name,
            elapsed.as_secs_f64(),
            total_tests as f64 / elapsed.as_secs_f64() / 1e6
        );
    } else {
        panic!(
            "{} HD={} ({}) verification FAILED for {} seeds",
            name,
            max_errors + 1,
            pattern_name,
            fail_count
        );
    }
}

/// Run HD tests with multiple data patterns at a single length.
fn run_hd_tests_multi_pattern<F, C>(
    name: &str,
    data_len: usize,
    max_errors: usize,
    checksum_fn: F,
) where
    F: Fn(&[u8], u8) -> C + Send + Sync + Copy,
    C: Eq + std::fmt::Debug + Send,
{
    for (pattern_name, data) in [
        ("zeros", generate_zeros(data_len)),
        ("pattern", generate_pattern(data_len)),
    ] {
        run_hd_test(name, &data, pattern_name, max_errors, checksum_fn);
    }
}

/// Run HD tests for ALL lengths from 1 to max_len (inclusive).
fn run_hd_tests_all_lengths<F, C>(
    name: &str,
    max_len: usize,
    max_errors: usize,
    checksum_fn: F,
) where
    F: Fn(&[u8], u8) -> C + Send + Sync + Copy,
    C: Eq + std::fmt::Debug + Send,
{
    println!("\n########## {} HD={} All Lengths Test (1-{} bytes) ##########",
             name, max_errors + 1, max_len);

    let overall_start = Instant::now();

    for data_len in 1..=max_len {
        for (pattern_name, data) in [
            ("zeros", generate_zeros(data_len)),
            ("pattern", generate_pattern(data_len)),
        ] {
            run_hd_test(name, &data, pattern_name, max_errors, checksum_fn);
        }
    }

    let overall_elapsed = overall_start.elapsed();
    println!(
        "\n########## {} HD={} All Lengths: COMPLETE in {:.2}s ##########",
        name,
        max_errors + 1,
        overall_elapsed.as_secs_f64()
    );
}

// koopman8 HD=3 exhaustive test (all lengths 1-13 bytes)
#[test]
fn koopman8_hd3_exhaustive() {
    run_hd_tests_all_lengths("koopman8", MAX_LEN_8, 2, koopman8);
}

// koopman8p HD=4 exhaustive test (all lengths 1-5 bytes)
#[test]
fn koopman8p_hd4_exhaustive() {
    run_hd_tests_all_lengths("koopman8p", MAX_LEN_8P, 3, koopman8p);
}

// koopman16 HD=3 exhaustive test (max length: 4092 bytes)
// WARNING: This test takes several hours to complete
#[test]
fn koopman16_hd3_exhaustive() {
    run_hd_tests_multi_pattern("koopman16", MAX_LEN_16, 2, koopman16);
}

// koopman16p HD=4 exhaustive test (max length: 2044 bytes)
// WARNING: This test takes days to complete due to 3-bit combinations
// At 2044 bytes (16352 bits): C(16352,3) ≈ 729 billion 3-bit tests per seed
// Total 3-bit tests: ~187 trillion
#[test]
fn koopman16p_hd4_exhaustive() {
    run_hd_tests_multi_pattern("koopman16p", MAX_LEN_16P, 3, koopman16p);
}

#[test]
fn hd_quick_sanity() {
    println!("\n=== Quick HD Sanity Check ===");

    for seed in [0u8, 1, 127, 128, 255] {
        // Test with zeros and pattern data
        for data8 in [generate_zeros(MAX_LEN_8), generate_pattern(MAX_LEN_8)] {
            let orig8 = koopman8(&data8, seed);

            // 1-bit error
            let mut corrupted = data8.clone();
            corrupted[0] ^= 1;
            assert_ne!(koopman8(&corrupted, seed), orig8, "koopman8 1-bit failed");

            // 2-bit error
            let mut corrupted = data8.clone();
            corrupted[0] ^= 1;
            corrupted[1] ^= 2;
            assert_ne!(koopman8(&corrupted, seed), orig8, "koopman8 2-bit failed");
        }

        for data8p in [generate_zeros(MAX_LEN_8P), generate_pattern(MAX_LEN_8P)] {
            let orig8p = koopman8p(&data8p, seed);

            // 3-bit error
            let mut corrupted = data8p.clone();
            corrupted[0] ^= 1;
            corrupted[1] ^= 2;
            corrupted[2] ^= 4;
            assert_ne!(koopman8p(&corrupted, seed), orig8p, "koopman8p 3-bit failed");
        }

        for data16 in [generate_zeros(64), generate_pattern(64)] {
            let orig16 = koopman16(&data16, seed);

            // 2-bit error
            let mut corrupted = data16.clone();
            corrupted[0] ^= 1;
            corrupted[32] ^= 0x80;
            assert_ne!(koopman16(&corrupted, seed), orig16, "koopman16 2-bit failed");
        }

        for data16p in [generate_zeros(32), generate_pattern(32)] {
            let orig16p = koopman16p(&data16p, seed);

            // 3-bit error
            let mut corrupted = data16p.clone();
            corrupted[0] ^= 1;
            corrupted[8] ^= 2;
            corrupted[16] ^= 4;
            assert_ne!(koopman16p(&corrupted, seed), orig16p, "koopman16p 3-bit failed");
        }
    }

    println!("Quick sanity check: PASSED");
}
