#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

// Copyright (c) 2025 the koopman-checksum authors, all rights reserved.
// See README.md for licensing information.

use core::num::{NonZeroU32, NonZeroU64};

// ============================================================================
// Constants
// ============================================================================

/// Recommended modulus for 8-bit Koopman checksum (HD=3 up to 13 bytes)
pub const MODULUS_8: u32 = 253;

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

const NONZERO_MODULUS_8: NonZeroU32 = NonZeroU32::new(MODULUS_8).unwrap();
const NONZERO_MODULUS_7P: NonZeroU32 = NonZeroU32::new(MODULUS_7P).unwrap();
const NONZERO_MODULUS_15P: NonZeroU32 = NonZeroU32::new(MODULUS_15P).unwrap();
const NONZERO_MODULUS_31P: NonZeroU64 = NonZeroU64::new(MODULUS_31P).unwrap();

// ============================================================================
// Fast Modular Reduction
//
// The moduli are of the form 2^k - c where c is small:
// - 65519 = 2^16 - 17
// - 4294967291 = 2^32 - 5
//
// This allows fast reduction: x % (2^k - c) ≡ (x >> k) * c + (x & (2^k - 1))
// ============================================================================

/// Fast reduction for modulus 65519 = 2^16 - 17
/// Input: x up to (MODULUS_16 - 1) << 16 + 0xFFFF ≈ 4_293_918_719 (remains < 2^32)
#[inline(always)]
fn fast_mod_65519(x: u32) -> u32 {
    // First reduction: x = hi * 2^16 + lo, result = hi * 17 + lo
    let hi: u32 = x >> 16;
    let lo: u32 = x & 0xFFFF;
    let r: u32 = hi * 17 + lo;
    // r < 17 * 256 + 65536 = 69888
    // Second reduction
    let hi2: u32 = r >> 16;
    let lo2: u32 = r & 0xFFFF;
    let r2: u32 = hi2 * 17 + lo2;
    // r2 < 17 * 2 + 65536 = 65570
    if r2 >= MODULUS_16 { r2 - MODULUS_16 } else { r2 }
}

/// Fast reduction for modulus 4294967291 = 2^32 - 5
/// Input: x < 2^40 (after shift+add)
#[inline(always)]
fn fast_mod_4294967291(x: u64) -> u64 {
    // x = hi * 2^32 + lo, result = hi * 5 + lo
    let hi: u64 = x >> 32;
    let lo: u64 = x & 0xFFFFFFFF;
    let r: u64 = hi * 5 + lo;
    // r < 5 * 2^8 + 2^32, need one check
    if r >= MODULUS_32 { r - MODULUS_32 } else { r }
}

/// Compute an 8-bit Koopman checksum.
///
/// Provides HD=3 fault detection for data words up to 13 bytes with modulus 253.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value, **IMPORTANT**: must be non-zero and odd
///
/// # Use an odd seed value
/// To ensure HD=3 fault detection, the initial seed must be odd and non-zero.
///
/// # Returns
/// 8-bit checksum value, or 0 if data is empty
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman8;
///
/// let checksum = koopman8(b"test data", 0xee);
/// assert_eq!(koopman8(&[], 0xee), 0); // Empty data returns 0
/// ```
#[inline]
#[must_use]
pub fn koopman8(data: &[u8], initial_seed: u8) -> u8 {
    koopman8_with_modulus(data, initial_seed, NONZERO_MODULUS_8)
}

/// Compute an 8-bit Koopman checksum with a custom modulus.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value, **IMPORTANT**: must be non-zero and odd
/// * `modulus` - The modulus to use (recommended: 253 or 239). Must be non-zero.
///
/// # Returns
/// 8-bit checksum value, or 0 if data is empty
///
/// # Use an odd seed value
/// To ensure HD=3 fault detection, the initial seed must be odd and non-zero.
///
/// # Example
/// ```rust
/// use std::num::NonZeroU32;
/// use koopman_checksum::koopman8_with_modulus;
///
/// let modulus = NonZeroU32::new(239).unwrap();
/// let checksum = koopman8_with_modulus(b"test", 0xee, modulus);
/// ```
#[inline]
#[must_use]
pub fn koopman8_with_modulus(data: &[u8], initial_seed: u8, modulus: NonZeroU32) -> u8 {
    let modulus = modulus.get();

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
/// 16-bit checksum value, or 0 if data is empty
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman16;
///
/// let checksum = koopman16(b"test data", 0xee);
/// assert_eq!(koopman16(&[], 0xee), 0); // Empty data returns 0
/// ```
#[inline]
#[must_use]
pub fn koopman16(data: &[u8], initial_seed: u8) -> u16 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    // Process bytes with delayed modulo reduction every 2 bytes
    // This reduces the number of modulo operations by half
    let mut count = 0;
    for &byte in &data[1..] {
        sum = (sum << 8) + byte as u64;
        count += 1;
        if count == 2 {
            sum = fast_mod_65519(sum as u32) as u64;
            count = 0;
        }
    }

    // Final reduction if needed
    if count > 0 {
        sum = fast_mod_65519(sum as u32) as u64;
    }

    // Append two implicit zero bytes
    sum = fast_mod_65519((sum << 8) as u32) as u64;
    sum = fast_mod_65519((sum << 8) as u32) as u64;

    sum as u16
}

/// Compute a 16-bit Koopman checksum with a custom modulus.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value
/// * `modulus` - The modulus to use. Must be non-zero.
///
/// # Returns
/// 16-bit checksum value, or 0 if data is empty
///
/// # Example
/// ```rust
/// use std::num::NonZeroU32;
/// use koopman_checksum::koopman16_with_modulus;
///
/// let modulus = NonZeroU32::new(65519).unwrap();
/// let checksum = koopman16_with_modulus(b"test", 0xee, modulus);
/// ```
#[inline]
#[must_use]
pub fn koopman16_with_modulus(data: &[u8], initial_seed: u8, modulus: NonZeroU32) -> u16 {
    let modulus = modulus.get();

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
/// 32-bit checksum value, or 0 if data is empty
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman32;
///
/// let checksum = koopman32(b"test data", 0xee);
/// assert_eq!(koopman32(&[], 0xee), 0); // Empty data returns 0
/// ```
#[inline]
#[must_use]
pub fn koopman32(data: &[u8], initial_seed: u8) -> u32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;

    // Use fast modular reduction for the default modulus
    for &byte in &data[1..] {
        sum = fast_mod_4294967291((sum << 8) + byte as u64);
    }

    // Append four implicit zero bytes
    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);
    sum = fast_mod_4294967291(sum << 8);

    sum as u32
}

/// Compute a 32-bit Koopman checksum with a custom modulus.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value
/// * `modulus` - The modulus to use. Must be non-zero.
///
/// # Returns
/// 32-bit checksum value, or 0 if data is empty
///
/// # Example
/// ```rust
/// use std::num::NonZeroU64;
/// use koopman_checksum::koopman32_with_modulus;
///
/// let modulus = NonZeroU64::new(4294967291).unwrap();
/// let checksum = koopman32_with_modulus(b"test", 0xee, modulus);
/// ```
#[inline]
#[must_use]
pub fn koopman32_with_modulus(data: &[u8], initial_seed: u8, modulus: NonZeroU64) -> u32 {
    let modulus = modulus.get();

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
/// Uses modulus 125 for the 7-bit checksum portion.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 8-bit value: 7-bit checksum in upper bits, parity in LSB, or 0 if data is empty
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman8p;
///
/// let checksum = koopman8p(b"test", 0xee);
/// let parity_bit = checksum & 1;
/// let checksum_bits = checksum >> 1;
/// ```
#[inline]
#[must_use]
pub fn koopman8p(data: &[u8], initial_seed: u8) -> u8 {
    koopman8p_with_modulus(data, initial_seed, NONZERO_MODULUS_7P)
}

/// Compute an 8-bit Koopman checksum with parity using a custom modulus.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value
/// * `modulus` - The modulus for the 7-bit checksum. Must be non-zero and ≤ 127.
///
/// # Returns
/// 8-bit value: 7-bit checksum in upper bits, parity in LSB, or 0 if data is empty
///
/// # Example
/// ```rust
/// use std::num::NonZeroU32;
/// use koopman_checksum::koopman8p_with_modulus;
///
/// let modulus = NonZeroU32::new(125).unwrap();
/// let checksum = koopman8p_with_modulus(b"test", 0xee, modulus);
/// ```
#[inline]
#[must_use]
pub fn koopman8p_with_modulus(data: &[u8], initial_seed: u8, modulus: NonZeroU32) -> u8 {
    let modulus = modulus.get();

    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;
    let mut psum: u8 = sum as u8;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % modulus;
        psum ^= byte;
    }

    // Append implicit zero byte
    sum = (sum << 8) % modulus;

    // Pack: checksum in upper 7 bits, parity in LSB
    // Parity covers the same byte stream as the checksum core, i.e. data[0] ^ seed
    ((sum as u8) << 1) | parity8(psum)
}

/// Compute a 16-bit Koopman checksum with parity (15-bit checksum + 1 parity bit).
///
/// Provides HD=4 fault detection for data words up to 2044 bytes.
/// Uses modulus 32749 for the 15-bit checksum portion.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 16-bit value: 15-bit checksum in upper bits, parity in LSB, or 0 if data is empty
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman16p;
///
/// let checksum = koopman16p(b"test data", 0xee);
/// let parity_bit = checksum & 1;
/// let checksum_bits = checksum >> 1;
/// ```
#[inline]
#[must_use]
pub fn koopman16p(data: &[u8], initial_seed: u8) -> u16 {
    koopman16p_with_modulus(data, initial_seed, NONZERO_MODULUS_15P)
}

/// Compute a 16-bit Koopman checksum with parity using a custom modulus.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value
/// * `modulus` - The modulus for the 15-bit checksum. Must be non-zero and ≤ 32767.
///
/// # Returns
/// 16-bit value: 15-bit checksum in upper bits, parity in LSB, or 0 if data is empty
///
/// # Example
/// ```rust
/// use std::num::NonZeroU32;
/// use koopman_checksum::koopman16p_with_modulus;
///
/// let modulus = NonZeroU32::new(32749).unwrap();
/// let checksum = koopman16p_with_modulus(b"test", 0xee, modulus);
/// ```
#[inline]
#[must_use]
pub fn koopman16p_with_modulus(data: &[u8], initial_seed: u8, modulus: NonZeroU32) -> u16 {
    let modulus = modulus.get();

    if data.is_empty() {
        return 0;
    }

    let mut sum: u32 = (data[0] ^ initial_seed) as u32;
    let mut psum: u8 = sum as u8;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u32) % modulus;
        psum ^= byte;
    }

    // Append two implicit zero bytes
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;

    // Pack: checksum in upper 15 bits, parity in LSB
    // Parity covers the same byte stream as the checksum core, i.e. data[0] ^ seed
    ((sum as u16) << 1) | (parity8(psum) as u16)
}

/// Compute a 32-bit Koopman checksum with parity (31-bit checksum + 1 parity bit).
///
/// Provides HD=4 fault detection for data words up to 134,217,720 bytes.
/// Uses modulus 2147483629 for the 31-bit checksum portion.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value (typically 0)
///
/// # Returns
/// 32-bit value: 31-bit checksum in upper bits, parity in LSB, or 0 if data is empty
///
/// # Example
/// ```rust
/// use koopman_checksum::koopman32p;
///
/// let checksum = koopman32p(b"test data", 0xee);
/// let parity_bit = checksum & 1;
/// let checksum_bits = checksum >> 1;
/// ```
#[inline]
#[must_use]
pub fn koopman32p(data: &[u8], initial_seed: u8) -> u32 {
    koopman32p_with_modulus(data, initial_seed, NONZERO_MODULUS_31P)
}

/// Compute a 32-bit Koopman checksum with parity using a custom modulus.
///
/// # Arguments
/// * `data` - The data bytes to checksum
/// * `initial_seed` - Initial seed value
/// * `modulus` - The modulus for the 31-bit checksum. Must be non-zero and ≤ 2^31-1.
///
/// # Returns
/// 32-bit value: 31-bit checksum in upper bits, parity in LSB, or 0 if data is empty
///
/// # Example
/// ```rust
/// use std::num::NonZeroU64;
/// use koopman_checksum::koopman32p_with_modulus;
///
/// let modulus = NonZeroU64::new(2147483629).unwrap();
/// let checksum = koopman32p_with_modulus(b"test", 0xee, modulus);
/// ```
#[inline]
#[must_use]
pub fn koopman32p_with_modulus(data: &[u8], initial_seed: u8, modulus: NonZeroU64) -> u32 {
    let modulus = modulus.get();

    if data.is_empty() {
        return 0;
    }

    let mut sum: u64 = (data[0] ^ initial_seed) as u64;
    let mut psum: u8 = sum as u8;

    for &byte in &data[1..] {
        sum = ((sum << 8) + byte as u64) % modulus;
        psum ^= byte;
    }

    // Append four implicit zero bytes
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;
    sum = (sum << 8) % modulus;

    // Pack: checksum in upper 31 bits, parity in LSB
    // Parity covers the same byte stream as the checksum core, i.e. data[0] ^ seed
    ((sum as u32) << 1) | (parity8(psum) as u32)
}

// ============================================================================
// Streaming/Incremental API
// ============================================================================

/// Macro to generate streaming checksum structs.
/// This reduces code duplication across Koopman8, Koopman16, Koopman32.
macro_rules! impl_streaming_hasher {
    (
        $name:ident,
        $sum_type:ty,
        $output_type:ty,
        $default_modulus_raw:expr,
        $nonzero_type:ty,
        $finalize_shifts:expr,
        $fast_mod:expr
    ) => {
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            /// Create a new hasher with the default modulus.
            #[inline]
            pub fn new() -> Self {
                Self {
                    sum: 0,
                    modulus: $default_modulus_raw,
                    seed: 0,
                    initialized: false,
                    use_fast_mod: true,
                }
            }

            /// Create a new hasher with a custom modulus.
            ///
            /// # Arguments
            /// * `modulus` - The modulus to use. Must be non-zero.
            ///
            /// # Example
            /// ```rust
            #[doc = concat!("use std::num::", stringify!($nonzero_type), ";")]
            #[doc = concat!("use koopman_checksum::{", stringify!($name), ", ", stringify!($default_modulus_raw), "};")]
            ///
            #[doc = concat!("let modulus = ", stringify!($nonzero_type), "::new(", stringify!($default_modulus_raw), ").unwrap();")]
            #[doc = concat!("let hasher = ", stringify!($name), "::with_modulus(modulus);")]
            /// ```
            #[inline]
            pub fn with_modulus(modulus: $nonzero_type) -> Self {
                let modulus_val = modulus.get();
                Self {
                    sum: 0,
                    modulus: modulus_val,
                    seed: 0,
                    initialized: false,
                    use_fast_mod: modulus_val == $default_modulus_raw,
                }
            }

            /// Create a new hasher with an initial seed.
            ///
            /// # Example
            /// ```rust
            #[doc = concat!("use koopman_checksum::", stringify!($name), ";")]
            ///
            #[doc = concat!("let hasher = ", stringify!($name), "::with_seed(0xee);")]
            /// ```
            #[inline]
            pub fn with_seed(seed: u8) -> Self {
                Self {
                    sum: seed as $sum_type,
                    modulus: $default_modulus_raw,
                    seed: seed as $sum_type,
                    initialized: false,
                    use_fast_mod: true,
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
                        self.sum ^= first as $sum_type;
                        self.initialized = true;
                    }
                }

                if self.use_fast_mod {
                    for &byte in iter {
                        self.sum = $fast_mod((self.sum << 8) + byte as $sum_type);
                    }
                } else {
                    for &byte in iter {
                        self.sum = ((self.sum << 8) + byte as $sum_type) % self.modulus;
                    }
                }
            }

            /// Finalize and return the checksum.
            ///
            /// Returns 0 if no data was provided.
            #[inline]
            #[must_use]
            pub fn finalize(self) -> $output_type {
                if !self.initialized {
                    return 0;
                }
                let mut sum = self.sum;
                if self.use_fast_mod {
                    for _ in 0..$finalize_shifts {
                        sum = $fast_mod(sum << 8);
                    }
                } else {
                    for _ in 0..$finalize_shifts {
                        sum = (sum << 8) % self.modulus;
                    }
                }
                sum as $output_type
            }

            /// Reset the hasher to initial state.
            #[inline]
            pub fn reset(&mut self) {
                self.sum = self.seed;
                self.initialized = false;
            }
        }
    };
}

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
    use_fast_mod: bool,
}

// Koopman8 doesn't have a fast_mod, so we use a passthrough
#[inline(always)]
fn identity_mod_8(x: u32) -> u32 { x % MODULUS_8 }

impl_streaming_hasher!(
    Koopman8, u32, u8,
    MODULUS_8, NonZeroU32,
    1, identity_mod_8
);

/// Incremental Koopman16 checksum calculator.
///
/// Allows computing checksums over data that arrives in chunks.
/// Uses fast modular reduction when using the default modulus.
///
/// # Example
/// ```rust
/// use koopman_checksum::Koopman16;
///
/// let mut hasher = Koopman16::new();
/// hasher.update(b"Hello, ");
/// hasher.update(b"World!");
/// let checksum = hasher.finalize();
/// ```
#[derive(Clone, Debug)]
pub struct Koopman16 {
    sum: u32,
    modulus: u32,
    seed: u32,
    initialized: bool,
    use_fast_mod: bool,
}

impl_streaming_hasher!(
    Koopman16, u32, u16,
    MODULUS_16, NonZeroU32,
    2, fast_mod_65519
);

/// Incremental Koopman32 checksum calculator.
///
/// Allows computing checksums over data that arrives in chunks.
/// Uses fast modular reduction when using the default modulus.
///
/// # Example
/// ```rust
/// use koopman_checksum::Koopman32;
///
/// let mut hasher = Koopman32::new();
/// hasher.update(b"Hello, ");
/// hasher.update(b"World!");
/// let checksum = hasher.finalize();
/// ```
#[derive(Clone, Debug)]
pub struct Koopman32 {
    sum: u64,
    modulus: u64,
    seed: u64,
    initialized: bool,
    use_fast_mod: bool,
}

impl_streaming_hasher!(
    Koopman32, u64, u32,
    MODULUS_32, NonZeroU64,
    4, fast_mod_4294967291
);

// ============================================================================
// Parity Streaming API
// ============================================================================

/// Macro to generate streaming parity checksum structs.
macro_rules! impl_streaming_parity_hasher {
    (
        $name:ident,
        $sum_type:ty,
        $output_type:ty,
        $default_modulus_raw:expr,
        $nonzero_type:ty,
        $finalize_shifts:expr
    ) => {
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            /// Create a new hasher with the default modulus.
            #[inline]
            pub fn new() -> Self {
                Self {
                    sum: 0,
                    psum: 0,
                    modulus: $default_modulus_raw,
                    seed: 0,
                    initialized: false,
                }
            }

            /// Create a new hasher with a custom modulus.
            ///
            /// # Arguments
            /// * `modulus` - The modulus to use. Must be non-zero.
            #[inline]
            pub fn with_modulus(modulus: $nonzero_type) -> Self {
                Self {
                    sum: 0,
                    psum: 0,
                    modulus: modulus.get(),
                    seed: 0,
                    initialized: false,
                }
            }

            /// Create a new hasher with an initial seed.
            #[inline]
            pub fn with_seed(seed: u8) -> Self {
                Self {
                    sum: seed as $sum_type,
                    psum: seed,
                    modulus: $default_modulus_raw,
                    seed: seed as $sum_type,
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
                        self.sum ^= first as $sum_type;
                        self.psum ^= first;
                        self.initialized = true;
                    }
                }

                for &byte in iter {
                    self.sum = ((self.sum << 8) + byte as $sum_type) % self.modulus;
                    self.psum ^= byte;
                }
            }

            /// Finalize and return the checksum with parity.
            ///
            /// Returns 0 if no data was provided.
            #[inline]
            #[must_use]
            pub fn finalize(self) -> $output_type {
                if !self.initialized {
                    return 0;
                }
                let mut sum = self.sum;
                for _ in 0..$finalize_shifts {
                    sum = (sum << 8) % self.modulus;
                }
                // Pack: checksum in upper bits, parity in LSB
                ((sum as $output_type) << 1) | (parity8(self.psum) as $output_type)
            }

            /// Reset the hasher to initial state.
            #[inline]
            pub fn reset(&mut self) {
                self.sum = self.seed;
                self.psum = self.seed as u8;
                self.initialized = false;
            }
        }
    };
}

/// Incremental Koopman8P checksum calculator (7-bit checksum + 1 parity bit).
///
/// Allows computing HD=4 checksums over data that arrives in chunks.
///
/// # Example
/// ```rust
/// use koopman_checksum::Koopman8P;
///
/// let mut hasher = Koopman8P::new();
/// hasher.update(b"Hello");
/// let checksum = hasher.finalize();
/// let parity_bit = checksum & 1;
/// ```
#[derive(Clone, Debug)]
pub struct Koopman8P {
    sum: u32,
    psum: u8,
    modulus: u32,
    seed: u32,
    initialized: bool,
}

impl_streaming_parity_hasher!(
    Koopman8P, u32, u8,
    MODULUS_7P, NonZeroU32,
    1
);

/// Incremental Koopman16P checksum calculator (15-bit checksum + 1 parity bit).
///
/// Allows computing HD=4 checksums over data that arrives in chunks.
///
/// # Example
/// ```rust
/// use koopman_checksum::Koopman16P;
///
/// let mut hasher = Koopman16P::new();
/// hasher.update(b"Hello, ");
/// hasher.update(b"World!");
/// let checksum = hasher.finalize();
/// let parity_bit = checksum & 1;
/// ```
#[derive(Clone, Debug)]
pub struct Koopman16P {
    sum: u32,
    psum: u8,
    modulus: u32,
    seed: u32,
    initialized: bool,
}

impl_streaming_parity_hasher!(
    Koopman16P, u32, u16,
    MODULUS_15P, NonZeroU32,
    2
);

/// Incremental Koopman32P checksum calculator (31-bit checksum + 1 parity bit).
///
/// Allows computing HD=4 checksums over data that arrives in chunks.
///
/// # Example
/// ```rust
/// use koopman_checksum::Koopman32P;
///
/// let mut hasher = Koopman32P::new();
/// hasher.update(b"Hello, ");
/// hasher.update(b"World!");
/// let checksum = hasher.finalize();
/// let parity_bit = checksum & 1;
/// ```
#[derive(Clone, Debug)]
pub struct Koopman32P {
    sum: u64,
    psum: u8,
    modulus: u64,
    seed: u64,
    initialized: bool,
}

impl_streaming_parity_hasher!(
    Koopman32P, u64, u32,
    MODULUS_31P, NonZeroU64,
    4
);

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
///
/// # Example
/// ```rust
/// use koopman_checksum::{koopman8, verify8};
///
/// let data = b"test data";
/// let checksum = koopman8(data, 0xee);
/// assert!(verify8(data, checksum, 0xee));
/// assert!(!verify8(data, checksum.wrapping_add(1), 0));
/// ```
#[inline]
#[must_use]
pub fn verify8(data: &[u8], expected: u8, initial_seed: u8) -> bool {
    koopman8(data, initial_seed) == expected
}

/// Verify data integrity using Koopman16 checksum.
///
/// # Arguments
/// * `data` - The data bytes (excluding checksum)
/// * `expected` - The expected checksum value
/// * `initial_seed` - Initial seed used when computing the checksum
///
/// # Returns
/// `true` if the checksum matches, `false` otherwise
///
/// # Example
/// ```rust
/// use koopman_checksum::{koopman16, verify16};
///
/// let data = b"test data";
/// let checksum = koopman16(data, 0xee);
/// assert!(verify16(data, checksum, 0xee));
/// ```
#[inline]
#[must_use]
pub fn verify16(data: &[u8], expected: u16, initial_seed: u8) -> bool {
    koopman16(data, initial_seed) == expected
}

/// Verify data integrity using Koopman32 checksum.
///
/// # Arguments
/// * `data` - The data bytes (excluding checksum)
/// * `expected` - The expected checksum value
/// * `initial_seed` - Initial seed used when computing the checksum
///
/// # Returns
/// `true` if the checksum matches, `false` otherwise
///
/// # Example
/// ```rust
/// use koopman_checksum::{koopman32, verify32};
///
/// let data = b"test data";
/// let checksum = koopman32(data, 0xee);
/// assert!(verify32(data, checksum, 0xee));
/// ```
#[inline]
#[must_use]
pub fn verify32(data: &[u8], expected: u32, initial_seed: u8) -> bool {
    koopman32(data, initial_seed) == expected
}

/// Verify data integrity using Koopman8P checksum (with parity).
///
/// # Arguments
/// * `data` - The data bytes (excluding checksum)
/// * `expected` - The expected checksum value (7-bit checksum + 1 parity bit)
/// * `initial_seed` - Initial seed used when computing the checksum
///
/// # Returns
/// `true` if the checksum matches, `false` otherwise
///
/// # Example
/// ```rust
/// use koopman_checksum::{koopman8p, verify8p};
///
/// let data = b"test";
/// let checksum = koopman8p(data, 0xee);
/// assert!(verify8p(data, checksum, 0xee));
/// ```
#[inline]
#[must_use]
pub fn verify8p(data: &[u8], expected: u8, initial_seed: u8) -> bool {
    koopman8p(data, initial_seed) == expected
}

/// Verify data integrity using Koopman16P checksum (with parity).
///
/// # Arguments
/// * `data` - The data bytes (excluding checksum)
/// * `expected` - The expected checksum value (15-bit checksum + 1 parity bit)
/// * `initial_seed` - Initial seed used when computing the checksum
///
/// # Returns
/// `true` if the checksum matches, `false` otherwise
///
/// # Example
/// ```rust
/// use koopman_checksum::{koopman16p, verify16p};
///
/// let data = b"test data";
/// let checksum = koopman16p(data, 0xee);
/// assert!(verify16p(data, checksum, 0xee));
/// ```
#[inline]
#[must_use]
pub fn verify16p(data: &[u8], expected: u16, initial_seed: u8) -> bool {
    koopman16p(data, initial_seed) == expected
}

/// Verify data integrity using Koopman32P checksum (with parity).
///
/// # Arguments
/// * `data` - The data bytes (excluding checksum)
/// * `expected` - The expected checksum value (31-bit checksum + 1 parity bit)
/// * `initial_seed` - Initial seed used when computing the checksum
///
/// # Returns
/// `true` if the checksum matches, `false` otherwise
///
/// # Example
/// ```rust
/// use koopman_checksum::{koopman32p, verify32p};
///
/// let data = b"test data";
/// let checksum = koopman32p(data, 0xee);
/// assert!(verify32p(data, checksum, 0xee));
/// ```
#[inline]
#[must_use]
pub fn verify32p(data: &[u8], expected: u32, initial_seed: u8) -> bool {
    koopman32p(data, initial_seed) == expected
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use core::num::NonZeroU32;
    use core::num::NonZeroU64;
    const NONZERO_MODULUS_16: NonZeroU32 = NonZeroU32::new(MODULUS_16).unwrap();
    const NONZERO_MODULUS_32: NonZeroU64 = NonZeroU64::new(MODULUS_32).unwrap();

    // Test vectors based on the C reference implementation
    const TEST_DATA: &[u8] = b"123456789";

    #[test]
    fn test_koopman8_empty() {
        assert_eq!(koopman8(&[], 0), 0);
        assert_eq!(koopman8(&[], 42), 0); // Empty data returns 0 regardless of initial seed
    }

    #[test]
    fn test_koopman8_single_byte() {
        // For single byte 0x12: sum = 0x12, then append zero: (0x12 << 8) % 253 = 4608 % 253 = 54
        assert_eq!(koopman8(&[0x12], 0), ((0x12u32 << 8) % MODULUS_8) as u8);
    }

    #[test]
    fn test_koopman16_empty() {
        assert_eq!(koopman16(&[], 0), 0);
        assert_eq!(koopman16(&[], 42), 0); // Empty data returns 0 regardless of initial seed
    }

    #[test]
    fn test_koopman32_empty() {
        assert_eq!(koopman32(&[], 0), 0);
        assert_eq!(koopman32(&[], 42), 0); // Empty data returns 0 regardless of initial seed
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
    fn test_seed_affects_result() {
        let result0 = koopman16(TEST_DATA, 0);
        let result1 = koopman16(TEST_DATA, 1);
        assert_ne!(result0, result1);
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
        // Input: [0x12, 0x34, 0x56] with initial seed 0, modulus 253
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
        // Verify that the parity bit correctly reflects the parity of data bytes only
        // (per the reference C implementation, checksum is NOT included in parity)
        let data = b"Test";
        let result = koopman8p(data, 0);

        // The checksum is in upper 7 bits
        let _checksum = result >> 1;
        let parity_bit = result & 1;

        // Compute expected parity: XOR all data bytes (NOT including checksum)
        let mut expected_parity: u8 = 0;
        for &byte in data {
            expected_parity ^= byte;
        }
        let expected_parity_bit = expected_parity.count_ones() & 1;

        assert_eq!(parity_bit as u32, expected_parity_bit);
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
        const MODULUS_8_ALT: u32 = 239;
        let data = b"test";
        let result1 = koopman8_with_modulus(data, 0, NONZERO_MODULUS_8);
        let modulus_alt = NonZeroU32::new(MODULUS_8_ALT).unwrap();
        let result2 = koopman8_with_modulus(data, 0, modulus_alt);

        // Different moduli should (usually) produce different results
        // Note: They could theoretically be equal, but very unlikely
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_custom_modulus_matches_default() {
        let data = b"test data";

        assert_eq!(
            koopman8(data, 0),
            koopman8_with_modulus(data, 0, NONZERO_MODULUS_8)
        );
        assert_eq!(
            koopman16(data, 0),
            koopman16_with_modulus(data, 0, NONZERO_MODULUS_16)
        );
        assert_eq!(
            koopman32(data, 0),
            koopman32_with_modulus(data, 0, NONZERO_MODULUS_32)
        );
    }

    #[test]
    fn test_parity_custom_modulus_matches_default() {
        let data = b"test data";

        assert_eq!(
            koopman8p(data, 0),
            koopman8p_with_modulus(data, 0, NONZERO_MODULUS_7P)
        );
        assert_eq!(
            koopman16p(data, 0),
            koopman16p_with_modulus(data, 0, NONZERO_MODULUS_15P)
        );
        assert_eq!(
            koopman32p(data, 0),
            koopman32p_with_modulus(data, 0, NONZERO_MODULUS_31P)
        );
    }

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

    #[test]
    fn test_streaming_parity_koopman8p() {
        let data = b"test";
        let expected = koopman8p(data, 0);

        let mut hasher = Koopman8P::new();
        hasher.update(&data[..2]);
        hasher.update(&data[2..]);
        let streaming = hasher.finalize();

        assert_eq!(expected, streaming);
    }

    #[test]
    fn test_streaming_parity_koopman16p() {
        let data = b"test data";
        let expected = koopman16p(data, 0);

        let mut hasher = Koopman16P::new();
        hasher.update(&data[..4]);
        hasher.update(&data[4..]);
        let streaming = hasher.finalize();

        assert_eq!(expected, streaming);
    }

    #[test]
    fn test_streaming_parity_koopman32p() {
        let data = b"test data for streaming";
        let expected = koopman32p(data, 0);

        let mut hasher = Koopman32P::new();
        hasher.update(&data[..10]);
        hasher.update(&data[10..]);
        let streaming = hasher.finalize();

        assert_eq!(expected, streaming);
    }

    #[test]
    fn test_streaming_parity_with_seed() {
        let data = b"test";
        let seed = 42u8;

        let expected = koopman16p(data, seed);

        let mut hasher = Koopman16P::with_seed(seed);
        hasher.update(data);
        let streaming = hasher.finalize();

        assert_eq!(expected, streaming);
    }

    // ========================================================================
    // Tests for parity verification
    // ========================================================================

    #[test]
    fn test_verify_parity() {
        let data = b"test data";

        let cs8p = koopman8p(data, 0);
        assert!(verify8p(data, cs8p, 0));
        assert!(!verify8p(data, cs8p.wrapping_add(1), 0));

        let cs16p = koopman16p(data, 0);
        assert!(verify16p(data, cs16p, 0));
        assert!(!verify16p(data, cs16p.wrapping_add(1), 0));

        let cs32p = koopman32p(data, 0);
        assert!(verify32p(data, cs32p, 0));
        assert!(!verify32p(data, cs32p.wrapping_add(1), 0));
    }

    // ========================================================================
    // Tests for streaming with custom modulus
    // ========================================================================

    #[test]
    fn test_streaming_with_custom_modulus() {
        let data = b"test data";

        // Test that streaming with default modulus matches one-shot
        let mut hasher = Koopman16::with_modulus(NONZERO_MODULUS_16);
        hasher.update(data);
        assert_eq!(hasher.finalize(), koopman16(data, 0));

        // Test with a different modulus
        let alt_modulus = NonZeroU32::new(32749).unwrap();
        let mut hasher = Koopman16::with_modulus(alt_modulus);
        hasher.update(data);
        let streaming = hasher.finalize();

        // Should produce a valid result (just verify it's deterministic)
        let mut hasher2 = Koopman16::with_modulus(alt_modulus);
        hasher2.update(data);
        assert_eq!(streaming, hasher2.finalize());
    }
}
