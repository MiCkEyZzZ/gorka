use crate::{BdsPrn, DbHz, GnssMeasurement, GorkaError, Hertz, MilliHz, Millimeter, SatelliteId};

/// BDS B1I carrier frequency (Hz).
pub const BDS_B1I_FREQ: Hertz = Hertz(1_561_098_000);

/// BDS B1C carrier frequency (Hz) same as L1.
pub const BDS_B1C_FREQ: Hertz = Hertz(1_575_420_000);

/// BDS B2a carrier frequency (Hz) same as E5a.
pub const BDS_B2A_FREQ: Hertz = Hertz(1_176_450_000);

#[derive(Debug, Clone, PartialEq)]
pub struct BeidouSample {
    pub timestamp_ms: u64,
    pub prn: BdsPrn,
    pub cn0_dbhz: DbHz,
    pub pseudorange_mm: Millimeter,
    pub doppler_millihz: MilliHz,
    pub carrier_phase_cycles: Option<i64>,
}

impl BeidouSample {
    pub const PSEUDORANGE_MIN_MM: Millimeter = Millimeter::new(21_500_000_000);
    pub const PSEUDORANGE_MAX_MM: Millimeter = Millimeter::new(42_000_000_000);
    pub const DOPPLER_MAX_MILLIHZ: MilliHz = MilliHz::new(5_000_000);

    pub fn validate_prn(&self) -> Result<(), GorkaError> {
        let prn = self.prn.get();

        if !(BdsPrn::MIN..=BdsPrn::MAX).contains(&prn) {
            return Err(GorkaError::InvalidPrn(prn));
        }

        Ok(())
    }

    pub fn validate_pseudorange(&self) -> Result<(), GorkaError> {
        let mm = self.pseudorange_mm.as_i64();

        if !(Self::PSEUDORANGE_MIN_MM.0..=Self::PSEUDORANGE_MAX_MM.0).contains(&mm) {
            return Err(GorkaError::InvalidPseudorange(mm));
        }

        Ok(())
    }

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
        (BdsPrn::MIN..=BdsPrn::MAX).contains(&prn)
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

impl GnssMeasurement for BeidouSample {
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
        SatelliteId::Beidou(self.prn)
    }

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

    fn make_bds() -> BeidouSample {
        BeidouSample {
            timestamp_ms: 1_700_000_000_000,
            prn: BdsPrn::new(25).unwrap(),
            cn0_dbhz: DbHz(38),
            pseudorange_mm: Millimeter::new(24_000_000_000),
            doppler_millihz: MilliHz::new(-1_800_000),
            carrier_phase_cycles: Some(5_555_555),
        }
    }

    #[test]
    fn test_valid_beidou_sample() {
        let sample = make_bds();

        assert!(sample.is_valid());
        assert!(sample.validate().is_ok());
    }

    #[test]
    fn test_invalid_prn() {
        let prn_result = BdsPrn::new(100);

        assert!(prn_result.is_err());
        assert!(matches!(prn_result, Err(GorkaError::InvalidPrn(100))));

        // Для создания BdsSample используем валидный PRN
        let mut sample = make_bds();

        sample.prn = BdsPrn::new(1).unwrap(); // валидное значение

        assert!(sample.is_valid_prn());
        assert!(sample.validate().is_ok());
    }

    #[test]
    fn test_invalid_pseudorange() {
        let mut sample = make_bds();

        sample.pseudorange_mm = Millimeter::new(50_000_000_000); // слишком большой

        assert!(!sample.is_valid_pseudorange());
        assert!(sample.validate().is_err());
    }

    #[test]
    fn test_invalid_doppler() {
        let mut sample = make_bds();

        sample.doppler_millihz = MilliHz::new(6_000_000); // слишком большой

        assert!(!sample.is_valid_doppler());
        assert!(sample.validate().is_err());
    }

    #[test]
    fn test_satellite_id() {
        let sample = make_bds();
        let sat_id = sample.satellite_id();

        match sat_id {
            SatelliteId::Beidou(prn) => assert_eq!(prn, sample.prn),
            _ => panic!("Expected Beidou satellite"),
        }
    }
}
