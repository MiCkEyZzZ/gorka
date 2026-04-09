use crate::{DbHz, GloSlot, GnssMeasurement, GorkaError, Hertz, MilliHz, Millimeter, SatelliteId};

#[derive(Debug, Clone, PartialEq)]
pub struct GlonassSample {
    pub timestamp_ms: u64,
    pub slot: GloSlot,
    pub cn0_dbhz: DbHz,
    pub pseudorange_mm: Millimeter,
    pub doppler_millihz: MilliHz,
    pub carrier_phase_cycles: Option<i64>,
}

impl GlonassSample {
    pub const BASE_FREQ: Hertz = Hertz(1_602_000_000);
    pub const FREQ_STEP: Hertz = Hertz(562_500);

    pub const PSEUDORANGE_MIN_MM: Millimeter = Millimeter(19_100_000_000);
    pub const PSEUDORANGE_MAX_MM: Millimeter = Millimeter(25_600_000_000);
    pub const DOPPLER_MAX_MILLIHZ: MilliHz = MilliHz(5_000_000);
    pub const CN0_MIN_TRACKED: u8 = 20;

    #[inline]
    pub fn validate_slot(&self) -> Result<(), GorkaError> {
        let slot = self.slot.get();

        if !(GloSlot::MIN..=GloSlot::MAX).contains(&slot) {
            return Err(GorkaError::InvalidSlot(slot));
        }

        Ok(())
    }

    #[inline]
    pub fn validate_pseudorange(&self) -> Result<(), GorkaError> {
        let mm = self.pseudorange_mm.as_i64();

        if self.pseudorange_mm.as_i64() < Self::PSEUDORANGE_MIN_MM.as_i64()
            || self.pseudorange_mm.as_i64() > Self::PSEUDORANGE_MAX_MM.as_i64()
        {
            return Err(GorkaError::InvalidPseudorange(mm));
        }

        Ok(())
    }

    #[inline]
    pub fn validate_doppler(&self) -> Result<(), GorkaError> {
        let abs = self.doppler_millihz.as_i32().abs();

        if abs > Self::DOPPLER_MAX_MILLIHZ.as_i32() {
            return Err(GorkaError::InvalidDoppler(abs));
        }

        Ok(())
    }

    #[inline]
    pub fn is_tracked(&self) -> bool {
        self.cn0_dbhz.get() >= Self::CN0_MIN_TRACKED
    }

    pub fn carrier_freq_millihz(&self) -> Result<i64, GorkaError> {
        self.validate_slot()?;

        Ok(Self::BASE_FREQ.as_i64() + self.slot.get() as i64 * Self::FREQ_STEP.as_i64())
    }

    #[cfg(feature = "std")]
    pub fn pseudorange_m_approx(&self) -> f64 {
        self.pseudorange_mm.as_i64() as f64 / 1_000.0
    }

    #[cfg(feature = "std")]
    pub fn doppler_hz_approx(&self) -> f64 {
        self.doppler_millihz.as_i32() as f64 / 1_000.0
    }

    /// Returns a zeroed-out placeholder `GlonassSample`.
    ///
    /// All numeric fields are `0`; `carrier_phase_cycles` is `None`.
    /// Useful for initialising fixed-size arrays on the stack without
    /// requiring `Default` or heap allocation.
    ///
    /// Note that a zeroed sample will **not** pass
    /// [`validate`](GlonassSample::validate) (the pseudorange is outside
    /// the plausible window).
    pub fn default_zeroed() -> Self {
        Self {
            timestamp_ms: 0,
            slot: GloSlot::new(0).unwrap(),
            cn0_dbhz: DbHz::new(0).unwrap(),
            pseudorange_mm: Millimeter(0),
            doppler_millihz: MilliHz(0),
            carrier_phase_cycles: None,
        }
    }
}

impl GnssMeasurement for GlonassSample {
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
    fn satellite_id(&self) -> super::SatelliteId {
        SatelliteId::Glonass(self.slot)
    }

    #[inline]
    fn validate(&self) -> Result<(), GorkaError> {
        self.validate_slot()?;
        self.validate_pseudorange()?;
        self.validate_doppler()?;

        Ok(())
    }

    fn is_tracked(&self) -> bool {
        self.is_tracked()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample() -> GlonassSample {
        GlonassSample {
            timestamp_ms: 1_700_000_000_000,
            slot: GloSlot::new(1).unwrap(),
            cn0_dbhz: DbHz::new(42).unwrap(),
            pseudorange_mm: Millimeter::new(21_500_000_000),
            doppler_millihz: MilliHz::new(1_200_500),
            carrier_phase_cycles: Some(0x0001_2345_6789_ABCDi64),
        }
    }

    #[test]
    fn test_slot_valid_range() {
        for k in GloSlot::MIN..=GloSlot::MAX {
            assert!(GloSlot::new(k).is_ok(), "slot {k} must be valid");
        }
    }

    #[test]
    fn test_slot_below_min_rejected() {
        assert!(GloSlot::new(GloSlot::MIN - 1).is_err());
    }

    #[test]
    fn test_slot_above_max_rejected() {
        assert!(GloSlot::new(GloSlot::MAX + 1).is_err());
    }

    #[test]
    fn test_validate_pseudorange_ok() {
        assert!(make_sample().validate_pseudorange().is_ok());
    }

    #[test]
    fn test_validate_pseudorange_boundaries() {
        let min = GlonassSample {
            pseudorange_mm: GlonassSample::PSEUDORANGE_MIN_MM,
            ..make_sample()
        };

        let max = GlonassSample {
            pseudorange_mm: GlonassSample::PSEUDORANGE_MAX_MM,
            ..make_sample()
        };

        assert!(min.validate_pseudorange().is_ok());
        assert!(max.validate_pseudorange().is_ok());
    }

    #[test]
    fn test_validate_pseudorange_out_of_range() {
        let small = GlonassSample {
            pseudorange_mm: Millimeter::new(1_000_000),
            ..make_sample()
        };

        let large = GlonassSample {
            pseudorange_mm: Millimeter::new(99_000_000_000),
            ..make_sample()
        };

        assert!(small.validate_pseudorange().is_err());
        assert!(large.validate_pseudorange().is_err());
    }

    #[test]
    fn test_validate_doppler_ok() {
        let s = make_sample();
        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_doppler_boundaries() {
        let max = GlonassSample {
            doppler_millihz: GlonassSample::DOPPLER_MAX_MILLIHZ,
            ..make_sample()
        };

        let min = GlonassSample {
            doppler_millihz: MilliHz::new(-GlonassSample::DOPPLER_MAX_MILLIHZ.as_i32()),
            ..make_sample()
        };

        assert!(max.validate_doppler().is_ok());
        assert!(min.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_doppler_out_of_range() {
        let s1 = GlonassSample {
            doppler_millihz: MilliHz::new(6_000_000),
            ..make_sample()
        };

        let s2 = GlonassSample {
            doppler_millihz: MilliHz::new(-6_000_000),
            ..make_sample()
        };

        assert!(s1.validate_doppler().is_err());
        assert!(s2.validate_doppler().is_err());
    }

    #[test]
    fn test_carrier_freq_values() {
        let s0 = GlonassSample {
            slot: GloSlot::new(0).unwrap(),
            ..make_sample()
        };
        assert_eq!(s0.carrier_freq_millihz().unwrap(), 1_602_000_000);

        let s1 = GlonassSample {
            slot: GloSlot::new(1).unwrap(),
            ..make_sample()
        };
        assert_eq!(s1.carrier_freq_millihz().unwrap(), 1_602_562_500);

        let s_7 = GlonassSample {
            slot: GloSlot::new(-7).unwrap(),
            ..make_sample()
        };
        assert_eq!(s_7.carrier_freq_millihz().unwrap(), 1_598_062_500);
    }

    #[test]
    fn test_is_tracked() {
        let good = GlonassSample {
            cn0_dbhz: DbHz::new(42).unwrap(),
            ..make_sample()
        };

        let edge = GlonassSample {
            cn0_dbhz: DbHz::new(GlonassSample::CN0_MIN_TRACKED).unwrap(),
            ..make_sample()
        };

        let bad = GlonassSample {
            cn0_dbhz: DbHz::new(10).unwrap(),
            ..make_sample()
        };

        assert!(good.is_tracked());
        assert!(edge.is_tracked());
        assert!(!bad.is_tracked());
    }

    #[test]
    fn test_full_validate_ok() {
        assert!(make_sample().validate().is_ok());
    }

    #[test]
    fn test_mm_precision() {
        let a = 21_500_000_000i64;
        let b = 21_500_000_001i64;

        assert_eq!(b - a, 1);
    }

    #[test]
    fn test_millihz_precision() {
        let a = 1_200_500i32;
        let b = 1_200_501i32;

        assert_eq!(b - a, 1);
    }

    #[test]
    fn test_ranges_cover_domain() {
        assert!(i64::MAX > GlonassSample::PSEUDORANGE_MAX_MM.as_i64());
        assert!(i32::MAX > GlonassSample::DOPPLER_MAX_MILLIHZ.as_i32());
    }
}
