//! Example demonstrating Koopman checksum usage.
//!
//! Run with: `cargo run --example basic`

use koopman_checksum::*;

fn main() {
    println!("Koopman Checksum Examples");
    println!("=========================\n");

    // Basic usage
    let data = b"Hello, World!";
    
    println!("Data: {:?}", String::from_utf8_lossy(data));
    println!();

    // 8-bit checksum
    let checksum8 = koopman8(data, 0);
    println!("Koopman8 checksum:  0x{:02X} (HD=3 up to 13 bytes)", checksum8);

    // 16-bit checksum
    let checksum16 = koopman16(data, 0);
    println!("Koopman16 checksum: 0x{:04X} (HD=3 up to 4092 bytes)", checksum16);

    // 32-bit checksum
    let checksum32 = koopman32(data, 0);
    println!("Koopman32 checksum: 0x{:08X} (HD=3 up to 134M bytes)", checksum32);

    println!();

    // Parity variants (HD=4)
    println!("Parity Variants (HD=4):");
    let checksum8p = koopman8p(data, 0);
    println!("Koopman8P:  0x{:02X} (7-bit checksum + parity)", checksum8p);

    let checksum16p = koopman16p(data, 0);
    println!("Koopman16P: 0x{:04X} (15-bit checksum + parity)", checksum16p);

    let checksum32p = koopman32p(data, 0);
    println!("Koopman32P: 0x{:08X} (31-bit checksum + parity)", checksum32p);

    println!();

    // Verification
    println!("Verification:");
    let is_valid = verify16(data, checksum16, 0);
    println!("Data integrity check: {}", if is_valid { "PASS" } else { "FAIL" });

    // Corrupted data
    let mut corrupted = data.to_vec();
    corrupted[0] ^= 0x01; // Flip one bit
    let is_valid_corrupted = verify16(&corrupted, checksum16, 0);
    println!("Corrupted data check: {}", if is_valid_corrupted { "PASS (ERROR!)" } else { "FAIL (correct detection)" });

    println!();

    // Streaming API
    println!("Streaming API:");
    let mut hasher = Koopman16::new();
    hasher.update(b"Hello, ");
    hasher.update(b"World!");
    let streaming_checksum = hasher.finalize();
    println!("Streaming checksum: 0x{:04X}", streaming_checksum);
    println!("Matches one-shot:   {}", streaming_checksum == checksum16);

    println!();

    // Using seed
    println!("Using Initial Seed:");
    let checksum_seed0 = koopman16(data, 0);
    let checksum_seed1 = koopman16(data, 1);
    let checksum_seed42 = koopman16(data, 42);
    println!("Seed 0:  0x{:04X}", checksum_seed0);
    println!("Seed 1:  0x{:04X}", checksum_seed1);
    println!("Seed 42: 0x{:04X}", checksum_seed42);

    println!();

    // Error detection demonstration
    println!("Error Detection Demo:");
    let original = b"The quick brown fox";
    let checksum = koopman16(original, 0);
    println!("Original: {:?}", String::from_utf8_lossy(original));
    println!("Checksum: 0x{:04X}", checksum);

    // Single bit error
    let mut single_bit = original.to_vec();
    single_bit[5] ^= 0x04;
    let single_bit_cs = koopman16(&single_bit, 0);
    println!("\n1-bit error: {:?}", String::from_utf8_lossy(&single_bit));
    println!("New checksum: 0x{:04X} (detected: {})", single_bit_cs, single_bit_cs != checksum);

    // Two bit error
    let mut two_bit = original.to_vec();
    two_bit[3] ^= 0x10;
    two_bit[10] ^= 0x02;
    let two_bit_cs = koopman16(&two_bit, 0);
    println!("\n2-bit error: {:?}", String::from_utf8_lossy(&two_bit));
    println!("New checksum: 0x{:04X} (detected: {})", two_bit_cs, two_bit_cs != checksum);

    println!();
    println!("Done!");
}
