use crate::{DbHz, GorkaError, MilliHz, Millimeter, SatelliteId};

/// Common interface for a single GNSS satellite observation.
pub trait GnssMeasurement {
    fn timestamp_ms(&self) -> u64;

    fn satellite_id(&self) -> SatelliteId;

    fn cn0_dbhz(&self) -> DbHz;

    fn pseudorange_mm(&self) -> Millimeter;

    fn doppler_millihz(&self) -> MilliHz;

    fn carrier_phase_cycles(&self) -> Option<i64>;

    fn validate(&self) -> Result<(), GorkaError>;

    fn is_tracked(&self) -> bool {
        self.cn0_dbhz().0 >= 20
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GnssSample {
    pub timestamp_ms: u64,
    pub satellite_id: SatelliteId,
    pub cn0_dbhz: DbHz,
    pub pseudorange_mm: Millimeter,
    pub doppler_millihz: MilliHz,
    pub carrier_phase_cycles: Option<i64>,
}

impl GnssMeasurement for GnssSample {
    fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }

    fn satellite_id(&self) -> SatelliteId {
        self.satellite_id
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

    fn validate(&self) -> Result<(), GorkaError> {
        // Минимальная проверка: pseudorange должна быть положительной, доплеровское
        // смещение — в пределах ±10 МГц.
        if self.pseudorange_mm.0 <= 0 {
            return Err(GorkaError::InvalidPseudorange(self.pseudorange_mm.0));
        }

        if self.doppler_millihz.abs().0 > 10_000_000 {
            return Err(GorkaError::InvalidDoppler(self.doppler_millihz.0));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConstellationType, GpsPrn};

    fn make_sample() -> GnssSample {
        GnssSample {
            timestamp_ms: 1_700_000_000_000,
            satellite_id: SatelliteId::gps(GpsPrn(7)),
            cn0_dbhz: DbHz(42),
            pseudorange_mm: Millimeter::new(22_000_000_000),
            doppler_millihz: MilliHz::new(1_500_000),
            carrier_phase_cycles: Some(12_345_678),
        }
    }

    #[test]
    fn test_gnss_measurement_trait_accessors() {
        let s = make_sample();

        assert_eq!(s.timestamp_ms(), 1_700_000_000_000);
        assert_eq!(s.satellite_id().constellation(), ConstellationType::Gps);
        assert_eq!(s.cn0_dbhz(), DbHz(42));
        assert_eq!(s.pseudorange_mm().0, 22_000_000_000);
        assert_eq!(s.doppler_millihz().0, 1_500_000);
        assert_eq!(s.carrier_phase_cycles(), Some(12_345_678));
    }

    #[test]
    fn test_is_tracked_above_threshold() {
        let s = make_sample();

        assert!(s.is_tracked());
    }

    #[test]
    fn test_is_tracked_below_threshold() {
        let s = GnssSample {
            cn0_dbhz: DbHz(15),
            ..make_sample()
        };
        assert!(!s.is_tracked());
    }

    #[test]
    fn test_validate_ok() {
        assert!(make_sample().validate().is_ok());
    }

    #[test]
    fn test_validate_negative_pseudorange() {
        let s = GnssSample {
            pseudorange_mm: Millimeter::new(-1),
            ..make_sample()
        };
        assert!(matches!(
            s.validate(),
            Err(GorkaError::InvalidPseudorange(_))
        ));
    }

    #[test]
    fn test_validate_excessive_doppler() {
        let s = GnssSample {
            doppler_millihz: MilliHz::new(15_000_000),
            ..make_sample()
        };
        assert!(matches!(s.validate(), Err(GorkaError::InvalidDoppler(_))));
    }

    #[test]
    fn test_generic_fn_over_trait() {
        fn print_cn0(obs: &impl GnssMeasurement) -> u8 {
            obs.cn0_dbhz().0
        }
        let s = make_sample();
        assert_eq!(print_cn0(&s), 42);
    }
}
