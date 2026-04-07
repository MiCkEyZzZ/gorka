use crate::{DbHz, GalSvn, GnssMeasurement, GorkaError, Hertz, MilliHz, Millimeter, SatelliteId};

/// GPS E1 (L1) carrier frequency (Hz)
pub const GAL_E1_FREQ: Hertz = Hertz(1_575_420_000);

/// GPS E5a carrier frequency (Hz)
pub const GAL_E5A_FREQ: Hertz = Hertz(1_176_450_000);

/// GPS E5b carrier frequency (Hz)
pub const GAL_E5B_FREQ: Hertz = Hertz(1_207_140_000);

#[derive(Debug, Clone, PartialEq)]
pub struct GalileoSample {
    pub timestamp_ms: u64,
    pub svn: GalSvn,
    pub cn0_dbhz: DbHz,
    pub pseudorange_mm: Millimeter,
    pub doppler_millihz: MilliHz,
    pub carrier_phase_cycles: Option<i64>,
}

impl GalileoSample {
    pub const PSEUDORANGE_MIN_MM: Millimeter = Millimeter::new(23_222_000_000);
    pub const PSEUDORANGE_MAX_MM: Millimeter = Millimeter::new(29_000_000_000);
    pub const DOPPLER_MAX_MILLIHZ: MilliHz = MilliHz::new(4_500_000);

    #[inline]
    pub fn validate_svn(&self) -> Result<(), GorkaError> {
        let svn = self.svn.get();

        if !(GalSvn::MIN..=GalSvn::MAX).contains(&svn) {
            return Err(GorkaError::InvalidSvn(svn));
        }

        Ok(())
    }

    #[inline]
    pub fn validate_pseudorange(&self) -> Result<(), GorkaError> {
        let mm = self.pseudorange_mm.as_i64();

        if !(Self::PSEUDORANGE_MIN_MM.0..=Self::PSEUDORANGE_MAX_MM.0).contains(&mm) {
            return Err(GorkaError::InvalidPseudorange(mm));
        }

        Ok(())
    }

    #[inline]
    pub fn validate_doppler(&self) -> Result<(), GorkaError> {
        let abs = self.doppler_millihz.as_i32().abs();

        if abs > Self::DOPPLER_MAX_MILLIHZ.0 {
            return Err(GorkaError::InvalidDoppler(abs));
        }

        Ok(())
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.is_valid_svn() && self.is_valid_pseudorange() && self.is_valid_doppler()
    }

    #[inline]
    pub fn is_valid_svn(&self) -> bool {
        let svn = self.svn.get();
        (GalSvn::MIN..=GalSvn::MAX).contains(&svn)
    }

    #[inline]
    pub fn is_valid_pseudorange(&self) -> bool {
        let mm = self.pseudorange_mm.as_i64();
        (Self::PSEUDORANGE_MIN_MM.0..=Self::PSEUDORANGE_MAX_MM.0).contains(&mm)
    }

    #[inline]
    pub fn is_valid_doppler(&self) -> bool {
        let mhz = self.doppler_millihz.abs().as_i32();
        mhz <= Self::DOPPLER_MAX_MILLIHZ.0
    }
}

impl GnssMeasurement for GalileoSample {
    fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }

    fn cn0_dbhz(&self) -> DbHz {
        self.cn0_dbhz
    }

    fn pseudorange_mm(&self) -> Millimeter {
        self.pseudorange_mm
    }

    fn doppler_millihz(&self) -> MilliHz {
        self.doppler_millihz
    }

    fn carrier_phase_cycles(&self) -> Option<i64> {
        self.carrier_phase_cycles
    }

    fn satellite_id(&self) -> super::SatelliteId {
        SatelliteId::Galileo(self.svn)
    }

    fn validate(&self) -> Result<(), GorkaError> {
        self.validate_svn()?;
        self.validate_pseudorange()?;
        self.validate_doppler()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConstellationType;

    fn make_gal() -> GalileoSample {
        GalileoSample {
            timestamp_ms: 1_700_000_000_000,
            svn: GalSvn(11),
            cn0_dbhz: DbHz(40),
            pseudorange_mm: Millimeter::new(24_000_000_000),
            doppler_millihz: MilliHz::new(2_000_000),
            carrier_phase_cycles: None,
        }
    }

    #[test]
    fn test_satellite_id_is_galileo() {
        let s = make_gal();
        let id = s.satellite_id();

        assert_eq!(id.constellation(), ConstellationType::Galileo);
    }

    #[test]
    fn test_satellite_id_contains_svn() {
        let s = make_gal();

        match s.satellite_id() {
            SatelliteId::Galileo(svn) => assert_eq!(svn.get(), 11),
            _ => panic!("expected Galileo"),
        }
    }

    #[test]
    fn test_validate_ok() {
        assert!(make_gal().validate().is_ok());
    }

    #[test]
    fn test_is_valid_ok() {
        assert!(make_gal().is_valid());
    }

    #[test]
    fn test_validate_bad_svn() {
        let s = GalileoSample {
            svn: GalSvn(37),
            ..make_gal()
        };

        assert!(matches!(s.validate(), Err(GorkaError::InvalidSvn(37))));
        assert!(!s.is_valid());
    }

    #[test]
    fn test_svn_boundary() {
        let s_min = GalileoSample {
            svn: GalSvn(GalSvn::MIN),
            ..make_gal()
        };
        let s_max = GalileoSample {
            svn: GalSvn(GalSvn::MAX),
            ..make_gal()
        };

        assert!(s_min.validate_svn().is_ok());
        assert!(s_max.validate_svn().is_ok());
    }

    #[test]
    fn test_validate_pseudorange_low() {
        let s = GalileoSample {
            pseudorange_mm: Millimeter::new(20_000_000_000),
            ..make_gal()
        };

        assert!(matches!(
            s.validate(),
            Err(GorkaError::InvalidPseudorange(_))
        ));
        assert!(!s.is_valid());
    }

    #[test]
    fn test_validate_pseudorange_high() {
        let s = GalileoSample {
            pseudorange_mm: Millimeter::new(30_000_000_000),
            ..make_gal()
        };

        assert!(matches!(
            s.validate(),
            Err(GorkaError::InvalidPseudorange(_))
        ));
        assert!(!s.is_valid());
    }

    #[test]
    fn test_pseudorange_boundary() {
        let s_min = GalileoSample {
            pseudorange_mm: GalileoSample::PSEUDORANGE_MIN_MM,
            ..make_gal()
        };
        let s_max = GalileoSample {
            pseudorange_mm: GalileoSample::PSEUDORANGE_MAX_MM,
            ..make_gal()
        };

        assert!(s_min.validate_pseudorange().is_ok());
        assert!(s_max.validate_pseudorange().is_ok());
    }

    #[test]
    fn test_validate_doppler_ok_positive() {
        let s = make_gal();
        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_doppler_ok_negative() {
        let s = GalileoSample {
            doppler_millihz: MilliHz::new(-2_000_000),
            ..make_gal()
        };
        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_doppler_too_large() {
        let s = GalileoSample {
            doppler_millihz: MilliHz::new(10_000_000),
            ..make_gal()
        };
        assert!(matches!(s.validate(), Err(GorkaError::InvalidDoppler(_))));
        assert!(!s.is_valid());
    }

    #[test]
    fn test_doppler_boundary() {
        let s = GalileoSample {
            doppler_millihz: GalileoSample::DOPPLER_MAX_MILLIHZ,
            ..make_gal()
        };
        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_vs_is_valid_consistency() {
        let s = make_gal();
        assert_eq!(s.validate().is_ok(), s.is_valid());
    }

    #[test]
    fn test_galileo_frequencies() {
        assert_eq!(GAL_E1_FREQ.0, 1_575_420_000);
        assert_eq!(GAL_E5A_FREQ.0, 1_176_450_000);
        assert_eq!(GAL_E5B_FREQ.0, 1_207_140_000);
    }
}
