//! Comparison tests between the Rust implementation and the reference C algorithm.
//!
//! This module implements the reference C algorithm from Koopman's paper exactly,
//! then compares outputs to verify correctness.

use koopman_checksum::*;

// ============================================================================
// Reference C Implementation (translated to Rust)
// ============================================================================

/// Reference Koopman8 - matches the C code exactly
fn reference_koopman8(data: &[u8], initial_seed: u8, modulus: u32) -> u8 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = data[0] as u32 ^ initial_seed as u32;

    for &byte in &data[1..] {
        // Note: C reference uses | but + is equivalent when lower bits are 0
        sum = ((sum << 8) | byte as u32) % modulus;
    }

    // Append implicit zero
    sum = (sum << 8) % modulus;

    sum as u8
}

/// Reference Koopman16 - matches the C code exactly
fn reference_koopman16(data: &[u8], initial_seed: u8, modulus: u32) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = initial_seed as u32 ^ data[0] as u32;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % modulus;
    }

    // Append two bytes of implicit zeros
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;

    sum as u16
}

/// Reference Koopman32 - matches the C code
/// Note: The C reference uses (sum<<32) which would overflow u64.
/// We use sequential shifts which is mathematically equivalent and safe.
fn reference_koopman32(data: &[u8], initial_seed: u8, modulus: u64) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = initial_seed as u64 ^ data[0] as u64;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u64) % modulus;
    }

    // Append four bytes of implicit zeros
    // C reference does (sum<<32) % modulus but that overflows u64
    // Sequential shifts are equivalent and safe
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;

    sum as u32
}

/// Compute parity of a byte
fn parity(x: u8) -> u8 {
    (x.count_ones() & 1) as u8
}

/// Reference Koopman16P - matches the C code exactly
fn reference_koopman16p(data: &[u8], initial_seed: u8, modulus: u32) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = initial_seed as u32 ^ data[0] as u32;
    let mut psum: u32 = sum; // Initialize parity sum

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % modulus;
        psum ^= byte as u32;
    }

    // Append two bytes of implicit zeros
    // C reference uses (sum<<16) % modulus
    sum = (sum << 16) % modulus;

    // Pack sum with parity - parity is ONLY of data bytes, NOT checksum
    let result = (sum << 1) | parity(psum as u8) as u32;

    result as u16
}

/// Reference Koopman8P
fn reference_koopman8p(data: &[u8], initial_seed: u8, modulus: u32) -> u8 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = data[0] as u32 ^ initial_seed as u32;
    let mut psum: u8 = sum as u8;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % modulus;
        psum ^= byte;
    }

    // Append implicit zero byte
    sum = (sum << 8) % modulus;

    // Pack: checksum in upper 7 bits, parity in LSB
    // Parity is ONLY of data bytes, NOT checksum
    ((sum as u8) << 1) | parity(psum)
}

/// Reference Koopman32P
fn reference_koopman32p(data: &[u8], initial_seed: u8, modulus: u64) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = initial_seed as u64 ^ data[0] as u64;
    let mut psum: u32 = sum as u32;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u64) % modulus;
        psum ^= byte as u32;
    }

    // Append four bytes of implicit zeros
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;

    // Pack sum with parity - parity is ONLY of data bytes, NOT checksum
    let result = (sum << 1) | parity(psum as u8) as u64;

    result as u32
}

// ============================================================================
// Comparison Tests
// ============================================================================

const TEST_VECTORS: &[&[u8]] = &[
    b"",
    b"a",
    b"ab",
    b"abc",
    b"123456789",
    b"Hello, World!",
    b"The quick brown fox jumps over the lazy dog",
    &[0x00],
    &[0xFF],
    &[0x00, 0x00, 0x00, 0x00],
    &[0xFF, 0xFF, 0xFF, 0xFF],
    &[0x12, 0x34, 0x56],
    &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
];

#[test]
fn test_koopman8_matches_reference() {
    for (i, data) in TEST_VECTORS.iter().enumerate() {
        let reference = reference_koopman8(data, 0, MODULUS_8);
        let implementation = koopman8(data, 0);

        assert_eq!(
            reference, implementation,
            "Koopman8 mismatch for test vector {}: {:?}\n  Reference: 0x{:02X}\n  Implementation: 0x{:02X}",
            i, data, reference, implementation
        );
    }
}

#[test]
fn test_koopman8_with_seed_matches_reference() {
    for seed in [0u8, 1, 42, 127, 255] {
        for data in TEST_VECTORS.iter() {
            let reference = reference_koopman8(data, seed, MODULUS_8);
            let implementation = koopman8(data, seed);

            assert_eq!(
                reference, implementation,
                "Koopman8 mismatch with seed {} for data {:?}",
                seed, data
            );
        }
    }
}

#[test]
fn test_koopman16_matches_reference() {
    for (i, data) in TEST_VECTORS.iter().enumerate() {
        let reference = reference_koopman16(data, 0, MODULUS_16);
        let implementation = koopman16(data, 0);

        assert_eq!(
            reference, implementation,
            "Koopman16 mismatch for test vector {}: {:?}\n  Reference: 0x{:04X}\n  Implementation: 0x{:04X}",
            i, data, reference, implementation
        );
    }
}

#[test]
fn test_koopman16_with_seed_matches_reference() {
    for seed in [0u8, 1, 42, 127, 255] {
        for data in TEST_VECTORS.iter() {
            let reference = reference_koopman16(data, seed, MODULUS_16);
            let implementation = koopman16(data, seed);

            assert_eq!(
                reference, implementation,
                "Koopman16 mismatch with seed {} for data {:?}",
                seed, data
            );
        }
    }
}

#[test]
fn test_koopman32_matches_reference() {
    for (i, data) in TEST_VECTORS.iter().enumerate() {
        let reference = reference_koopman32(data, 0, MODULUS_32);
        let implementation = koopman32(data, 0);

        assert_eq!(
            reference, implementation,
            "Koopman32 mismatch for test vector {}: {:?}\n  Reference: 0x{:08X}\n  Implementation: 0x{:08X}",
            i, data, reference, implementation
        );
    }
}

#[test]
fn test_koopman32_with_seed_matches_reference() {
    for seed in [0u8, 1, 42, 127, 255] {
        for data in TEST_VECTORS.iter() {
            let reference = reference_koopman32(data, seed, MODULUS_32);
            let implementation = koopman32(data, seed);

            assert_eq!(
                reference, implementation,
                "Koopman32 mismatch with seed {} for data {:?}",
                seed, data
            );
        }
    }
}

// ============================================================================
// Parity Variant Comparison Tests
// These are expected to FAIL if the implementation differs from reference
// ============================================================================

#[test]
fn test_koopman8p_vs_reference() {
    println!("\n=== Koopman8P Comparison ===");
    let mut mismatches = 0;

    for (i, data) in TEST_VECTORS.iter().enumerate() {
        if data.is_empty() {
            continue;
        }

        let reference = reference_koopman8p(data, 0, MODULUS_7P);
        let implementation = koopman8p(data, 0);

        if reference != implementation {
            mismatches += 1;
            println!(
                "Koopman8P MISMATCH for test vector {}: {:?}",
                i,
                String::from_utf8_lossy(data)
            );
            println!("  Reference:      0x{:02X} (checksum: 0x{:02X}, parity: {})",
                reference, reference >> 1, reference & 1);
            println!("  Implementation: 0x{:02X} (checksum: 0x{:02X}, parity: {})",
                implementation, implementation >> 1, implementation & 1);
        }
    }

    if mismatches > 0 {
        panic!(
            "Koopman8P has {} mismatches with reference implementation!\n\
             The Rust implementation includes checksum in parity calculation,\n\
             but the reference C code does not.",
            mismatches
        );
    }
}

#[test]
fn test_koopman16p_vs_reference() {
    println!("\n=== Koopman16P Comparison ===");
    let mut mismatches = 0;

    for (i, data) in TEST_VECTORS.iter().enumerate() {
        if data.is_empty() {
            continue;
        }

        let reference = reference_koopman16p(data, 0, MODULUS_15P);
        let implementation = koopman16p(data, 0);

        if reference != implementation {
            mismatches += 1;
            println!(
                "Koopman16P MISMATCH for test vector {}: {:?}",
                i,
                String::from_utf8_lossy(data)
            );
            println!("  Reference:      0x{:04X} (checksum: 0x{:04X}, parity: {})",
                reference, reference >> 1, reference & 1);
            println!("  Implementation: 0x{:04X} (checksum: 0x{:04X}, parity: {})",
                implementation, implementation >> 1, implementation & 1);
        }
    }

    if mismatches > 0 {
        panic!(
            "Koopman16P has {} mismatches with reference implementation!\n\
             The Rust implementation includes checksum in parity calculation,\n\
             but the reference C code does not.",
            mismatches
        );
    }
}

#[test]
fn test_koopman32p_vs_reference() {
    println!("\n=== Koopman32P Comparison ===");
    let mut mismatches = 0;

    for (i, data) in TEST_VECTORS.iter().enumerate() {
        if data.is_empty() {
            continue;
        }

        let reference = reference_koopman32p(data, 0, MODULUS_31P);
        let implementation = koopman32p(data, 0);

        if reference != implementation {
            mismatches += 1;
            println!(
                "Koopman32P MISMATCH for test vector {}: {:?}",
                i,
                String::from_utf8_lossy(data)
            );
            println!("  Reference:      0x{:08X} (checksum: 0x{:08X}, parity: {})",
                reference, reference >> 1, reference & 1);
            println!("  Implementation: 0x{:08X} (checksum: 0x{:08X}, parity: {})",
                implementation, implementation >> 1, implementation & 1);
        }
    }

    if mismatches > 0 {
        panic!(
            "Koopman32P has {} mismatches with reference implementation!\n\
             The Rust implementation includes checksum in parity calculation,\n\
             but the reference C code does not.",
            mismatches
        );
    }
}

// ============================================================================
// Additional verification: manual calculation
// ============================================================================

#[test]
fn test_manual_koopman8_calculation() {
    // Input: [0x12, 0x34, 0x56] with seed 0, modulus 253
    // Step 1: sum = 0x12 = 18
    // Step 2: sum = ((18 << 8) | 0x34) % 253 = (4608 + 52) % 253 = 4660 % 253 = 106
    // Step 3: sum = ((106 << 8) | 0x56) % 253 = (27136 + 86) % 253 = 27222 % 253 = 151
    // Final:  sum = (151 << 8) % 253 = 38656 % 253 = 200

    let data = [0x12u8, 0x34, 0x56];

    // Verify manual calculation
    let mut sum: u32 = 0x12;
    sum = ((sum << 8) | 0x34) % 253;
    assert_eq!(sum, 106, "Step 2 failed");
    sum = ((sum << 8) | 0x56) % 253;
    assert_eq!(sum, 151, "Step 3 failed");
    sum = (sum << 8) % 253;
    assert_eq!(sum, 200, "Final step failed");

    // Verify implementation
    assert_eq!(koopman8(&data, 0), 200);
    assert_eq!(reference_koopman8(&data, 0, 253), 200);
}

#[test]
fn test_manual_koopman16_calculation() {
    // Input: [0x12, 0x34] with seed 0, modulus 65519
    // Step 1: sum = 0x12 = 18
    // Step 2: sum = ((18 << 8) + 0x34) % 65519 = 4660 % 65519 = 4660
    // Final:  sum = (4660 << 8) % 65519 = 1192960 % 65519 = 13618
    //         sum = (13618 << 8) % 65519 = 3486208 % 65519 = 13701

    let data = [0x12u8, 0x34];

    let mut sum: u32 = 0x12;
    sum = ((sum << 8) + 0x34) % 65519;
    assert_eq!(sum, 4660, "Step 2 failed");
    sum = (sum << 8) % 65519;
    assert_eq!(sum, 13618, "First finalization shift failed");
    sum = (sum << 8) % 65519;
    assert_eq!(sum, 13701, "Second finalization shift failed");

    assert_eq!(koopman16(&data, 0), 13701);
    assert_eq!(reference_koopman16(&data, 0, 65519), 13701);
}

// ============================================================================
// Verify finalization equivalence
// ============================================================================

#[test]
fn test_finalization_equivalence() {
    // Verify that sequential shifts produce the same result as a single large shift
    // (when the large shift doesn't overflow)

    // For 16-bit: (sum << 16) % mod == ((sum << 8) % mod << 8) % mod
    for sum in [0u32, 1, 100, 1000, 10000, 32000, 65000] {
        let single_shift = (sum << 16) % MODULUS_16;
        let double_shift = {
            let tmp = (sum << 8) % MODULUS_16;
            (tmp << 8) % MODULUS_16
        };
        assert_eq!(
            single_shift, double_shift,
            "Finalization equivalence failed for sum={}", sum
        );
    }
}
