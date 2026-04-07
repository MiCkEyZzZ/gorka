//! Satellite contellation types and identifiers.
//!
//! Provides [`ConstellationType`] and [`SatelliteId`] - the foundation for
//! multi-GNSS support. These types are purely decriptive and do not affect the
//! wire format of any existing chunk.

use crate::gnss::{BdsPrn, GalSvn, GloSlot, GpsPrn};

/// Unique satellite identifier within a constellation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SatelliteId {
    Glonass(GloSlot),
    Gps(GpsPrn),
    Galileo(GalSvn),
    Beidou(BdsPrn),
}

/// GNSS constellation (access method).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConstellationType {
    Glonass,
    Gps,
    Galileo,
    Beidou,
}

impl ConstellationType {
    /// Return a short ASCII identifier for the constellation.
    pub const fn abbrev(self) -> &'static str {
        match self {
            Self::Glonass => "GLO",
            Self::Gps => "GPS",
            Self::Galileo => "GAL",
            Self::Beidou => "BDS",
        }
    }

    /// Return `true` if the constellation uses FDMA (GLONASS).
    pub fn is_fdma(self) -> bool {
        matches!(self, Self::Glonass)
    }

    pub const fn order(self) -> u8 {
        match self {
            Self::Glonass => 0,
            Self::Gps => 1,
            Self::Galileo => 2,
            Self::Beidou => 3,
        }
    }
}

impl SatelliteId {
    #[inline]
    pub const fn constellation(self) -> ConstellationType {
        match self {
            Self::Glonass(_) => ConstellationType::Glonass,
            Self::Gps(_) => ConstellationType::Gps,
            Self::Galileo(_) => ConstellationType::Galileo,
            Self::Beidou(_) => ConstellationType::Beidou,
        }
    }

    /// Creates a `SatelliteId` for a GLONASS satellite from its FDMA slot `k`.
    pub const fn glonass(slot: GloSlot) -> Self {
        Self::Glonass(slot)
    }

    /// Creates a `SatelliteId` for a GPS satellite from its PRN.
    pub const fn gps(prn: GpsPrn) -> Self {
        Self::Gps(prn)
    }

    /// Creates a `SatelliteId` for a Galileo satellite from its SVN.
    pub const fn galileo(svn: GalSvn) -> Self {
        Self::Galileo(svn)
    }

    /// Creates a `SatelliteId` for a BeiDou satellite from its PRN.
    pub const fn beidou(prn: BdsPrn) -> Self {
        Self::Beidou(prn)
    }

    /// Returns the GLONASS FDMA slot `k` for this satellite.
    ///
    /// Returns `None` if this is not a GLONASS satellite.
    pub const fn glonass_slot(self) -> Option<GloSlot> {
        match self {
            Self::Glonass(slot) => Some(slot),
            _ => None,
        }
    }

    #[inline]
    pub fn to_wire(self) -> (ConstellationType, u8) {
        match self {
            Self::Glonass(slot) => {
                let k = slot.get();
                debug_assert!((-7..=6).contains(&k));
                (ConstellationType::Glonass, (k + 7) as u8)
            }
            Self::Gps(prn) => (ConstellationType::Gps, prn.get()),
            Self::Galileo(svn) => (ConstellationType::Galileo, svn.get()),
            Self::Beidou(prn) => (ConstellationType::Beidou, prn.get()),
        }
    }
}

impl core::fmt::Display for ConstellationType {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str(self.abbrev())
    }
}

impl core::fmt::Display for SatelliteId {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        match self {
            Self::Glonass(slot) => write!(f, "GLO{:02}", slot.get() + 7),
            Self::Gps(prn) => write!(f, "GPS{:02}", prn.get()),
            Self::Galileo(svn) => write!(f, "GAL{:02}", svn.get()),
            Self::Beidou(prn) => write!(f, "BDS{:02}", prn.get()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glonass_slot_roundtrip() {
        for k in -7_i8..=6 {
            let sid = SatelliteId::glonass(GloSlot(k));

            assert_eq!(sid.constellation(), ConstellationType::Glonass);
            assert_eq!(sid.glonass_slot(), Some(GloSlot(k)));
        }
    }

    #[test]
    fn test_gps_prn() {
        let sid = SatelliteId::gps(GpsPrn(15));

        assert_eq!(sid.constellation(), ConstellationType::Gps);
        assert_eq!(sid.glonass_slot(), None);

        match sid {
            SatelliteId::Gps(prn) => assert_eq!(prn.get(), 15),
            _ => panic!("expected GPS"),
        }
    }

    #[test]
    fn test_galileo_svn() {
        let sid = SatelliteId::galileo(GalSvn(15));

        assert_eq!(sid.constellation(), ConstellationType::Galileo);
        assert_eq!(sid.glonass_slot(), None);

        match sid {
            SatelliteId::Galileo(svn) => assert_eq!(svn.get(), 15),
            _ => panic!("expected Galileo"),
        }
    }

    #[test]
    fn test_beidou_prn() {
        let sid = SatelliteId::beidou(BdsPrn(15));

        assert_eq!(sid.constellation(), ConstellationType::Beidou);
        assert_eq!(sid.glonass_slot(), None);

        match sid {
            SatelliteId::Beidou(prn) => assert_eq!(prn.get(), 15),
            _ => panic!("expected Beidou"),
        }
    }

    #[test]
    fn test_abbrev() {
        assert_eq!(ConstellationType::Glonass.abbrev(), "GLO");
        assert_eq!(ConstellationType::Gps.abbrev(), "GPS");
        assert_eq!(ConstellationType::Galileo.abbrev(), "GAL");
        assert_eq!(ConstellationType::Beidou.abbrev(), "BDS");
    }

    #[test]
    fn test_display_satellite_id() {
        let s = alloc::format!("{}", SatelliteId::gps(GpsPrn(5)));
        assert_eq!(s, "GPS05");

        let g = alloc::format!("{}", SatelliteId::glonass(GloSlot(1)));
        assert_eq!(g, "GLO08");
    }

    #[test]
    fn test_ordering() {
        assert!(ConstellationType::Glonass.order() < ConstellationType::Gps.order());
        assert!(ConstellationType::Gps.order() < ConstellationType::Galileo.order());
        assert!(ConstellationType::Galileo.order() < ConstellationType::Beidou.order());
    }

    #[test]
    fn test_to_wire() {
        let gps = SatelliteId::gps(GpsPrn(5));
        assert_eq!(gps.to_wire(), (ConstellationType::Gps, 5));

        let glo = SatelliteId::glonass(GloSlot(1));
        assert_eq!(glo.to_wire(), (ConstellationType::Glonass, 8));
    }
}
