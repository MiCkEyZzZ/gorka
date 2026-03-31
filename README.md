# Gorka — GNSS Time-Series Compression

**Gorka** is a high-performance Rust library for compressing and
decompressing GNSS (Global Navigation Satellite System) time-series data.

A production-oriented time-series compression codec tailored for GNSS workloads.

It is designed for efficient storage and transmission of satellite
measurements such as pseudorange, Doppler, carrier phase, and signal quality.

The current implementation focuses on GLONASS, with planned support for
other constellations including GPS, Galileo, and BeiDou.

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

## Compression Model

- Delta encoding
- Delta-of-delta encoding
- Bit-packing
- Variable-length control bits

The approach is inspired by the Gorilla time-series compression algorithm,
but adapted and extended for GNSS-specific data patterns (multi-signal,
high-precision measurements, and slot-based state).

## Supported Data (GLONASS)

- Timestamp (ms)
- Frequency slot
- C/N0 (signal strength)
- Pseudorange (mm precision)
- Doppler (mHz)
- Carrier phase (optional)

## Roadmap

Planned improvements:

- GPS / Galileo / BeiDou support
- Entropy coding (Huffman / ANS)
- Streaming encoder/decoder API
- SIMD / branchless optimizations
- Real-world GNSS dataset benchmarks
- Cross-constellation compression model
- Advanced predictive models (beyond delta)

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
