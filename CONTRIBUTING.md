# Contributing to Gorka

Thanks for your interest in the project! Gorka is a library for GNSS telemetry compression.
All contributions are welcome: bug fixes, new features, documentation, and tests.

## Contents

- [Quick Start for Contributors](#quick-start-for-contributors)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Tests](#tests)
- [How to Add Support for a New Constellation](#how-to-add-support-for-a-new-constellation)
- [Working with Issues](#working-with-issues)
- [Pull Request Checklist](#pull-request-checklist)

## Quick Start for Contributors

```zsh
# Clone the repository
git clone https://github.com/MiCkEyZzZ/gorka
cd gorka

# Install tools
cargo install cargo-nextest taplo-cli

# Check that everything works
just dev
# or run manually:
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
```

All development commands:

```zsh
just fmt-all   # format Rust + TOML
just lint      # clippy
just test-next # run all tests through nextest
just bench     # benchmarks
just dev       # fmt + check (recommended before committing)
```

## Project Structure

```text
src/
├── bits/          — BitReader, BitWriter (bit-level IO)
├── codec/
│   ├── encoder.rs — GlonassEncoder (main encoder)
│   ├── decoder.rs — GlonassDecoder (encoder mirror)
│   ├── delta.rs   — delta / delta-of-delta calculations
│   ├── zigzag.rs  — encode_i64 / decode_i64
│   └── format/    — FormatVersion, CHUNK_MAGIC, header layout
├── gnss/
│   ├── types.rs   — Millimeter, MilliHz (newtype wrappers)
│   ├── glonass.rs — GlonassSample, validation, helper methods
│   ├── frame.rs   — GnssFrame (epoch buffer, fixed-size array)
│   └── mod.rs
├── io/mod.rs      — ChunkWriter, ChunkReader (std only)
└── error.rs       — GorkaError

tests/             — integration tests
benches/           — Criterion benchmarks
docs/              — specifications and documentation
examples/          — working examples
```

Key invariant: **the encoder and decoder must be symmetric**.
Any change in `encode_delta` requires a matching change in `decode_delta`, and
vice versa.

## Code Style

The project follows standard Rust style with a few additional rules:

- Formatting through `cargo fmt` (settings in `rustfmt.toml`)
- Clippy must pass with no warnings (`-D warnings`)
- Documentation comments should be in **English** for public API items (`///`)
- Avoid `unwrap()` in production code except in places guarded by `debug_assert!`
- `#[inline(always)]` only for hot paths in bit-IO
- Comments for bucket schemes are required: `// '10' + 7b zigzag`

## Tests

Every new function should have:

1. **Unit tests** inside a `#[cfg(test)]` block in the module — basic correctness
2. **Roundtrip tests** — encode → decode must return identical data
3. **Edge-case tests** — boundary values (empty chunk, max slot, None phase)
4. **Property tests** in `tests/` using `proptest` — random valid data

Rule: **if you add a bucket in the encoder, add a test for that bucket**.

```zsh
# Run a specific test
cargo test test_roundtrip_carrier_phase_reacquired

# Show println! output
cargo test --test compression_ratio -- --nocapture
```

## How to Add Support for a New Constellation

This is the main extension point of Gorka. Below is a step-by-step plan for adding,
for example, **GPS**.

### Step 1: Define the Data Type

Create `src/gnss/gps.rs`:

```rust
use crate::{GorkaError, MilliHz, Millimeter, DbHz};

/// One GPS L1 C/A observation.
///
/// PRN = Pseudo-Random Noise code number, identifies the satellite.
/// GPS L1 C/A: 1575.42 MHz, all satellites share the same frequency (CDMA).
#[derive(Debug, Clone, PartialEq)]
pub struct GpsSample {
    pub timestamp_ms:         u64,
    pub prn:                  u8,          // 1..=32
    pub cn0_dbhz:             DbHz,
    pub pseudorange_mm:       Millimeter,
    pub doppler_millihz:      MilliHz,
    pub carrier_phase_cycles: Option<i64>,
}

impl GpsSample {
    pub const PRN_MIN: u8 = 1;
    pub const PRN_MAX: u8 = 32;

    pub fn validate_prn(&self) -> Result<(), GorkaError> {
        if !(Self::PRN_MIN..=Self::PRN_MAX).contains(&self.prn) {
            return Err(GorkaError::InvalidPrn(self.prn));
        }
        Ok(())
    }
    // ...
}
```

### Step 2: Add It to `gnss/mod.rs`

```rust
pub mod gps;
pub use gps::GpsSample;
```

### Step 3: Implement the Encoder

Create `src/codec/gps_encoder.rs`. Copy the structure from `encoder.rs`
and adapt it:

- `slot` → `prn` (PRN 1..32, 5 bits instead of 4)
- Remove per-slot FDMA state (GPS uses CDMA — one frequency)
- Doppler: delta without FDMA correction (all satellites on the same carrier)

```rust
pub struct GpsEncoder;

impl GpsEncoder {
    pub fn encode_chunk(samples: &[GpsSample]) -> Result<Vec<u8>, GorkaError> {
        // Similar to GlonassEncoder, but:
        // - state.last_prn instead of last_slot
        // - last_doppler: Option<i32> (single value, not an array)
        // - verbatim: 1B prn instead of 1B slot
        todo!()
    }
}
```

### Step 4: Implement the Decoder

`src/codec/gps_decoder.rs` — exact mirror of the encoder. Add a test:

```rust
#[test]
fn test_gps_roundtrip() {
    let samples: Vec<GpsSample> = (1..=10).map(|prn| GpsSample {
        prn,
        // ...
    }).collect();
    let enc = GpsEncoder::encode_chunk(&samples).unwrap();
    let dec = GpsDecoder::decode_chunk(&enc).unwrap();
    assert_eq!(samples, dec);
}
```

### Step 5: Update the Public API

In `src/lib.rs`:

```rust
pub use codec::{GpsDecoder, GpsEncoder, /* ... */};
pub use gnss::{GpsSample, /* ... */};
```

### Step 6: Update `FORMAT.md`

Add a section to `docs/FORMAT.md`:

```markdown
## 9. GPS chunk format (V2)

Chunk version V2 adds...
```

A format change requires a new `FormatVersion::V2`.

### Step 7: Update Tests and Benchmarks

- `tests/gps_roundtrip.rs` — full roundtrip tests
- `benches/encode_bench.rs` — add `bench_encode_gps_smooth`
- `tests/compression_ratio.rs` — add `compression_ratio_gps_smooth`

### Step 8: Add an Example

`examples/multi_gnss.rs` — show joint usage of GLONASS and GPS.

### Key Rules When Adding Constellations

- **One type — one encoder/decoder**. Do not create a generic "GNSS encoder".
- **Versioning**: any wire format change requires a new `FormatVersion`.
- **Fixed-point everywhere**: no `f32`/`f64` in the codec path.
- **Symmetry tests**: every `encode_X` / `decode_X` pair must have roundtrip coverage.
- **Per-signal state**: if a signal uses FDMA (GLONASS), use an array of states;
  if it uses CDMA (GPS, Galileo), use a single state.

## Working with Issues

- **Bug**: provide a minimal reproduction, and describe expected vs actual behavior
- **Feature**: open an issue with the `enhancement` label before implementing
- **Breaking change**: discuss it in an issue; it requires a major version bump

## Pull Request Checklist

- **Scope**: what area of the codebase this touches (e.g. `gorka`).
- **Summary**: brief description of the change.
- **Testing**: how did you verify it works? (unit tests, manual steps).
- **Checklist**:
  - [ ] `cargo fmt --check`
  - [ ] `taplo format`
  - [ ] `cargo clippy -- -D warnings`
  - [ ] `cargo test`
  - [ ] New tests added / existing tests updated
  - [ ] Added the necessary rustdoc comments.
  - [ ] Changelog updated if applicable
