//! Common measurement trait for all GNSS constellations.
//!
//! [`GnssMeasurement`] is the unified interface that GLONASS, Galileo, BeiDou
//! and GPS observations implement. Codec-specific code (encoder, decoder) can
//! be written generically this trait.

use crate::{DbHz, GorkaError, MilliHz, Millimeter, SatelliteId};

/// Common interface for a single GNSS satellite observation.
///
/// All constellation-specific sample types (`GlonassSample`, `GpsSample`,
/// `GalileoSample`, `BeidouSample`) implement this trait, allowing generic
/// algorithms to work across constellations.
///
/// # no_std
///
/// The trait has no heap-allocation requirements and is suitable for
/// `no_std` + `alloc` environments.
pub trait GnssMeasurement {
    /// Unix timestamp of this observation in milliseconds.
    fn timestamp_ms(&self) -> u64;
    /// Unique satellite identifier (constellation + numeric id).
    fn satellite_id(&self) -> SatelliteId;
    /// Carrier-to-noise density ratio in dB·Hz.
    fn cn0_dbhz(&self) -> DbHz;
    /// Pseudorange in millimetres.
    fn pseudorange_mm(&self) -> Millimeter;
    /// Doppler shift in millihertz.
    fn doppler_millihz(&self) -> MilliHz;
    /// Accumulated carrier phase in 2⁻³² cycles, if available.
    fn carrier_phase_cycles(&self) -> Option<i64>;
    /// Validates all fields. Returns the first error found.
    fn validate(&self) -> Result<(), GorkaError>;
    /// Returns `true` if the signal is considered tracked.
    ///
    /// Default implementation: `cn0_dbhz >= 20`.
    fn is_tracked(&self) -> bool {
        self.cn0_dbhz().is_tracked()
    }
}

pub const CNO_TRACK_THRESHOLD: u8 = 20;

const MAX_DOPPLER_MHZ: i32 = 10_000_000;

/// A generic, constellation-agnostic GNSS observation.
#[derive(Debug, Clone, PartialEq)]
pub struct GnssSample {
    /// Unix timestamp in milliseconds.
    timestamp_ms: u64,
    /// Satellite identifier (constellation + id).
    satellite_id: SatelliteId,
    /// Carrier-to-noise density in dB·Hz.
    cn0_dbhz: DbHz,
    /// Pseudorange in millimetres.
    pseudorange_mm: Millimeter,
    /// Doppler shift in millihertz.
    doppler_millihz: MilliHz,
    /// Accumulated carrier phase in 2⁻³² cycles, `None` if not tracked.
    carrier_phase_cycles: Option<i64>,
}

impl GnssSample {
    pub fn new(
        timestamp_ms: u64,
        satellite_id: SatelliteId,
        cn0_dbhz: DbHz,
        pseudorange_mm: Millimeter,
        doppler_millihz: MilliHz,
        carrier_phase_cycles: Option<i64>,
    ) -> Result<Self, GorkaError> {
        let s = Self {
            timestamp_ms,
            satellite_id,
            cn0_dbhz,
            pseudorange_mm,
            doppler_millihz,
            carrier_phase_cycles,
        };

        s.validate()?;

        Ok(s)
    }
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
        if self.pseudorange_mm.as_i64() <= 0 {
            return Err(GorkaError::InvalidPseudorange(self.pseudorange_mm.as_i64()));
        }

        if self.doppler_millihz.abs().as_i32() > MAX_DOPPLER_MHZ {
            return Err(GorkaError::InvalidDoppler(self.doppler_millihz.as_i32()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConstellationType, GpsPrn};

    fn make_sample() -> GnssSample {
        GnssSample::new(
            1_700_000_000_000,
            SatelliteId::gps(GpsPrn::new(7).unwrap()),
            DbHz::new(42).unwrap(),
            Millimeter::new(22_000_000_000),
            MilliHz::new(1_500_000),
            Some(12_345_678),
        )
        .unwrap()
    }

    fn make_sample_with(
        cn0: u8,
        pseudorange: i64,
        doppler: i32,
    ) -> Result<GnssSample, GorkaError> {
        GnssSample::new(
            1_700_000_000_000,
            SatelliteId::gps(GpsPrn::new(7).unwrap()),
            DbHz::new(cn0).unwrap(),
            Millimeter::new(pseudorange),
            MilliHz::new(doppler),
            Some(12_345_678),
        )
    }

    #[test]
    fn test_gnss_measurement_trait_accessors() {
        let s = make_sample();

        assert_eq!(s.timestamp_ms(), 1_700_000_000_000);
        assert_eq!(s.satellite_id().constellation(), ConstellationType::Gps);
        assert_eq!(s.cn0_dbhz(), DbHz::new(42).unwrap());
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
        let s = make_sample_with(15, 22_000_000_000, 1_500_000).unwrap();

        assert!(!s.is_tracked());
    }

    #[test]
    fn test_validate_ok() {
        assert!(make_sample().validate().is_ok());
    }

    #[test]
    fn test_validate_negative_pseudorange() {
        let res = make_sample_with(42, -1, 1_500_000);

        assert!(matches!(res, Err(GorkaError::InvalidPseudorange(_))));
    }

    #[test]
    fn test_validate_excessive_doppler() {
        let res = make_sample_with(42, 22_000_000_000, 15_000_000);

        assert!(matches!(res, Err(GorkaError::InvalidDoppler(_))));
    }

    #[test]
    fn test_generic_fn_over_trait() {
        fn print_cn0(obs: &impl GnssMeasurement) -> u8 {
            obs.cn0_dbhz().get()
        }

        let s = make_sample();

        assert_eq!(print_cn0(&s), 42);
    }
}
