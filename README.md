# Gorka — GNSS Time-Series Compression

[![crates.io](https://img.shields.io/crates/v/gorka.svg)](https://crates.io/crates/gorka)
[![docs.rs](https://docs.rs/gorka/badge.svg)](https://docs.rs/gorka)

**Gorka** is a high-performance Rust library for compressing and
decompressing GNSS (Global Navigation Satellite System) time-series data.

A production-oriented time-series compression codec tailored for GNSS workloads,
focusing on performance, compactness, and correctness.

It is designed for efficient storage and transmission of satellite
measurements such as pseudorange, Doppler, carrier phase, and signal quality.

The current implementation focuses on GLONASS, with planned support for
other constellations including GPS, Galileo, and BeiDou.

## Why Gorka?

GNSS time-series data has unique properties:

- High temporal correlation
- High-precision measurements (mm / mHz)
- Multi-signal interleaving (multiple satellites)
- Frequent small deltas with occasional large jumps

General-purpose compressors (e.g. gzip, zstd) fail to exploit these patterns efficiently.

Gorka is designed specifically for GNSS workloads, achieving significantly better
compression ratios by combining:

- Domain-specific delta modeling
- Slot-aware state tracking (GLONASS FDMA)
- Bit-level encoding strategies

The result is a compact, fast, and predictable binary format.

## Status

⚠️ This project is under active development. The format and APIs may change
before the first stable release.

## Features

- Bit-level encoding/decoding (`bits` module)
- Delta and delta-of-delta encoding (inspired by Gorilla 🦍)
- Efficient integer encoding (zigzag)
- GLONASS sample compression (`gnss` module)
- Chunk-based binary format
- Forward-compatible versioning
- `no_std` support (core functionality)
- Zero-copy friendly design

## Example

```rust
use gorka::{
    codec::{GlonassEncoder, GlonassDecoder},
    GlonassSample, MilliHz, Millimeter,
};

let samples = vec![ /* ... */ ];

let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();
let decoded = GlonassDecoder::decode_chunk(&encoded).unwrap();

assert_eq!(samples, decoded);
```

## Installation

```toml
[dependencies]
gorka = "0.1" # or latest
```

## Examples

Gorka ships with working examples in the `examples/` folder. You can run them
with Cargo:

```zsh
cargo run --example encode_decode
# → Basic encode/decode test

cargo run --example edge_cases
# → Edge case test (missing carrier phase, multi-slot gaps, etc.)
```

## Compression Model

Gorka combines several techniques:

- Delta and delta-of-delta encoding
- Variable-length bit packing
- Zigzag encoding for signed values
- Field-specific encoding strategies
- Stateful prediction per signal (e.g. per-slot Doppler)

The design is inspired by the Gorilla time-series compression algorithm,
but extends it for GNSS-specific challenges such as:

- Multi-satellite interleaving
- High-precision numeric fields
- Optional signals (e.g. carrier phase)

## Compression Performance

Typical compression ratios (current implementation):

| Signal type | Ratio |
| ----------- | ----- |
| Constant    | ~20×  |
| Smooth      | ~7×   |
| Noisy       | ~2×   |

Results depend on signal characteristics.

## Supported Data (GLONASS)

- Timestamp (ms)
- Frequency slot
- C/N0 (signal strength)
- Pseudorange (mm precision)
- Doppler (mHz)
- Carrier phase (optional)

## Roadmap

### Core features

- GPS / Galileo / BeiDou support
- Streaming encoder/decoder API
- Cross-constellation compression model

### Compression improvements

- Entropy coding (Huffman / ANS)
- Advanced predictive models (beyond delta)

### Performance

- SIMD / branchless optimizations
- Real-world GNSS dataset benchmarks

## Testing

- Roundtrip correctness (encode -> decode)
- Compression ratio benchmarks:
  - constant signals
  - smooth signals
  - noisy signals
- Edge cases:
  - missing carrier phase
  - multi-slot interleaving
  - large timestamp gaps

All tests are automated and act as regression guards for both correctness
and compression efficiency.

## Documentation

- Format specification: `docs/FORMAT.md`
- Project structure: `docs/PROJECT_STRUCTURE.md`

## Platform support

- `std` (default)
- `no_std` (core encoding/decoding without the standard library)

## Development

```zsh
just dev       # run formatting, lint and all tests
just fmt-all   # format Rust and TOML
just test-next # run tests via nextest
```

## License

MIT OR Apache-2.0
