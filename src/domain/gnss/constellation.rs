//! Satellite contellation types and identifiers.
//!
//! Provides [`ConstellationType`] and [`SatelliteId`] - the foundation for
//! multi-GNSS support. These types are purely decriptive and do not affect the
//! wire format of any existing chunk.

use crate::{BdsPrn, GalSvn, GloSlot, GorkaError, GpsPrn};

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
#[repr(u8)]
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
                (ConstellationType::Glonass, (k - GloSlot::MIN) as u8)
            }
            Self::Gps(prn) => (ConstellationType::Gps, prn.get()),
            Self::Galileo(svn) => (ConstellationType::Galileo, svn.get()),
            Self::Beidou(prn) => (ConstellationType::Beidou, prn.get()),
        }
    }

    pub fn from_wire(
        c: ConstellationType,
        id: u8,
    ) -> Result<Self, GorkaError> {
        match c {
            ConstellationType::Glonass => {
                let k = id as i8 + GloSlot::MIN;
                Ok(Self::Glonass(GloSlot::new(k)?))
            }
            ConstellationType::Gps => Ok(Self::Gps(GpsPrn::new(id)?)),
            ConstellationType::Galileo => Ok(Self::Galileo(GalSvn::new(id)?)),
            ConstellationType::Beidou => Ok(Self::Beidou(BdsPrn::new(id)?)),
        }
    }

    pub const fn display_id(self) -> u8 {
        match self {
            Self::Glonass(slot) => (slot.get() - GloSlot::MIN) as u8,
            Self::Gps(prn) => prn.get(),
            Self::Galileo(svn) => svn.get(),
            Self::Beidou(prn) => prn.get(),
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
        let c = self.constellation();

        write!(f, "{}{:02}", c.abbrev(), self.display_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glonass_slot_roundtrip() {
        for k in -7_i8..=6 {
            let sid = SatelliteId::glonass(GloSlot::new(k).unwrap());

            assert_eq!(sid.constellation(), ConstellationType::Glonass);
            assert_eq!(sid.glonass_slot(), Some(GloSlot::new(k).unwrap()));
        }
    }

    #[test]
    fn test_gps_prn() {
        let sid = SatelliteId::gps(GpsPrn::new(15).unwrap());

        assert_eq!(sid.constellation(), ConstellationType::Gps);
        assert_eq!(sid.glonass_slot(), None);

        match sid {
            SatelliteId::Gps(prn) => assert_eq!(prn.get(), 15),
            _ => panic!("expected GPS"),
        }
    }

    #[test]
    fn test_galileo_svn() {
        let sid = SatelliteId::galileo(GalSvn::new(15).unwrap());

        assert_eq!(sid.constellation(), ConstellationType::Galileo);
        assert_eq!(sid.glonass_slot(), None);

        match sid {
            SatelliteId::Galileo(svn) => assert_eq!(svn.get(), 15),
            _ => panic!("expected Galileo"),
        }
    }

    #[test]
    fn test_beidou_prn() {
        let sid = SatelliteId::beidou(BdsPrn::new(15).unwrap());

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
        let s = alloc::format!("{}", SatelliteId::gps(GpsPrn::new(5).unwrap()));
        assert_eq!(s, "GPS05");

        let g = alloc::format!("{}", SatelliteId::glonass(GloSlot::new(1).unwrap()));
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
        let gps = SatelliteId::gps(GpsPrn::new(5).unwrap());
        assert_eq!(gps.to_wire(), (ConstellationType::Gps, 5));

        let glo = SatelliteId::glonass(GloSlot::new(1).unwrap());
        assert_eq!(glo.to_wire(), (ConstellationType::Glonass, 8));
    }

    #[test]
    fn test_glonass_slot_edges() {
        let min_slot = SatelliteId::glonass(GloSlot::new(-7).unwrap());
        let max_slot = SatelliteId::glonass(GloSlot::new(6).unwrap());

        assert_eq!(min_slot.glonass_slot(), Some(GloSlot::new(-7).unwrap()));
        assert_eq!(min_slot.to_wire(), (ConstellationType::Glonass, 0));
        assert_eq!(max_slot.glonass_slot(), Some(GloSlot::new(6).unwrap()));
        assert_eq!(max_slot.to_wire(), (ConstellationType::Glonass, 13))
    }

    #[test]
    fn test_display_glonass_edges() {
        assert_eq!(
            alloc::format!("{}", SatelliteId::glonass(GloSlot::new(-7).unwrap())),
            "GLO00"
        );
        assert_eq!(
            alloc::format!("{}", SatelliteId::glonass(GloSlot::new(6).unwrap())),
            "GLO13"
        );
    }

    #[test]
    fn test_all_constellations_to_wire_and_constellation() {
        let satellites: [(SatelliteId, ConstellationType, u8); 4] = [
            (
                SatelliteId::glonass(GloSlot::new(0).unwrap()),
                ConstellationType::Glonass,
                SatelliteId::glonass(GloSlot::new(0).unwrap()).to_wire().1,
            ),
            (
                SatelliteId::gps(GpsPrn::new(32).unwrap()),
                ConstellationType::Gps,
                32,
            ),
            (
                SatelliteId::galileo(GalSvn::new(1).unwrap()),
                ConstellationType::Galileo,
                1,
            ),
            (
                SatelliteId::beidou(BdsPrn::new(10).unwrap()),
                ConstellationType::Beidou,
                10,
            ),
        ];

        for (sid, ctype, code) in satellites {
            assert_eq!(sid.constellation(), ctype);
            assert_eq!(sid.to_wire(), (ctype, code));
        }
    }

    #[test]
    fn test_display_all_constellations() {
        let satellites: [(SatelliteId, &str); 4] = [
            (SatelliteId::glonass(GloSlot::new(3).unwrap()), "GLO10"),
            (SatelliteId::gps(GpsPrn::new(7).unwrap()), "GPS07"),
            (SatelliteId::galileo(GalSvn::new(1).unwrap()), "GAL01"),
            (SatelliteId::beidou(BdsPrn::new(12).unwrap()), "BDS12"),
        ];

        for (sid, expected) in satellites {
            assert_eq!(alloc::format!("{}", sid), expected);
        }
    }

    #[test]
    fn test_is_fdma() {
        assert!(ConstellationType::Glonass.is_fdma());
        assert!(!ConstellationType::Gps.is_fdma());
        assert!(!ConstellationType::Galileo.is_fdma());
        assert!(!ConstellationType::Beidou.is_fdma());
    }

    #[test]
    fn test_constellation_ordering() {
        assert!(ConstellationType::Glonass.order() < ConstellationType::Gps.order());
        assert!(ConstellationType::Gps.order() < ConstellationType::Galileo.order());
        assert!(ConstellationType::Galileo.order() < ConstellationType::Beidou.order());
    }
}
