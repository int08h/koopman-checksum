# Koopman Checksum

Rust implementation of Philip Koopman's modular addition checksum (arXiv:2304.13496).

## Build & Test

```bash
cargo build
cargo test
cargo bench
```

## Project Structure

- `src/lib.rs` - Core implementation with all checksum functions and streaming API
- `src/basic.rs` - Example usage (run with `cargo run --example basic`)
- `benches/benchmarks.rs` - Criterion benchmarks

## Key Design Decisions

- **No SIMD**: Sequential data dependency (`sum = ((sum << k) + byte) % modulus`) prevents parallelization. Pure Rust is faster.
- **Sequential finalization**: Appending implicit zero bytes must use multiple 8-bit shifts with intermediate modulo operations, not a single large shift.
- **Seed of 0**: Default seed is 0 for simplicity. Use non-zero seed if leading zeros must affect checksum.

## Variants

- **HD=3**: Detects all 1-bit and 2-bit errors (but NOT all 3-bit errors)
- **HD=4**: Detects all 1-bit, 2-bit, and 3-bit errors (but NOT all 4-bit errors)

| Function | Bits | Detects | Max Length |
|----------|------|---------|------------|
| koopman8 | 8 | all 1-2 bit errors | 13 bytes |
| koopman16 | 16 | all 1-2 bit errors | 4092 bytes |
| koopman32 | 32 | all 1-2 bit errors | 134M bytes |
| koopman8p | 8 | all 1-3 bit errors | 5 bytes |
| koopman16p | 16 | all 1-3 bit errors | 2044 bytes |
| koopman32p | 32 | all 1-3 bit errors | 134M bytes |

## Notes
- Do not add "crated by claude" to commit messages
- You MUST use benchmarks to validate that changes actually do improve performance. Make no assumptions, you must always test before and after and confirm quantitatively the results.
