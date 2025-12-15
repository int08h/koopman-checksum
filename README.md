# Koopman Checksum

A Rust implementation of the Koopman checksum algorithm as described in:

> Philip Koopman, "An Improved Modular Addition Checksum Algorithm"
> [arXiv:2304.13496](https://arxiv.org/abs/2304.13496) (2023)

## Overview

The Koopman checksum provides **Hamming Distance 3 (HD=3)** fault detection for significantly longer data words than traditional dual-sum checksums like Fletcher, while using only a single running sum that is twice the size of the final check value.

### Key Features

- **Better fault detection** than Fletcher/Adler checksums for the same check value size
- **Simpler computation** than CRC (uses integer division, not polynomial arithmetic)
- **HD=3** detection for data up to 4KB (16-bit) or 134MB (32-bit)
- **HD=4** variants available with parity bit
- **No-std compatible** for embedded systems

### Algorithm

The computational kernel is elegantly simple:

```
sum = ((sum << k) + block) % modulus
```

Where `k` is the check value size in bits (8, 16, or 32).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
koopman-checksum = "0.1"
```

## Usage

### Basic Usage

```rust
use koopman_checksum::{koopman8, koopman16, koopman32};

let data = b"Hello, World!";

// 8-bit checksum (HD=3 up to 13 bytes)
let cs8 = koopman8(data, 0);

// 16-bit checksum (HD=3 up to 4092 bytes)
let cs16 = koopman16(data, 0);

// 32-bit checksum (HD=3 up to 134,217,720 bytes)
let cs32 = koopman32(data, 0);
```

The second argument is the seed. A seed of 0 is simple but means leading zero bytes don't affect the checksum. Use a non-zero seed (e.g., 1) if you need to detect leading zeros.

### Verification

```rust
use koopman_checksum::{koopman16, verify16};

let data = b"Important data";
let checksum = koopman16(data, 0);

// Later, verify integrity
if verify16(data, checksum, 0) {
    println!("Data is intact");
} else {
    println!("Data corruption detected!");
}
```

### Streaming API

For processing data in chunks:

```rust
use koopman_checksum::Koopman16;

let mut hasher = Koopman16::new();
hasher.update(b"First chunk");
hasher.update(b"Second chunk");
hasher.update(b"Third chunk");
let checksum = hasher.finalize();
```

### HD=4 Variants (with Parity)

For applications requiring detection of all 3-bit errors:

```rust
use koopman_checksum::{koopman8p, koopman16p, koopman32p};

let data = b"Critical data";

// 8-bit with parity (HD=4 up to 5 bytes)
let cs8p = koopman8p(data, 0);

// 16-bit with parity (HD=4 up to 2044 bytes)
let cs16p = koopman16p(data, 0);

// 32-bit with parity (HD=4 up to 134,217,720 bytes)
let cs32p = koopman32p(data, 0);
```

## Performance

Run benchmarks with:
```bash
cargo bench
```

### Why SIMD Doesn't Help

You might wonder why this library doesn't include SIMD optimizations. The Koopman checksum algorithm has a fundamental property that prevents parallelization: **sequential data dependency**.

The core computation is:
```rust
for byte in data {
    sum = ((sum << k) + byte) % modulus;
}
```

Each iteration's result (`sum[n]`) depends on the previous iteration's result (`sum[n-1]`). This creates a loop-carried dependency that cannot be parallelized. We cannot compute `sum[n+1]` until `sum[n]` is known

The pure Rust implementation was consistently 20-40% faster than the simd implementations I could come up with. But
I am not experienced with simd techniques. If you know how to speed things up, please submit a PR!

## Hamming Distance Capabilities

| Variant    | Modulus      | HD=3 Length  | HD=4 Length |
|------------|--------------|--------------|-------------|
| Koopman8   | 253          | 13 bytes     | -           |
| Koopman8P  | 125          | -            | 5 bytes     |
| Koopman16  | 65519        | 4092 bytes   | -           |
| Koopman16P | 32749        | -            | 2044 bytes  |
| Koopman32  | 4294967291   | 134M bytes   | -           |
| Koopman32P | 2147483629   | -            | 134M bytes  |

## Comparison with Other Checksums

| Algorithm    | HD=3 Length (16-bit) | Computation          |
|--------------|----------------------|----------------------|
| Fletcher-16  | ~21 bytes            | Dual modular sums    |
| Adler-16     | ~253 bytes           | Dual modular sums    |
| Koopman16    | **4092 bytes**       | Single modular sum   |
| CRC-16       | Varies by polynomial | Polynomial division  |

## Use Cases

- **Embedded systems**: Simpler than CRC, better than Fletcher
- **Network protocols**: Fast integrity checking
- **File storage**: Corruption detection for small-to-medium files
- **Memory integrity**: Detect bit flips in RAM

## No-Std Support

This crate is `no_std` compatible by default. The `std` feature is enabled by default but can be disabled:

```toml
[dependencies]
koopman-checksum = { version = "0.1", default-features = false }
```

## References

- [arXiv:2304.13496](https://arxiv.org/abs/2304.13496) - Original paper
- [Koopman CRC Resource Page](https://users.ece.cmu.edu/~koopman/crc/) - Additional resources
- [Understanding Checksums and CRCs](https://users.ece.cmu.edu/~koopman/crc/book/index.html) - Book by Philip Koopman

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Credits

Algorithm designed by Prof. Philip Koopman, Carnegie Mellon University.
Rust implementation by the community.
