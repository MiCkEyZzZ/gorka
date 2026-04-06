/// GNSS observation types and frame assembly for the gorka positioning engine.
///
/// This module provides the core data structures for working with raw GNSS
/// measurements:
///
/// * [`GlonassSample`] — a single satellite observation (pseudorange, Doppler,
///   carrier phase, signal strength) for one GLONASS FDMA frequency slot.
/// * [`GnssFrame`] — a fixed-capacity, slot-sorted collection of
///   [`GlonassSample`]s that all share the same measurement epoch.
/// * [`Millimeter`] / [`MilliHz`] — integer newtypes for lossless fixed-point
///   representation of distances and frequencies.
///
/// # Design goals
/// * **`no_std` compatible** — all storage uses fixed-size arrays; no heap
///   allocation is required.
/// * **Integer-first arithmetic** — physical quantities are stored as `i64`
///   millimetres or `i32` millihertz to eliminate floating-point rounding
///   errors in the hot path.  `f64` helpers (`*_approx` methods) are provided
///   only for display and are gated behind the `std` feature flag.
pub mod frame;
pub mod glonass;
pub mod types;

pub use frame::*;
pub use glonass::*;
pub use types::*;
