//! GNSS data model — constellations, measurements and frames.
//!
//! # Modules
//!
//! | Module          | Contents                                               |
//! |-----------------|--------------------------------------------------------|
//! | `constellation` | [`ConstellationType`], [`SatelliteId`]                 |
//! | `measurement`   | [`GnssMeasurement`] trait, [`GnssSample`]              |
//! | `glonass`       | [`GlonassSample`] (FDMA, codec-ready)                  |
//! | `gps`           | [`GpsSample`] (CDMA, data model only)                  |
//! | `galileo`       | [`GalileoSample`] (CDMA, data model only)              |
//! | `beidou`        | [`BeidouSample`] (CDMA/FDMA, data model only)          |
//! | `frame`         | [`GnssFrame`] — single-epoch multi-satellite buffer    |
//! | `types`         | [`Millimeter`], [`MilliHz`] — fixed-point newtypes     |

pub mod beidou;
pub mod constellation;
pub mod frame;
pub mod galileo;
pub mod glonass;
pub mod gps;
pub mod measurement;
pub mod types;

pub use beidou::*;
pub use constellation::*;
pub use frame::*;
pub use galileo::*;
pub use glonass::*;
pub use gps::*;
pub use measurement::*;
pub use types::*;
