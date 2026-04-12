//! GNSS data model — constellations, measurements and frames.
//!
//! # Modules
//!
//! | Module          | Contents                                               |
//! |-----------------|--------------------------------------------------------|
//! | `glonass`       | [`GlonassSample`] (FDMA, codec-ready)                  |
//! | `gps`           | [`GpsSample`] (CDMA, data model only)                  |
//! | `galileo`       | [`GalileoSample`] (CDMA, data model only)              |
//! | `beidou`        | [`BeidouSample`] (CDMA/FDMA, data model only)          |

pub mod beidou;
pub mod galileo;
pub mod glonass;
pub mod gps;

pub use beidou::*;
pub use galileo::*;
pub use glonass::*;
pub use gps::*;
