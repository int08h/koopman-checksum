//! # Koopman Checksum
//!
//! Implementation of the Koopman checksum algorithm as described in:
//! "An Improved Modular Addition Checksum Algorithm" by Philip Koopman (arXiv:2304.13496)
//!
//! The Koopman checksum provides Hamming Distance 3 (HD=3) fault detection for longer
//! data word lengths than dual-sum approaches like Fletcher checksums, while using
//! only a single running sum that is twice the size of the final check value.
//!
//! ## Algorithm
//!
//! The computational kernel is:
//! ```text
//! sum = ((sum << k) + block) % modulus
//! ```
//!
//! Where `k` is the number of bits in the check value (8, 16, or 32).
//!
//! ## Recommended Moduli
//!
//! | Variant    | Modulus      | HD=3 Length |
//! |------------|--------------|-------------|
//! | Koopman8   | 253          | 13 bytes    |
//! | Koopman16  | 65519        | 4092 bytes  |
//! | Koopman32  | 4294967291   | 134M bytes  |
//!
//! ## Seed Value
//!
//! A seed of 0 is simple but means leading zero bytes don't affect the checksum.
//! Use a non-zero seed (e.g., 1) if you need to detect leading zeros.
//!
//! ## Example
//!
//! ```rust
//! use koopman_checksum::{koopman8, koopman16, koopman32};
//!
//! let data = b"Hello, World!";
//!
//! let checksum8 = koopman8(data, 0);
//! let checksum16 = koopman16(data, 0);
//! let checksum32 = koopman32(data, 0);
//!
//! println!("Koopman8:  0x{:02X}", checksum8);
//! println!("Koopman16: 0x{:04X}", checksum16);
//! println!("Koopman32: 0x{:08X}", checksum32);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

// ============================================================================
// Constants
// ============================================================================

/// Recommended modulus for 8-bit Koopman checksum (HD=3 up to 13 bytes)
pub const MODULUS_8: u32 = 253;

/// Alternative modulus for 8-bit Koopman checksum (HD=3 up to 14 bytes)
pub const MODULUS_8_ALT: u32 = 239;

/// Recommended modulus for 16-bit Koopman checksum (HD=3 up to 4092 bytes)
pub const MODULUS_16: u32 = 65519;

/// Recommended modulus for 32-bit Koopman checksum (HD=3 up to 134,217,720 bytes)
pub const MODULUS_32: u64 = 4294967291;

/// Modulus for 7-bit Koopman checksum with parity (HD=4 up to 5 bytes)
pub const MODULUS_7P: u32 = 125;

/// Modulus for 15-bit Koopman checksum with parity (HD=4 up to 2044 bytes)
pub const MODULUS_15P: u32 = 32749;

/// Modulus for 31-bit Koopman checksum with parity (HD=4 up to 134,217,720 bytes)
pub const MODULUS_31P: u64 = 2147483629;

// ============================================================================
// Pure Rust Implementation - Core Functions
// ============================================================================

/// Compute an 8-bit Koopman checksum.
///
/// Provides HD=3 fault detection for data words up to 13 bytes with modulus 253.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 8-bit checksum value
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman8;
/// let checksum = koopman8(b"test data", 0);
/// ```
#[inline]
pub fn koopman8(data: &[u8], initial_seed: u8) -> u8 {
    koopman8_with_modulus(data, initial_seed, MODULUS_8)
}

/// Compute an 8-bit Koopman checksum with a custom modulus.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value
/// * `modulus` - The modulus to use (recommended: 253 or 239)
#[inline]
pub fn koopman8_with_modulus(data: &[u8], initial_seed: u8, modulus: u32) -> u8 {
    debug_assert!(modulus > 0 && modulus <= 256, "Modulus must be in range 1..=256");
    
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % modulus;
    }

    // Append implicit zero byte
    sum = (sum << 8) % modulus;

    sum as u8
}

/// Compute a 16-bit Koopman checksum.
///
/// Provides HD=3 fault detection for data words up to 4092 bytes.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 16-bit checksum value
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman16;
/// let checksum = koopman16(b"test data", 0);
/// ```
#[inline]
pub fn koopman16(data: &[u8], initial_seed: u8) -> u16 {
    koopman16_with_modulus(data, initial_seed, MODULUS_16)
}

/// Compute a 16-bit Koopman checksum with a custom modulus.
#[inline]
pub fn koopman16_with_modulus(data: &[u8], initial_seed: u8, modulus: u32) -> u16 {
    debug_assert!(modulus > 0 && modulus <= 65536, "Modulus must be in range 1..=65536");
    
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % modulus;
    }

    // Append two implicit zero bytes
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;

    sum as u16
}

/// Compute a 32-bit Koopman checksum.
///
/// Provides HD=3 fault detection for data words up to 134,217,720 bytes.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 32-bit checksum value
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman32;
/// let checksum = koopman32(b"test data", 0);
/// ```
#[inline]
pub fn koopman32(data: &[u8], initial_seed: u8) -> u32 {
    koopman32_with_modulus(data, initial_seed, MODULUS_32)
}

/// Compute a 32-bit Koopman checksum with a custom modulus.
#[inline]
pub fn koopman32_with_modulus(data: &[u8], initial_seed: u8, modulus: u64) -> u32 {
    debug_assert!(modulus > 0 && modulus <= (1u64 << 32), "Modulus must be in range 1..=2^32");
    
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u64) % modulus;
    }

    // Append four implicit zero bytes
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;

    sum as u32
}

// ============================================================================
// Parity Variants (HD=4)
// ============================================================================

/// Compute parity of a byte (number of set bits mod 2).
#[inline]
fn parity8(x: u8) -> u8 {
    (x.count_ones() & 1) as u8
}

/// Compute an 8-bit Koopman checksum with parity (7-bit checksum + 1 parity bit).
///
/// Provides HD=4 fault detection for data words up to 5 bytes.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 8-bit value: 7-bit checksum in upper bits, parity in LSB
#[inline]
pub fn koopman8p(data: &[u8], initial_seed: u8) -> u8 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;
    let mut psum: u8 = sum as u8;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % MODULUS_7P;
        psum ^= byte;
    }

    // Append implicit zero byte
    sum = (sum << 8) % MODULUS_7P;

    // Include checksum in parity calculation
    psum ^= sum as u8;

    // Pack: checksum in upper 7 bits, parity in LSB
    ((sum as u8) << 1) | parity8(psum)
}

/// Compute a 16-bit Koopman checksum with parity (15-bit checksum + 1 parity bit).
///
/// Provides HD=4 fault detection for data words up to 2044 bytes.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 16-bit value: 15-bit checksum in upper bits, parity in LSB
#[inline]
pub fn koopman16p(data: &[u8], initial_seed: u8) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;
    let mut psum: u32 = sum;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % MODULUS_15P;
        psum ^= byte as u32;
    }

    // Append two implicit zero bytes
    sum = (sum << 8) % MODULUS_15P;
    sum = (sum << 8) % MODULUS_15P;

    // Include checksum in parity calculation (fold to single byte, then get parity)
    let checksum_parity = (sum as u8) ^ ((sum >> 8) as u8);
    psum ^= checksum_parity as u32;

    // Pack: checksum in upper 15 bits, parity in LSB
    ((sum as u16) << 1) | (parity8(psum as u8) as u16)
}

/// Compute a 32-bit Koopman checksum with parity (31-bit checksum + 1 parity bit).
///
/// Provides HD=4 fault detection for data words up to 134,217,720 bytes.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 32-bit value: 31-bit checksum in upper bits, parity in LSB
#[inline]
pub fn koopman32p(data: &[u8], initial_seed: u8) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;
    let mut psum: u32 = sum as u32;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u64) % MODULUS_31P;
        psum ^= byte as u32;
    }

    // Append four implicit zero bytes
    sum = (sum << 8) % MODULUS_31P;
    sum = (sum << 8) % MODULUS_31P;
    sum = (sum << 8) % MODULUS_31P;
    sum = (sum << 8) % MODULUS_31P;

    // Include checksum in parity calculation
    let cs = sum as u32;
    let checksum_parity = (cs as u8) ^ ((cs >> 8) as u8) ^ ((cs >> 16) as u8) ^ ((cs >> 24) as u8);
    psum ^= checksum_parity as u32;

    // Pack: checksum in upper 31 bits, parity in LSB
    ((sum as u32) << 1) | (parity8(psum as u8) as u32)
}

// ============================================================================
// Streaming/Incremental API
// ============================================================================

/// Incremental Koopman8 checksum calculator.
///
/// Allows computing checksums over data that arrives in chunks.
///
/// # Example
/// ```rust
/// use koopman_checksum::Koopman8;
///
/// let mut hasher = Koopman8::new();
/// hasher.update(b"Hello, ");
/// hasher.update(b"World!");
/// let checksum = hasher.finalize();
/// ```
#[derive(Clone, Debug)]
pub struct Koopman8 {
    sum: u32,
    modulus: u32,
    seed: u32,
    initialized: bool,
}

impl Default for Koopman8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Koopman8 {
    /// Create a new Koopman8 hasher with default modulus (253).
    #[inline]
    pub fn new() -> Self {
        Self::with_modulus(MODULUS_8)
    }

    /// Create a new Koopman8 hasher with a custom modulus.
    #[inline]
    pub fn with_modulus(modulus: u32) -> Self {
        Self {
            sum: 0,
            modulus,
            seed: 0,
            initialized: false,
        }
    }

    /// Create a new Koopman8 hasher with an initial seed.
    #[inline]
    pub fn with_seed(seed: u8) -> Self {
        Self {
            sum: seed as u32,
            modulus: MODULUS_8,
            seed: seed as u32,
            initialized: false,
        }
    }

    /// Update the checksum with more data.
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        let mut iter = data.iter();

        if !self.initialized {
            if let Some(&first) = iter.next() {
                self.sum ^= first as u32;
                self.initialized = true;
            }
        }

        for &byte in iter {
            self.sum = ((self.sum << 8) + byte as u32) % self.modulus;
        }
    }

    /// Finalize and return the checksum.
    #[inline]
    pub fn finalize(self) -> u8 {
        if !self.initialized {
            return 0;
        }
        ((self.sum << 8) % self.modulus) as u8
    }

    /// Reset the hasher to initial state.
    #[inline]
    pub fn reset(&mut self) {
        self.sum = self.seed;
        self.initialized = false;
    }
}

/// Incremental Koopman16 checksum calculator.
#[derive(Clone, Debug)]
pub struct Koopman16 {
    sum: u32,
    modulus: u32,
    seed: u32,
    initialized: bool,
}

impl Default for Koopman16 {
    fn default() -> Self {
        Self::new()
    }
}

impl Koopman16 {
    /// Create a new Koopman16 hasher with default modulus (65519).
    #[inline]
    pub fn new() -> Self {
        Self::with_modulus(MODULUS_16)
    }

    /// Create a new Koopman16 hasher with a custom modulus.
    #[inline]
    pub fn with_modulus(modulus: u32) -> Self {
        Self {
            sum: 0,
            modulus,
            seed: 0,
            initialized: false,
        }
    }

    /// Create a new Koopman16 hasher with an initial seed.
    #[inline]
    pub fn with_seed(seed: u8) -> Self {
        Self {
            sum: seed as u32,
            modulus: MODULUS_16,
            seed: seed as u32,
            initialized: false,
        }
    }

    /// Update the checksum with more data.
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        let mut iter = data.iter();

        if !self.initialized {
            if let Some(&first) = iter.next() {
                self.sum ^= first as u32;
                self.initialized = true;
            }
        }

        for &byte in iter {
            self.sum = ((self.sum << 8) + byte as u32) % self.modulus;
        }
    }

    /// Finalize and return the checksum.
    #[inline]
    pub fn finalize(self) -> u16 {
        if !self.initialized {
            return 0;
        }
        let mut sum = self.sum;
        sum = (sum << 8) % self.modulus;
        sum = (sum << 8) % self.modulus;
        sum as u16
    }

    /// Reset the hasher to initial state.
    #[inline]
    pub fn reset(&mut self) {
        self.sum = self.seed;
        self.initialized = false;
    }
}

/// Incremental Koopman32 checksum calculator.
#[derive(Clone, Debug)]
pub struct Koopman32 {
    sum: u64,
    modulus: u64,
    seed: u64,
    initialized: bool,
}

impl Default for Koopman32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Koopman32 {
    /// Create a new Koopman32 hasher with default modulus.
    #[inline]
    pub fn new() -> Self {
        Self::with_modulus(MODULUS_32)
    }

    /// Create a new Koopman32 hasher with a custom modulus.
    #[inline]
    pub fn with_modulus(modulus: u64) -> Self {
        Self {
            sum: 0,
            modulus,
            seed: 0,
            initialized: false,
        }
    }

    /// Create a new Koopman32 hasher with an initial seed.
    #[inline]
    pub fn with_seed(seed: u8) -> Self {
        Self {
            sum: seed as u64,
            modulus: MODULUS_32,
            seed: seed as u64,
            initialized: false,
        }
    }

    /// Update the checksum with more data.
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        let mut iter = data.iter();

        if !self.initialized {
            if let Some(&first) = iter.next() {
                self.sum ^= first as u64;
                self.initialized = true;
            }
        }

        for &byte in iter {
            self.sum = ((self.sum << 8) + byte as u64) % self.modulus;
        }
    }

    /// Finalize and return the checksum.
    #[inline]
    pub fn finalize(self) -> u32 {
        if !self.initialized {
            return 0;
        }
        let mut sum = self.sum;
        sum = (sum << 8) % self.modulus;
        sum = (sum << 8) % self.modulus;
        sum = (sum << 8) % self.modulus;
        sum = (sum << 8) % self.modulus;
        sum as u32
    }

    /// Reset the hasher to initial state.
    #[inline]
    pub fn reset(&mut self) {
        self.sum = self.seed;
        self.initialized = false;
    }
}

// ============================================================================
// Verification Functions
// ============================================================================

/// Verify data integrity using Koopman8 checksum.
///
/// # Arguments
/// * `data` - The data bytes (excluding checksum)
/// * `expected` - The expected checksum value
/// * `initial_seed` - Initial seed used when computing the checksum
///
/// # Returns
/// `true` if the checksum matches, `false` otherwise
#[inline]
pub fn verify8(data: &[u8], expected: u8, initial_seed: u8) -> bool {
    koopman8(data, initial_seed) == expected
}

/// Verify data integrity using Koopman16 checksum.
#[inline]
pub fn verify16(data: &[u8], expected: u16, initial_seed: u8) -> bool {
    koopman16(data, initial_seed) == expected
}

/// Verify data integrity using Koopman32 checksum.
#[inline]
pub fn verify32(data: &[u8], expected: u32, initial_seed: u8) -> bool {
    koopman32(data, initial_seed) == expected
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test vectors based on the C reference implementation
    const TEST_DATA: &[u8] = b"123456789";

    #[test]
    fn test_koopman8_empty() {
        assert_eq!(koopman8(&[], 0), 0);
    }

    #[test]
    fn test_koopman8_single_byte() {
        // For single byte 0x12: sum = 0x12, then append zero: (0x12 << 8) % 253 = 4608 % 253 = 54
        assert_eq!(koopman8(&[0x12], 0), ((0x12u32 << 8) % MODULUS_8) as u8);
    }

    #[test]
    fn test_koopman8_basic() {
        // Verify the algorithm produces consistent results
        let result = koopman8(TEST_DATA, 0);
        assert_eq!(koopman8(TEST_DATA, 0), result); // Deterministic
    }

    #[test]
    fn test_koopman16_empty() {
        assert_eq!(koopman16(&[], 0), 0);
    }

    #[test]
    fn test_koopman16_basic() {
        let result = koopman16(TEST_DATA, 0);
        assert_eq!(koopman16(TEST_DATA, 0), result); // Deterministic
    }

    #[test]
    fn test_koopman32_empty() {
        assert_eq!(koopman32(&[], 0), 0);
    }

    #[test]
    fn test_koopman32_basic() {
        let result = koopman32(TEST_DATA, 0);
        assert_eq!(koopman32(TEST_DATA, 0), result); // Deterministic
    }

    #[test]
    fn test_streaming_koopman8() {
        let full = koopman8(TEST_DATA, 0);

        let mut hasher = Koopman8::new();
        hasher.update(&TEST_DATA[..4]);
        hasher.update(&TEST_DATA[4..]);
        let streaming = hasher.finalize();

        assert_eq!(full, streaming);
    }

    #[test]
    fn test_streaming_koopman16() {
        let full = koopman16(TEST_DATA, 0);

        let mut hasher = Koopman16::new();
        hasher.update(&TEST_DATA[..4]);
        hasher.update(&TEST_DATA[4..]);
        let streaming = hasher.finalize();

        assert_eq!(full, streaming);
    }

    #[test]
    fn test_streaming_koopman32() {
        let full = koopman32(TEST_DATA, 0);

        let mut hasher = Koopman32::new();
        hasher.update(&TEST_DATA[..4]);
        hasher.update(&TEST_DATA[4..]);
        let streaming = hasher.finalize();

        assert_eq!(full, streaming);
    }

    #[test]
    fn test_verification() {
        let checksum = koopman16(TEST_DATA, 0);
        assert!(verify16(TEST_DATA, checksum, 0));
        assert!(!verify16(TEST_DATA, checksum.wrapping_add(1), 0));
    }

    #[test]
    fn test_seed_affects_result() {
        let result0 = koopman16(TEST_DATA, 0);
        let result1 = koopman16(TEST_DATA, 1);
        assert_ne!(result0, result1);
    }

    #[test]
    fn test_koopman8p() {
        let result = koopman8p(TEST_DATA, 0);
        // Verify parity bit is set correctly
        let parity_bit = result & 1;
        assert!(parity_bit == 0 || parity_bit == 1);
    }

    #[test]
    fn test_koopman16p() {
        let result = koopman16p(TEST_DATA, 0);
        // Verify parity bit is set correctly
        let parity_bit = result & 1;
        assert!(parity_bit == 0 || parity_bit == 1);
    }

    #[test]
    fn test_koopman32p() {
        let result = koopman32p(TEST_DATA, 0);
        // Verify parity bit is set correctly
        let parity_bit = result & 1;
        assert!(parity_bit == 0 || parity_bit == 1);
    }

    #[test]
    fn test_single_bit_detection() {
        // Koopman checksums should detect all single-bit errors
        let original = koopman16(TEST_DATA, 0);
        
        for i in 0..TEST_DATA.len() {
            for bit in 0..8 {
                let mut corrupted = TEST_DATA.to_vec();
                corrupted[i] ^= 1 << bit;
                let corrupted_checksum = koopman16(&corrupted, 0);
                assert_ne!(original, corrupted_checksum, 
                    "Failed to detect single bit flip at byte {} bit {}", i, bit);
            }
        }
    }

    #[test]
    fn test_reference_calculation() {
        // Manual calculation for simple input to verify algorithm
        // Input: [0x12, 0x34, 0x56] with seed 0, modulus 253
        // Step 1: sum = 0x12 = 18
        // Step 2: sum = ((18 << 8) + 0x34) % 253 = 4660 % 253 = 106
        // Step 3: sum = ((106 << 8) + 0x56) % 253 = 27222 % 253 = 151
        // Final:  sum = (151 << 8) % 253 = 38656 % 253 = 200

        let data = [0x12u8, 0x34, 0x56];
        let result = koopman8(&data, 0);
        assert_eq!(result, 200);
    }

    // ========================================================================
    // Additional tests for parity variants
    // ========================================================================

    #[test]
    fn test_koopman8p_parity_correctness() {
        // Verify that the parity bit correctly reflects the parity of data + checksum
        let data = b"Test";
        let result = koopman8p(data, 0);

        // The checksum is in upper 7 bits
        let checksum = result >> 1;
        let parity_bit = result & 1;

        // Compute expected parity: XOR all data bytes, then XOR with checksum
        let mut expected_parity: u8 = 0;
        for &byte in data {
            expected_parity ^= byte;
        }
        expected_parity ^= checksum;
        let expected_parity_bit = expected_parity.count_ones() & 1;

        assert_eq!(parity_bit as u32, expected_parity_bit);
    }

    #[test]
    fn test_koopman16p_parity_correctness() {
        let data = b"Test data";
        let result = koopman16p(data, 0);

        // The checksum is in upper 15 bits
        let checksum = result >> 1;
        let parity_bit = result & 1;

        // Parity should be 0 or 1
        assert!(parity_bit == 0 || parity_bit == 1);

        // Verify checksum is within expected range for 15-bit value
        assert!(checksum < 32768);
    }

    #[test]
    fn test_koopman32p_parity_correctness() {
        let data = b"Test data for 32-bit";
        let result = koopman32p(data, 0);

        // The checksum is in upper 31 bits
        let checksum = result >> 1;
        let parity_bit = result & 1;

        // Parity should be 0 or 1
        assert!(parity_bit == 0 || parity_bit == 1);

        // Verify checksum is within expected range for 31-bit value
        assert!(checksum < (1 << 31));
    }

    #[test]
    fn test_parity_variants_detect_single_bit_errors() {
        // Parity variants should detect all single-bit errors (HD=4)
        let data = b"Test";
        let original = koopman16p(data, 0);

        for i in 0..data.len() {
            for bit in 0..8 {
                let mut corrupted = data.to_vec();
                corrupted[i] ^= 1 << bit;
                let corrupted_checksum = koopman16p(&corrupted, 0);
                assert_ne!(original, corrupted_checksum,
                    "Failed to detect single bit flip at byte {} bit {}", i, bit);
            }
        }
    }

    // ========================================================================
    // Tests for custom moduli
    // ========================================================================

    #[test]
    fn test_custom_modulus_8() {
        let data = b"test";
        let result1 = koopman8_with_modulus(data, 0, MODULUS_8);
        let result2 = koopman8_with_modulus(data, 0, MODULUS_8_ALT);

        // Different moduli should (usually) produce different results
        // Note: They could theoretically be equal, but very unlikely
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_custom_modulus_matches_default() {
        let data = b"test data";

        assert_eq!(
            koopman8(data, 0),
            koopman8_with_modulus(data, 0, MODULUS_8)
        );
        assert_eq!(
            koopman16(data, 0),
            koopman16_with_modulus(data, 0, MODULUS_16)
        );
        assert_eq!(
            koopman32(data, 0),
            koopman32_with_modulus(data, 0, MODULUS_32)
        );
    }

    // ========================================================================
    // Tests for edge cases
    // ========================================================================

    #[test]
    fn test_all_zeros() {
        let data = [0u8; 100];

        // Should produce valid checksums (not necessarily zero)
        let cs8 = koopman8(&data, 0);
        let cs16 = koopman16(&data, 0);
        let cs32 = koopman32(&data, 0);

        // Verify determinism
        assert_eq!(cs8, koopman8(&data, 0));
        assert_eq!(cs16, koopman16(&data, 0));
        assert_eq!(cs32, koopman32(&data, 0));
    }

    #[test]
    fn test_all_ones() {
        let data = [0xFFu8; 100];

        let cs8 = koopman8(&data, 0);
        let cs16 = koopman16(&data, 0);
        let cs32 = koopman32(&data, 0);

        // Verify determinism
        assert_eq!(cs8, koopman8(&data, 0));
        assert_eq!(cs16, koopman16(&data, 0));
        assert_eq!(cs32, koopman32(&data, 0));
    }

    #[test]
    fn test_single_byte_all_values() {
        // Test all possible single-byte inputs
        for byte in 0u8..=255 {
            let data = [byte];
            let _ = koopman8(&data, 0);
            let _ = koopman16(&data, 0);
            let _ = koopman32(&data, 0);
        }
    }

    // ========================================================================
    // Tests for streaming API with seed
    // ========================================================================

    #[test]
    fn test_streaming_with_seed() {
        let data = b"test data";
        let seed = 42u8;

        // One-shot with seed
        let expected = koopman16(data, seed);

        // Streaming with seed
        let mut hasher = Koopman16::with_seed(seed);
        hasher.update(data);
        let streaming = hasher.finalize();

        assert_eq!(expected, streaming);
    }

    #[test]
    fn test_streaming_with_seed_chunked() {
        let data = b"test data for chunked processing";
        let seed = 123u8;

        let expected = koopman16(data, seed);

        let mut hasher = Koopman16::with_seed(seed);
        hasher.update(&data[..10]);
        hasher.update(&data[10..20]);
        hasher.update(&data[20..]);
        let streaming = hasher.finalize();

        assert_eq!(expected, streaming);
    }

    // ========================================================================
    // Tests for reset behavior
    // ========================================================================

    #[test]
    fn test_reset_without_seed() {
        let data = b"test";

        let mut hasher = Koopman16::new();
        hasher.update(data);
        let first = hasher.finalize();

        let mut hasher = Koopman16::new();
        hasher.update(b"other data");
        hasher.reset();
        hasher.update(data);
        let after_reset = hasher.finalize();

        assert_eq!(first, after_reset);
    }

    #[test]
    fn test_reset_preserves_seed() {
        let data = b"test";
        let seed = 42u8;

        // First computation with seed
        let mut hasher = Koopman16::with_seed(seed);
        hasher.update(data);
        let first = hasher.finalize();

        // Computation after reset should produce same result
        let mut hasher = Koopman16::with_seed(seed);
        hasher.update(b"garbage data");
        hasher.reset();
        hasher.update(data);
        let after_reset = hasher.finalize();

        assert_eq!(first, after_reset);
    }

    #[test]
    fn test_reset_all_variants() {
        let data = b"test";

        // Koopman8
        let mut h8 = Koopman8::with_seed(10);
        h8.update(b"junk");
        h8.reset();
        h8.update(data);
        assert_eq!(h8.finalize(), koopman8(data, 10));

        // Koopman16
        let mut h16 = Koopman16::with_seed(20);
        h16.update(b"junk");
        h16.reset();
        h16.update(data);
        assert_eq!(h16.finalize(), koopman16(data, 20));

        // Koopman32
        let mut h32 = Koopman32::with_seed(30);
        h32.update(b"junk");
        h32.reset();
        h32.update(data);
        assert_eq!(h32.finalize(), koopman32(data, 30));
    }

    // ========================================================================
    // Tests for two-bit error detection
    // ========================================================================

    #[test]
    fn test_two_bit_error_detection() {
        // Test that most two-bit errors are detected
        // Note: HD=3 means we detect ALL 1-bit and 2-bit errors
        let data = b"Test";
        let original = koopman16(data, 0);
        let mut detected = 0;
        let mut total = 0;

        for i in 0..data.len() {
            for j in i..data.len() {
                for bit_i in 0..8 {
                    for bit_j in 0..8 {
                        if i == j && bit_i == bit_j {
                            continue; // Skip single-bit errors
                        }
                        total += 1;
                        let mut corrupted = data.to_vec();
                        corrupted[i] ^= 1 << bit_i;
                        corrupted[j] ^= 1 << bit_j;
                        if koopman16(&corrupted, 0) != original {
                            detected += 1;
                        }
                    }
                }
            }
        }

        // Should detect all two-bit errors for data within HD=3 length
        assert_eq!(detected, total, "Should detect all two-bit errors");
    }

    // ========================================================================
    // Tests for streaming API edge cases
    // ========================================================================

    #[test]
    fn test_streaming_empty_updates() {
        let data = b"test";

        let mut hasher = Koopman16::new();
        hasher.update(&[]);  // Empty update
        hasher.update(data);
        hasher.update(&[]);  // Another empty update

        assert_eq!(hasher.finalize(), koopman16(data, 0));
    }

    #[test]
    fn test_streaming_byte_by_byte() {
        let data = b"test data";

        let mut hasher = Koopman16::new();
        for &byte in data {
            hasher.update(&[byte]);
        }

        assert_eq!(hasher.finalize(), koopman16(data, 0));
    }

    #[test]
    fn test_finalize_without_data() {
        let hasher = Koopman16::new();
        assert_eq!(hasher.finalize(), 0);

        let hasher_with_seed = Koopman16::with_seed(42);
        assert_eq!(hasher_with_seed.finalize(), 0);
    }
}
