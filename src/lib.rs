//! Gorka — GNSS time-series compression codec.
//!
//! See README for usage examples.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

pub mod bits;
pub mod codec;
pub mod error;
pub mod gnss;

#[cfg(feature = "std")]
pub mod io;

pub use bits::{BitReader, BitWriter};
pub use codec::{
    decode_i64, delta_i64, delta_of_delta_i64, delta_of_delta_u64, delta_u64, encode_i64,
    reconstruct_from_delta, reconstruct_from_dod, reconstruct_from_dod_u64, CompatibilityInfo,
    FormatVersion, GlonassDecoder, GlonassEncoder, VersionUtils, CHUNK_MAGIC,
};
pub use error::GorkaError;
pub use gnss::{GlonassSample, GnssFrame, MilliHz, Millimeter, MAX_GLONASS_SATS};
