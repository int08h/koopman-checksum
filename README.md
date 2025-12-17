# Koopman Checksum

A no-std implementation of the Koopman checksum algorithm as described in:

> Philip Koopman, "An Improved Modular Addition Checksum Algorithm"
> [arXiv:2304.13496](https://arxiv.org/abs/2304.13496) (2023)

## Overview

The Koopman checksum provides fault detection for significantly longer data values than dual-sum checksums like Adler, while using a single running sum.

### Advantages of Koopman Checksum

- Better fault detection than Fletcher/Adler dual-sum checksums for the same output check value size
- Simpler computation than CRC (uses integer division, not polynomial arithmetic)
- Detects all 1-2 bit errors for data up to 13 bytes (8-bit), 4,092 bytes (16-bit), or 134MiB (32-bit)
- Detects all 1-3 bit errors with '*p' parity variants for data up to 5 bytes (8-bit), 2,044 bytes (16-bit), or 134MB (32-bit)

### Understanding Hamming Distance (HD)

HD refers to the minimum Hamming distance of the _code_, which determines error detection capability:

- _HD=3_: Detects all 1-bit and 2-bit errors (but NOT all 3-bit errors)
- _HD=4_: Detects all 1-bit, 2-bit, and 3-bit errors (but NOT all 4-bit errors)

The number of detectable bit errors is always HD minus 1.

### Algorithm

The computational kernel is elegantly simple:

```text
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

// 8-bit checksum (detects all 1-2 bit errors up to 13 bytes)
let cs8 = koopman8(data, 0x01);

// 16-bit checksum (detects all 1-2 bit errors up to 4,092 bytes)
let cs16 = koopman16(data, 0x01);

// 32-bit checksum (detects all 1-2 bit errors up to 134,217,720 bytes)
let cs32 = koopman32(data, 0x01);
```

The second argument is the initial seed value. **A non-zero initial value (e.g., 1) is recommended** as this detects leading zeros in the data. 

**Warning: An initial seed of 0 means leading zero bytes in the data don't affect the checksum value.** Use an initial seed of 0 only if you want this behavior.

### Verification

```rust
use koopman_checksum::{koopman16, verify16};

let data = b"Important data";
let checksum = koopman16(data, 0x01);

// Later, verify integrity
if verify16(data, checksum, 0x01) {
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

### Parity Variants (Detects all 1-3 bit errors)

For applications requiring detection of all 1, 2, AND 3-bit errors, use the parity variants:

```rust
use koopman_checksum::{koopman8p, koopman16p, koopman32p};

let data = b"Critical data";

// 8-bit with parity (detects all 1-3 bit errors up to 5 bytes)
let cs8p = koopman8p(data, 0x01);

// 16-bit with parity (detects all 1-3 bit errors up to 2,044 bytes)
let cs16p = koopman16p(data, 0x01);

// 32-bit with parity (detects all 1-3 bit errors up to 134,217,720 bytes)
let cs32p = koopman32p(data, 0x01);
```

## Error Detection Capabilities

| Variant    | Modulus      | Detects all 1-2 bit errors | Detects all 1-3 bit errors |
|------------|--------------|---------------------------|---------------------------|
| Koopman8   | 253          | up to 13 bytes            | -                         |
| Koopman8P  | 125          | -                         | up to 5 bytes             |
| Koopman16  | 65519        | up to 4092 bytes          | -                         |
| Koopman16P | 32749        | -                         | up to 2044 bytes          |
| Koopman32  | 4294967291   | up to 134M bytes          | -                         |
| Koopman32P | 2147483629   | -                         | up to 134M bytes          |

**Note:** Beyond these lengths, the checksums still provide error detection, but some multi-bit errors may go undetected.

## Comparison with Other Checksums

| Algorithm    | Detects all 1-2 bit errors (16-bit) | Computation          |
|--------------|-------------------------------------|----------------------|
| Fletcher-16  | up to ~21 bytes                     | Dual modular sums    |
| Adler-16     | up to ~253 bytes                    | Dual modular sums    |
| Koopman16    | **up to 4092 bytes**                | Single modular sum   |
| CRC-16       | Varies by polynomial                | Polynomial division  |

## Use Cases

- Embedded systems: Simpler than CRC, better than Adler/Fletcher
- Network protocols: Fast integrity checking
- File storage: Corruption detection for small-to-medium files
- Memory integrity: Detect bit flips in RAM

## No-Std Support

This crate is `no_std` compatible by default. The `std` feature is enabled by default but can be disabled:

```toml
[dependencies]
koopman-checksum = { version = "0.1", default-features = false }
```

## Performance

Run benchmarks with:
```bash
cargo bench
```

### Why SIMD Doesn't Help

You might wonder why this library doesn't include SIMD optimizations. The Koopman checksum algorithm has a fundamental property that prevents parallelization: sequential data dependency.

The core computation is:
```text
for byte in data {
    sum = ((sum << k) + byte) % modulus;
}
```

Each iteration's result (`sum[n]`) depends on the previous iteration's result (`sum[n-1]`). This creates a loop-carried dependency that cannot be parallelized. We cannot compute `sum[n+1]` until `sum[n]` is known

The pure Rust implementation was consistently 20-40% faster than the simd implementations I could come up with. But
I am not experienced with simd techniques. If you know how to speed things up, please submit a PR!


## References

- [arXiv:2304.13496](https://arxiv.org/abs/2304.13496) - Original paper
- [Koopman CRC Resource Page](https://users.ece.cmu.edu/~koopman/crc/) - Checksum and CRC resources
- [Understanding Checksums and CRCs](https://users.ece.cmu.edu/~koopman/crc/book/index.html) - Book by Philip Koopman

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Copyright

Copyright (c) 2025 the koopman-checksum contributors, all rights reserved.

## Credits

Algorithm designed by Prof. Philip Koopman, Carnegie Mellon University.
Implementation by Stuart Stock (stuart@int08h.com)


