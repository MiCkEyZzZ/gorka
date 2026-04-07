//! Gorka — GNSS Time-Series Compression Codec
//!
//! Gorka is a library for compressing and decompressing GNSS
//! (Global Navigation Satellite System) time series data.
//! Optimized for GLONASS, with planned support for GPS, Galileo, and BeiDou.
//!
//! ## Architecture
//!
//! ```text
//! GlonassSample[]
//!       │
//!       ▼
//! GlonassEncoder::encode_chunk() -> Vec<u8> (chunk)
//!       │
//!       ▼
//! ChunkWriter -> file / storage
//! ChunkReader -> &[u8] (chunk)
//!       │
//!       ▼
//! GlonassDecoder::decode_chunk() -> Vec<GlonassSample>
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use gorka::{
//!     codec::{GlonassDecoder, GlonassEncoder},
//!     GlonassSample, MilliHz, Millimeter,
//! };
//!
//! let samples = vec![GlonassSample {
//!     timestamp_ms: 1_700_000_000_000,
//!     slot: 1,
//!     cn0_dbhz: 42,
//!     pseudorange_mm: Millimeter::new(21_500_000_000),
//!     doppler_millihz: MilliHz::new(1_200_500),
//!     carrier_phase_cycles: None,
//! }];
//!
//! let compressed = GlonassEncoder::encode_chunk(&samples).unwrap();
//! let decoded = GlonassDecoder::decode_chunk(&compressed).unwrap();
//!
//! assert_eq!(samples, decoded);
//! ```
//!
//! ## no_std Support
//!
//! The codec works without the standard library (only requires `alloc`):
//!
//! ```toml
//! gorka = { version = "0.1", default-features = false, features = ["alloc"] }
//! ```
//!
//! The [`io`] module requires `std` and is only available when `feature =
//! "std"`.
//!
//! ## Data Format
//!
//! For a detailed specification of the chunk format, see `docs/FORMAT.md`.
//!
//! See README for usage examples.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod bits;
pub mod codec;
pub mod error;
pub mod gnss;

/// Stream I/O for chunk sequences.
///
/// Available only with the `std` feature.
/// Contains [`io::ChunkWriter`] and [`io::ChunkReader`].
#[cfg(feature = "std")]
pub mod io;

#[allow(deprecated)]
pub use bits::{BitReader, BitWriter};
pub use bits::{BitWrite, RawBitWriter};
pub use codec::{
    decode_i64, delta_i64, delta_of_delta_i64, delta_of_delta_u64, delta_u64, encode_i64,
    reconstruct_from_delta, reconstruct_from_dod, reconstruct_from_dod_u64, CompatibilityInfo,
    FormatVersion, GlonassDecoder, GlonassEncoder, StreamEncoder, VersionUtils, CHUNK_MAGIC,
    STREAM_ENCODER_MIN_BUF_NO_PHASE, STREAM_ENCODER_MIN_BUF_WITH_PHASE,
};
pub use error::GorkaError;
pub use gnss::{
    BdsPrn, ConstellationType, DbHz, GalSvn, GloSlot, GlonassSample, GnssFrame, GpsPrn, Hertz,
    MilliHz, Millimeter, SatelliteId, MAX_GLONASS_SATS,
};
