use crate::{DbHz, GnssMeasurement, GorkaError, GpsPrn, Hertz, MilliHz, Millimeter, SatelliteId};

/// GPS L1 carrier frequency (Hz)
pub const GPS_L1_FREQ: Hertz = Hertz(1_575_420_000);

/// GPS L2 carrier frequency (Hz)
pub const GPS_L2_FREQ: Hertz = Hertz(1_227_600_000);

#[derive(Debug, Clone, PartialEq)]
pub struct GpsSample {
    pub timestamp_ms: u64,
    pub prn: GpsPrn,
    pub cn0_dbhz: DbHz,
    pub pseudorange_mm: Millimeter,
    pub doppler_millihz: MilliHz,
    pub carrier_phase_cycles: Option<i64>,
}

impl GpsSample {
    pub const PSEUDORANGE_MIN_MM: Millimeter = Millimeter(20_200_000_000);
    pub const PSEUDORANGE_MAX_MM: Millimeter = Millimeter(25_600_000_000);
    pub const DOPPLER_MAX_MILLIHZ: MilliHz = MilliHz(6_000_000);

    #[inline]
    pub fn validate_prn(&self) -> Result<(), GorkaError> {
        let prn = self.prn.get();

        if !(GpsPrn::MIN..=GpsPrn::MAX).contains(&prn) {
            return Err(GorkaError::InvalidPrn(prn));
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
        self.is_valid_prn() && self.is_valid_pseudorange() && self.is_valid_doppler()
    }

    #[inline]
    pub fn is_valid_prn(&self) -> bool {
        let prn = self.prn.get();
        (GpsPrn::MIN..=GpsPrn::MAX).contains(&prn)
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

impl GnssMeasurement for GpsSample {
    #[inline]
    fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }

    #[inline]
    fn cn0_dbhz(&self) -> DbHz {
        self.cn0_dbhz
    }

    #[inline]
    fn pseudorange_mm(&self) -> Millimeter {
        self.pseudorange_mm
    }

    #[inline]
    fn doppler_millihz(&self) -> MilliHz {
        self.doppler_millihz
    }

    #[inline]
    fn carrier_phase_cycles(&self) -> Option<i64> {
        self.carrier_phase_cycles
    }

    #[inline]
    fn satellite_id(&self) -> SatelliteId {
        SatelliteId::Gps(self.prn)
    }

    #[inline]
    fn validate(&self) -> Result<(), GorkaError> {
        self.validate_prn()?;
        self.validate_pseudorange()?;
        self.validate_doppler()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConstellationType;

    fn make_gps() -> GpsSample {
        GpsSample {
            timestamp_ms: 1_700_000_000_000,
            prn: GpsPrn(7),
            cn0_dbhz: DbHz(42),
            pseudorange_mm: Millimeter::new(22_000_000_000),
            doppler_millihz: MilliHz::new(1_500_000),
            carrier_phase_cycles: Some(9_876_543),
        }
    }

    #[test]
    fn test_satellite_id_is_gps() {
        let s = make_gps();
        let id = s.satellite_id();

        assert_eq!(id.constellation(), ConstellationType::Gps);
    }

    #[test]
    fn test_satellite_id_contains_prn() {
        let s = make_gps();

        match s.satellite_id() {
            SatelliteId::Gps(prn) => assert_eq!(prn.get(), 7),
            _ => panic!("expected GPS"),
        }
    }

    #[test]
    fn test_validate_ok() {
        assert!(make_gps().validate().is_ok());
    }

    #[test]
    fn test_is_valid_ok() {
        assert!(make_gps().is_valid());
    }

    #[test]
    fn test_validate_bad_prn() {
        let s = GpsSample {
            prn: GpsPrn(33),
            ..make_gps()
        };

        assert!(matches!(s.validate(), Err(GorkaError::InvalidPrn(33))));
        assert!(!s.is_valid());
    }

    #[test]
    fn test_prn_boundary() {
        let s_min = GpsSample {
            prn: GpsPrn(1),
            ..make_gps()
        };
        let s_max = GpsSample {
            prn: GpsPrn(32),
            ..make_gps()
        };

        assert!(s_min.validate_prn().is_ok());
        assert!(s_max.validate_prn().is_ok());
    }

    #[test]
    fn test_validate_pseudorange_low() {
        let s = GpsSample {
            pseudorange_mm: Millimeter::new(1_000_000_000),
            ..make_gps()
        };

        assert!(matches!(
            s.validate(),
            Err(GorkaError::InvalidPseudorange(_))
        ));

        assert!(!s.is_valid());
    }

    #[test]
    fn test_validate_pseudorange_high() {
        let s = GpsSample {
            pseudorange_mm: Millimeter::new(30_000_000_000),
            ..make_gps()
        };

        assert!(matches!(
            s.validate(),
            Err(GorkaError::InvalidPseudorange(_))
        ));

        assert!(!s.is_valid());
    }

    #[test]
    fn test_pseudorange_boundary() {
        let s_min = GpsSample {
            pseudorange_mm: GpsSample::PSEUDORANGE_MIN_MM,
            ..make_gps()
        };

        let s_max = GpsSample {
            pseudorange_mm: GpsSample::PSEUDORANGE_MAX_MM,
            ..make_gps()
        };

        assert!(s_min.validate_pseudorange().is_ok());
        assert!(s_max.validate_pseudorange().is_ok());
    }

    #[test]
    fn test_validate_doppler_ok_positive() {
        let s = make_gps();
        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_doppler_ok_negative() {
        let s = GpsSample {
            doppler_millihz: MilliHz::new(-1_500_000),
            ..make_gps()
        };

        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_doppler_too_large() {
        let s = GpsSample {
            doppler_millihz: MilliHz::new(10_000_000),
            ..make_gps()
        };

        assert!(matches!(s.validate(), Err(GorkaError::InvalidDoppler(_))));

        assert!(!s.is_valid());
    }

    #[test]
    fn test_doppler_boundary() {
        let s = GpsSample {
            doppler_millihz: GpsSample::DOPPLER_MAX_MILLIHZ,
            ..make_gps()
        };

        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_vs_is_valid_consistency() {
        let s = make_gps();

        assert_eq!(s.validate().is_ok(), s.is_valid());
    }

    #[test]
    fn test_gps_frequencies() {
        assert_eq!(GPS_L1_FREQ.0, 1_575_420_000);
        assert_eq!(GPS_L2_FREQ.0, 1_227_600_000);
    }
}
