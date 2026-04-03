# Gorka — GNSS Time-Series Compression

[![crates.io](https://img.shields.io/crates/v/gorka.svg)](https://crates.io/crates/gorka)
[![docs.rs](https://docs.rs/gorka/badge.svg)](https://docs.rs/gorka)

**Gorka** is a high-performance Rust library for compressing and
decompressing GNSS (Global Navigation Satellite System) time-series data.

A production-oriented time-series compression codec tailored for GNSS workloads,
focusing on performance, compactness, correctness, and `no_std` compatibility.

It is designed for efficient storage and transmission of satellite
measurements such as pseudorange, Doppler, carrier phase, and signal quality.

The current implementation focuses on GLONASS, with planned support for
other constellations including GPS, Galileo, and BeiDou.

## Table of Contents

- [Why Gorka?](#why-gorka)
- [Quick Start](#quick-start)
- [Examples](#examples)
- [Compression Performance](#compression-performance)
- [Supported Data](#supported-data)
- [no_std Mode](#no_std-mode)
- [Testing](#testing)
- [Documentation](#documentation)
- [Project Status](#project-status)
- [License](#license)

## Why Gorka?

GNSS time-series data has unique properties:

- High temporal correlation (slow field drift)
- High-precision measurements (pseudorange in mm, Doppler in mHz)
- Multiple satellites interleaved in one stream
- Frequent small deltas with occasional large jumps

General-purpose compressors (gzip, zstd) fail to exploit these patterns efficiently.

Gorka is designed specifically for GNSS workloads, achieving significantly
better compression ratios by combining:

- Domain-specific delta-of-delta encoding
- Per-slot state tracking (GLONASS FDMA)
- Bit-level packing with adaptive bucket schemes
- Integer fixed-point arithmetic — no floats, fully deterministic

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
gorka = "0.1"
```

Minimal example:

````rust
use gorka::{
    codec::{GlonassEncoder, GlonassDecoder},
    GlonassSample, MilliHz, Millimeter,
};

let samples = vec![GlonassSample {
    timestamp_ms:         1_700_000_000_000,
    slot:                 1,          // FDMA slot k ∈ [-7, +6]
    cn0_dbhz:             42,         // signal-to-noise, dBHz
    pseudorange_mm:       Millimeter::new(21_500_000_000),
    doppler_millihz:      MilliHz::new(1_200_500),
    carrier_phase_cycles: Some(123_456_789),
}];

// Compress
let compressed = GlonassEncoder::encode_chunk(&samples).unwrap();

// Decompress
let decoded = GlonassDecoder::decode_chunk(&compressed).unwrap();

assert_eq!(samples, decoded); // bitwise identical
```

## Examples

### Basic Encode/Decode

```rust
use gorka::{codec::{GlonassEncoder, GlonassDecoder}, GlonassSample, MilliHz, Millimeter};

let base_ts: u64 = 1_700_000_000_000;

// Create 10 observation epochs (1 Hz, 10 seconds)
let samples: Vec<GlonassSample> = (0..10).map(|i| GlonassSample {
    timestamp_ms:         base_ts + i * 1000,
    slot:                 1,
    cn0_dbhz:             42,
    pseudorange_mm:       Millimeter::new(21_500_000_000 + i as i64 * 100),
    doppler_millihz:      MilliHz::new(1_200_000 + i as i32 * 5),
    carrier_phase_cycles: Some(i as i64 * 65_536),
}).collect();

let compressed = GlonassEncoder::encode_chunk(&samples).unwrap();
let decoded    = GlonassDecoder::decode_chunk(&compressed).unwrap();

assert_eq!(samples, decoded);

let ratio = (samples.len() * 31) as f64 / compressed.len() as f64;
println!("Compression ratio: {ratio:.1}×"); // typically 4–7×
```

### Streaming to File

```rust
use gorka::{
    codec::GlonassEncoder,
    io::ChunkWriter,
    GlonassSample, MilliHz, Millimeter,
};
use std::{fs, io::BufWriter};

let file   = fs::File::create("gnss_log.bin").unwrap();
let mut w  = ChunkWriter::new(BufWriter::new(file));

for slot in [-7i8, -3, 0, 3, 6] {
    let samples: Vec<GlonassSample> = (0..60).map(|i| GlonassSample {
        timestamp_ms:         1_700_000_000_000 + i * 1000,
        slot,
        cn0_dbhz:             38,
        pseudorange_mm:       Millimeter::new(21_500_000_000),
        doppler_millihz:      MilliHz::new(1_000_000),
        carrier_phase_cycles: None,
    }).collect();

    let chunk = GlonassEncoder::encode_chunk(&samples).unwrap();
    w.write_chunk(&chunk).unwrap();
}

w.flush().unwrap();
println!("Written {} chunks", w.chunks_written());
```

### Streaming from File

```rust
use gorka::{codec::GlonassDecoder, io::ChunkReader};
use std::{fs, io::Read};

let mut data = Vec::new();
fs::File::open("gnss_log.bin").unwrap().read_to_end(&mut data).unwrap();

for (i, frame) in ChunkReader::new(&data).enumerate() {
    let samples = GlonassDecoder::decode_chunk(frame.unwrap()).unwrap();
    println!("chunk[{i}]: {} samples, slot k={:+}", samples.len(), samples[0].slot);
}
```

### GnssFrame — Single Epoch Buffer

```rust
use gorka::{GnssFrame, GlonassSample, MilliHz, Millimeter};

let ts = 1_700_000_000_000u64;
let mut frame = GnssFrame::new(ts);

for slot in [-7i8, -3, 0, 3, 6] {
    frame.push(GlonassSample {
        timestamp_ms: ts, slot,
        cn0_dbhz: 40,
        pseudorange_mm:  Millimeter::new(21_500_000_000),
        doppler_millihz: MilliHz::new(1_000_000),
        carrier_phase_cycles: None,
    }).unwrap();
}

assert_eq!(frame.len(), 5);
let sv = frame.get_by_slot(-7).unwrap();
println!("Slot -7 cn0: {} dBHz", sv.cn0_dbhz);

frame.validate_all().unwrap();
```

### GlonassSample Helper Methods

```rust
use gorka::{GlonassSample, MilliHz, Millimeter};

let s = GlonassSample {
    timestamp_ms: 1_700_000_000_000,
    slot: 1,
    cn0_dbhz: 42,
    pseudorange_mm:  Millimeter::new(21_500_000_000),
    doppler_millihz: MilliHz::new(1_200_500),
    carrier_phase_cycles: None,
};

let freq_mhz = s.carrier_freq_millihz().unwrap() as f64 / 1_000_000.0;
println!("Carrier: {freq_mhz} MHz");

println!("Pseudorange: {:.3} m", s.pseudorange_m_approx());

println!("Tracked: {}", s.is_tracked()); // cn0 ≥ 20 dBHz
```

## Compression Performance

Typical compression ratios (release build):

| Signal Type | Raw Size | Compressed | Ratio  | Bits/Sample |
| ----------- | -------- | ---------- | ------ | ----------- |
| Constant    | 11 776 B | 544 B      | 21.65× | 8.5         |
| Smooth      | 15 872 B | 2 155 B    | 7.37×  | 33.7        |
| Noisy       | 15 048 B | 6 490 B    | 2.32×  | 101.4       |

Results depend on signal characteristics. See [`docs/FORMAT.md`](docs/FORMAT.md).

## Supported Data (GLONASS)

| Field                  | Type   | Precision   | Range                       |
| ---------------------- | ------ | ----------- | --------------------------- |
| `timestamp_ms`         | `u64`  | 1 ms        | Unix timestamp              |
| `slot`                 | `i8`   | —           | k ∈ [-7, +6] (FDMA)         |
| `cn0_dbhz`             | `u8`   | 1 dBHz      | 0..255 (typical 20..55)     |
| `pseudorange_mm`       | `i64`  | 1 mm        | 19 100 000..25 600 000 km   |
| `doppler_millihz`      | `i32`  | 0.001 Hz    | ±5000 Hz                    |
| `carrier_phase_cycles` | `i64?` | 2⁻³² cycles | accumulated phase, optional |

Planned (#GORKA-11):

* GPS (PRN 1–32, L1/L2/L5)
* Galileo (E1/E5)
* BeiDou (BDS-2/BDS-3)

## no_std Mode

The codec works without the standard library (only `alloc`):

```toml
[dependencies]
gorka = { version = "0.1", default-features = false, features = ["alloc"] }
```

Available in `no_std + alloc`:

* `GlonassEncoder::encode_chunk` / `GlonassDecoder::decode_chunk`
* `BitReader`, `BitWriter`
* `encode_i64`, `decode_i64`, `delta_of_delta_i64`
* `GnssFrame`, `GlonassSample`

Requires `std`:

* `gorka::io` (ChunkWriter, ChunkReader)
* `impl std::error::Error for GorkaError`

## Testing

```zsh
cargo test
cargo nextest run
cargo test --lib
cargo test --test codec_property
cargo test --test bit_property
cargo test --test compression_ratio -- --nocapture
cargo bench
```

## Documentation

```zsh
cargo doc --no-deps --open
cargo run --example basic_encode
cargo run --example streaming
cargo run --example no_std_demo
```

* Format specification: [`docs/FORMAT.md`](docs/FORMAT.md)
* Project structure: [`docs/PROJECT_STRUCTURE.md`](docs/PROJECT_STRUCTURE.md)

## Platform support

- `std` (default)
- `no_std` (core encoding/decoding without the standard library)

## Project Status

⚠️ Active development. Format and APIs may change before the first stable release.

Completed for v0.1.0: bit-IO, encoder, decoder, GnssFrame, IO layer, tests, benchmarks,
FORMAT.md.

Planned: GPS/Galileo/BeiDou, streaming API, entropy coding, SIMD optimizations.

## License

MIT OR Apache-2.0
````
