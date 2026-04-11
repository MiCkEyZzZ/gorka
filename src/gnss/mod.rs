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

pub mod beidou;
pub mod cdma;
pub mod constellation;
pub mod fdma;
pub mod frame;
pub mod galileo;
pub mod glonass;
pub mod gps;
pub mod measurement;

pub use beidou::*;
pub use cdma::*;
pub use constellation::*;
pub use fdma::*;
pub use frame::*;
pub use galileo::*;
pub use glonass::*;
pub use gps::*;
pub use measurement::*;
