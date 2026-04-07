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
//!
//! ## Data Format
//!
//! For a detailed specification of the chunk format, see `docs/FORMAT.md`.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod bits;
pub mod codec;
pub mod error;
pub mod gnss;
pub mod prelude;

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
    BdsPrn, ConstellationType, DbHz, GalSvn, GloSlot, GlonassSample, GnssFrame, GnssMeasurement,
    GnssSample, GpsPrn, GpsSample, Hertz, MilliHz, Millimeter, SatelliteId, GPS_L1_FREQ,
    GPS_L2_FREQ, MAX_GLONASS_SATS,
};
