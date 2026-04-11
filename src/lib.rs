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
pub mod domain;
pub mod error;
pub mod gnss;
pub mod pipeline;
pub mod prelude;

/// Stream I/O for chunk sequences.
///
/// Available only with the `std` feature.
/// Contains [`io::ChunkWriter`] and [`io::ChunkReader`].
#[cfg(feature = "std")]
pub mod io;

#[allow(deprecated)]
pub use bits::BitReader;
pub use bits::{BitWrite, RawBitWriter};
pub use codec::{
    decode_i64, delta_i64, delta_of_delta_i64, delta_of_delta_u64, delta_u64, encode_i64,
    reconstruct_from_delta, reconstruct_from_dod, reconstruct_from_dod_u64, CompatibilityInfo,
    FormatVersion, VersionUtils, CHUNK_MAGIC,
};
pub use domain::{BdsPrn, DbHz, GalSvn, GloSlot, GpsPrn, Hertz, MilliHz, Millimeter};
pub use error::GorkaError;
pub use gnss::{
    decode_doppler_cdma, encode_doppler_cdma, encode_doppler_fdma, BeidouSample, CdmaState,
    ConstellationType, FdmaState, GalileoSample, GlonassSample, GnssEpoch, GnssFrame,
    GnssMeasurement, GnssSample, GpsSample, SatelliteId, BDS_B1C_FREQ, BDS_B1I_FREQ, BDS_B2A_FREQ,
    CNO_TRACK_THRESHOLD, EMA_SHIFT, GAL_E1_FREQ, GAL_E5A_FREQ, GAL_E5B_FREQ, GPS_L1_FREQ,
    GPS_L2_FREQ, MAX_GLONASS_SATS, N_SLOT,
};
pub use pipeline::{
    GlonassDecoder, GlonassEncoder, StreamEncoder, STREAM_ENCODER_MIN_BUF_NO_PHASE,
    STREAM_ENCODER_MIN_BUF_WITH_PHASE,
};
