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
//! | `frame`         | [`GnssFrame`] — single-epoch multi-satellite buffer    |

pub mod beidou;
pub mod frame;
pub mod galileo;
pub mod glonass;
pub mod gps;

pub use beidou::*;
pub use frame::*;
pub use galileo::*;
pub use glonass::*;
pub use gps::*;
