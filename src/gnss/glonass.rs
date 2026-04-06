use crate::{GorkaError, MilliHz, Millimeter};

// Общие типы вынес в отдельный файл `types.rs` для удобства в будущем
// при создании других источников таких как: GPS, Galileo, Beido

/// A single GLONASS satellite observation captured at one epoch.
///
/// GLONASS uses **FDMA** (Frequency Division Multiple Access): each satellite
/// transmits on a unique carrier frequency determined by its frequency slot `k`
/// (see [`GlonassSample::slot`]).  A `GlonassSample` bundles the raw
/// measurements needed for pseudorange positioning and Doppler velocity
/// estimation.
///
/// # Fixed-point representation
/// All measurements use integer newtypes ([`Millimeter`], [`MilliHz`]) to
/// avoid floating-point rounding errors in the processing pipeline.
///
/// # Validation
/// Call [`validate`](GlonassSample::validate) before using a sample in any
/// computation.  Individual checks are also available as
/// [`validate_slot`](GlonassSample::validate_slot),
/// [`validate_pseudorange`](GlonassSample::validate_pseudorange), and
/// [`validate_doppler`](GlonassSample::validate_doppler).
// `slot` is the FDMA frequency slot k ∈ [-7, +6].
// Carrier frequency = 1602 + k * 0.5625 Mhz.
#[derive(Debug, Clone, PartialEq)]
pub struct GlonassSample {
    /// Unix timestamp in milliseconds since epoch.
    pub timestamp_ms: u64,
    /// GLONASS FDMA frequency slot: k ∈ \[−7, +6\] (14 slots total).
    ///
    /// The L1 carrier frequency for slot `k` is
    /// `1 602 + k × 0.5625 MHz`.
    /// Use [`carrier_freq_millihz`](GlonassSample::carrier_freq_millihz)
    /// to compute the exact value.
    pub slot: i8,
    /// Carrier-to-noise density ratio in dB·Hz.
    ///
    /// Typical tracked values: 30–50 dB·Hz.
    /// Signals below [`CN0_MIN_TRACKED`](GlonassSample::CN0_MIN_TRACKED)
    /// (20 dB·Hz) are considered untracked.
    pub cn0_dbhz: u8,
    /// Measured pseudorange in millimetres.
    ///
    /// Physically plausible range:
    /// [`PSEUDORANGE_MIN_MM`]…[`PSEUDORANGE_MAX_MM`] (roughly 19 100 km –
    /// 25 600 km).
    ///
    /// [`PSEUDORANGE_MIN_MM`]: GlonassSample::PSEUDORANGE_MIN_MM
    /// [`PSEUDORANGE_MAX_MM`]: GlonassSample::PSEUDORANGE_MAX_MM
    pub pseudorange_mm: Millimeter,
    /// Measured Doppler shift in millihertz.
    ///
    /// Positive values indicate the satellite is approaching; negative values
    /// indicate it is receding.  Magnitude is bounded by
    /// [`DOPPLER_MAX_MILLIHZ`](GlonassSample::DOPPLER_MAX_MILLIHZ) (≈ ±5 000
    /// Hz).
    pub doppler_millihz: MilliHz,
    /// Accumulated carrier phase in units of 2⁻³² cycles.
    ///
    /// `None` when the receiver has not yet achieved phase lock or after a
    /// cycle slip.  When present, the value can grow large over long
    /// observation sessions — `i64` provides sufficient range.
    pub carrier_phase_cycles: Option<i64>,
}

impl GlonassSample {
    // GLONASS slot range: k ∈ [-7, +6] (14 slots total)
    /// Minimum valid FDMA frequency slot (`-7`).
    pub const SLOT_MIN: i8 = -7;

    /// Maximum valid FDMA frequency slot (`+6`).
    pub const SLOT_MAX: i8 = 6;

    // Carrier frequency for slot `k` in Hz (integer mHz for precision)
    /// L1 centre frequency for slot `k = 0` in millihertz (1 602.000 000 MHz).
    pub const BASE_FREQ_MILLIHZ: i64 = 1_602_000_000;

    /// Per-slot frequency step in millihertz (0.562 5 MHz).
    pub const FREQ_STEP_MILLIHZ: i64 = 562_500;

    /// Minimum plausible pseudorange for a GLONASS satellite (LEO, ~19 100 km).
    pub const PSEUDORANGE_MIN_MM: Millimeter = Millimeter(19_100_000_000);

    /// Maximum plausible pseudorange for a GLONASS satellite (~25 600 km at
    /// horizon).
    pub const PSEUDORANGE_MAX_MM: Millimeter = Millimeter(25_600_000_000);

    /// Maximum plausible Doppler for GLONASS (orbital speed ≈ 3.9 km/s → ~±5000
    /// Hz).
    pub const DOPPLER_MAX_MILLIHZ: MilliHz = MilliHz(5_000_000);

    /// Minimum signal strength considered "tracked".
    pub const CN0_MIN_TRACKED: u8 = 20;

    /// Validates that all fields are within physically plausible bounds.
    ///
    /// Runs [`validate_slot`], [`validate_pseudorange`], and
    /// [`validate_doppler`] in sequence, returning the first error encountered.
    ///
    /// [`validate_slot`]: GlonassSample::validate_slot
    /// [`validate_pseudorange`]: GlonassSample::validate_pseudorange
    /// [`validate_doppler`]: GlonassSample::validate_doppler
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidSlot`] if the slot is out of range,
    /// [`GorkaError::InvalidPseudorange`] if the pseudorange is implausible, or
    /// [`GorkaError::InvalidDoppler`] if the Doppler magnitude is too large.
    pub fn validate(&self) -> Result<(), GorkaError> {
        self.validate_slot()?;
        self.validate_pseudorange()?;
        self.validate_doppler()?;

        Ok(())
    }

    /// Validates only the FDMA slot identifier.
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidSlot`] when `slot` is outside
    /// \[`SLOT_MIN`, `SLOT_MAX`\] (i.e., outside \[−7, +6\]).
    ///
    /// # Example
    /// ```
    /// use gorka::{GlonassSample, MilliHz, Millimeter};
    ///
    /// let s = GlonassSample {
    ///     slot: 7,
    ///     ..GlonassSample::default_zeroed()
    /// };
    /// assert!(s.validate_slot().is_err());
    /// ```
    pub fn validate_slot(&self) -> Result<(), GorkaError> {
        if !(Self::SLOT_MIN..=Self::SLOT_MAX).contains(&self.slot) {
            return Err(GorkaError::InvalidSlot(self.slot));
        }

        Ok(())
    }

    /// Validates that the pseudorange is within physically plausible bounds.
    ///
    /// Checks that [`pseudorange_mm`](GlonassSample::pseudorange_mm) falls in
    /// \[[`PSEUDORANGE_MIN_MM`], [`PSEUDORANGE_MAX_MM`]\].
    ///
    /// [`PSEUDORANGE_MIN_MM`]: GlonassSample::PSEUDORANGE_MIN_MM
    /// [`PSEUDORANGE_MAX_MM`]: GlonassSample::PSEUDORANGE_MAX_MM
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidPseudorange`] with the offending raw value
    /// when the range is outside the expected window.
    pub fn validate_pseudorange(&self) -> Result<(), GorkaError> {
        if self.pseudorange_mm < Self::PSEUDORANGE_MIN_MM
            || self.pseudorange_mm > Self::PSEUDORANGE_MAX_MM
        {
            return Err(GorkaError::InvalidPseudorange(self.pseudorange_mm.0));
        }

        Ok(())
    }

    /// Validates that the Doppler magnitude is within plausible bounds.
    ///
    /// The absolute value of
    /// [`doppler_millihz`](GlonassSample::doppler_millihz) must not exceed
    /// [`DOPPLER_MAX_MILLIHZ`](GlonassSample::DOPPLER_MAX_MILLIHZ)
    /// (5 000 000 mHz ≈ 5 000 Hz).
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidDoppler`] with the offending raw value
    /// when the magnitude is too large.
    pub fn validate_doppler(&self) -> Result<(), GorkaError> {
        if self.doppler_millihz.abs() > Self::DOPPLER_MAX_MILLIHZ {
            return Err(GorkaError::InvalidDoppler(self.doppler_millihz.0));
        }

        Ok(())
    }

    /// Returns `true` if the signal strength is above the tracking threshold.
    ///
    /// A sample is considered *tracked* when
    /// `cn0_dbhz >= `[`CN0_MIN_TRACKED`](GlonassSample::CN0_MIN_TRACKED)` (20
    /// dB·Hz)`. Untracked samples should generally be excluded from
    /// positioning computations.
    #[inline]
    pub fn is_tracked(&self) -> bool {
        self.cn0_dbhz >= Self::CN0_MIN_TRACKED
    }

    /// Computes the L1 carrier frequency for this sample's slot in millihertz.
    ///
    /// Formula: `BASE_FREQ_MILLIHZ + slot × FREQ_STEP_MILLIHZ`
    /// (i.e., `1 602 + k × 0.5625 MHz`).
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidSlot`] if the slot fails
    /// [`validate_slot`](GlonassSample::validate_slot).
    ///
    /// # Example
    /// ```
    /// use gorka::GlonassSample;
    ///
    /// let mut s = GlonassSample::default_zeroed();
    /// s.slot = 1; // k = +1 → 1 602.5625 MHz
    /// assert_eq!(s.carrier_freq_millihz().unwrap(), 1_602_562_500);
    /// ```
    pub fn carrier_freq_millihz(&self) -> Result<i64, GorkaError> {
        self.validate_slot()?;

        Ok(Self::BASE_FREQ_MILLIHZ + self.slot as i64 * Self::FREQ_STEP_MILLIHZ)
    }

    /// Pseudorange in metres as a human-readable `f64` (for display / debug
    /// only).
    // Don't use this value for computation inside gorka - use pseudorange_mm directly to avoid
    // floating-point noise.
    #[cfg(feature = "std")]
    pub fn pseudorange_m_approx(&self) -> f64 {
        self.pseudorange_mm.0 as f64 / 1_000.0
    }

    /// Doppler in Hz as a human-readable `f64` (for display / debug only).
    ///
    /// Do **not** use this value for any computation — use
    /// [`doppler_millihz`](GlonassSample::doppler_millihz) directly.
    #[cfg(feature = "std")]
    pub fn doppler_hz_approx(&self) -> f64 {
        self.doppler_millihz.0 as f64 / 1_000.0
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
            slot: 0,
            cn0_dbhz: 0,
            pseudorange_mm: Millimeter(0),
            doppler_millihz: MilliHz(0),
            carrier_phase_cycles: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample() -> GlonassSample {
        GlonassSample {
            timestamp_ms: 1_700_000_000_000,
            slot: 1,
            cn0_dbhz: 42,
            pseudorange_mm: Millimeter::new(21_500_000_000),
            doppler_millihz: MilliHz::new(1_200_500),
            carrier_phase_cycles: Some(0x0001_2345_6789_ABCDi64),
        }
    }

    #[test]
    fn test_validate_slot_valid_boundaries() {
        for slot in GlonassSample::SLOT_MIN..=GlonassSample::SLOT_MAX {
            let s = GlonassSample {
                slot,
                ..make_sample()
            };

            assert!(s.validate_slot().is_ok(), "slot {slot} should be valid");
        }
    }

    #[test]
    fn test_validate_slot_below_min() {
        let s = GlonassSample {
            slot: -8,
            ..make_sample()
        };

        assert!(matches!(
            s.validate_slot(),
            Err(GorkaError::InvalidSlot(-8))
        ));
    }

    #[test]
    fn test_validate_slot_above_max() {
        let s = GlonassSample {
            slot: 7,
            ..make_sample()
        };

        assert!(matches!(s.validate_slot(), Err(GorkaError::InvalidSlot(7))));
    }

    #[test]
    fn test_validate_slot_extremes() {
        let min = GlonassSample {
            slot: GlonassSample::SLOT_MIN,
            ..make_sample()
        };
        let max = GlonassSample {
            slot: GlonassSample::SLOT_MAX,
            ..make_sample()
        };

        assert!(min.validate_slot().is_ok());
        assert!(max.validate_slot().is_ok());
    }

    #[test]
    fn test_validate_pseudorange_typical() {
        let s = make_sample();

        assert!(s.validate_pseudorange().is_ok());
    }

    #[test]
    fn test_validate_pseudorange_at_min_boundary() {
        let s = GlonassSample {
            pseudorange_mm: GlonassSample::PSEUDORANGE_MIN_MM,
            ..make_sample()
        };

        assert!(s.validate_pseudorange().is_ok());
    }

    #[test]
    fn test_validate_pseudorange_at_max_boundary() {
        let s = GlonassSample {
            pseudorange_mm: GlonassSample::PSEUDORANGE_MAX_MM,
            ..make_sample()
        };

        assert!(s.validate_pseudorange().is_ok());
    }

    #[test]
    fn test_validate_pseudorange_too_small() {
        let s = GlonassSample {
            pseudorange_mm: Millimeter::new(1_000_000),
            ..make_sample()
        };

        assert!(matches!(
            s.validate_pseudorange(),
            Err(GorkaError::InvalidPseudorange(_))
        ));
    }

    #[test]
    fn test_validate_pseudorange_too_large() {
        let s = GlonassSample {
            pseudorange_mm: Millimeter::new(99_000_000_000),
            ..make_sample()
        };

        assert!(matches!(
            s.validate_pseudorange(),
            Err(GorkaError::InvalidPseudorange(_))
        ));
    }

    #[test]
    fn test_validate_pseudorange_negative() {
        let s = GlonassSample {
            pseudorange_mm: Millimeter::new(-1),
            ..make_sample()
        };

        assert!(matches!(
            s.validate_pseudorange(),
            Err(GorkaError::InvalidPseudorange(_))
        ));
    }

    #[test]
    fn test_validate_doppler_typical_positive() {
        let s = GlonassSample {
            doppler_millihz: MilliHz::new(1_200_500),
            ..make_sample()
        };

        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_validate_doppler_typical_negative() {
        let s = GlonassSample {
            doppler_millihz: MilliHz::new(-3_500_000),
            ..make_sample()
        };

        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_test_validate_doppler_at_max_boundary() {
        let s = GlonassSample {
            doppler_millihz: GlonassSample::DOPPLER_MAX_MILLIHZ,
            ..make_sample()
        };

        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_test_validate_doppler_at_min_boundary() {
        let s = GlonassSample {
            doppler_millihz: MilliHz::new(-GlonassSample::DOPPLER_MAX_MILLIHZ.0),
            ..make_sample()
        };

        assert!(s.validate_doppler().is_ok());
    }

    #[test]
    fn test_test_validate_doppler_exceeds_max() {
        let s = GlonassSample {
            doppler_millihz: MilliHz::new(6_000_000),
            ..make_sample()
        };

        assert!(matches!(
            s.validate_doppler(),
            Err(GorkaError::InvalidDoppler(_))
        ));
    }

    #[test]
    fn test_test_validate_doppler_exceeds_min() {
        let s = GlonassSample {
            doppler_millihz: MilliHz::new(-6_000_000),
            ..make_sample()
        };

        assert!(matches!(
            s.validate_doppler(),
            Err(GorkaError::InvalidDoppler(_))
        ));
    }

    #[test]
    fn test_test_carrier_phase_some_and_none() {
        let with = GlonassSample {
            carrier_phase_cycles: Some(123_456_789),
            ..make_sample()
        };

        let without = GlonassSample {
            carrier_phase_cycles: None,
            ..make_sample()
        };

        assert_eq!(with.carrier_phase_cycles, Some(123_456_789));
        assert_eq!(without.carrier_phase_cycles, None);
    }

    #[test]
    fn test_carrier_phase_large_accumulation() {
        let s = GlonassSample {
            carrier_phase_cycles: Some(450_000_000_i64 * (1 << 16)),
            ..make_sample()
        };
        assert!(s.carrier_phase_cycles.is_some());
    }

    #[test]
    fn test_carrier_freq_slot_zero() {
        // k=0 -> 1602.000 000 MHz = 1_602_000_000 mHz
        let s = GlonassSample {
            slot: 0,
            ..make_sample()
        };
        assert_eq!(s.carrier_freq_millihz().unwrap(), 1_602_000_000);
    }

    #[test]
    fn test_carrier_freq_slot_plus_one() {
        // k=+1 -> 1602.5625 MHz = 1_602_562_500 mHz
        let s = GlonassSample {
            slot: 1,
            ..make_sample()
        };
        assert_eq!(s.carrier_freq_millihz().unwrap(), 1_602_562_500);
    }

    #[test]
    fn test_carrier_freq_slot_minus_seven() {
        // k=-7 -> 1602 - 7×0.5625 = 1598.0625 MHz = 1_598_062_500 mHz
        let s = GlonassSample {
            slot: -7,
            ..make_sample()
        };
        assert_eq!(s.carrier_freq_millihz().unwrap(), 1_598_062_500);
    }

    #[test]
    fn test_carrier_freq_invalid_slot_returns_error() {
        let s = GlonassSample {
            slot: 99,
            ..make_sample()
        };
        assert!(matches!(
            s.carrier_freq_millihz(),
            Err(GorkaError::InvalidSlot(99))
        ));
    }

    #[test]
    fn test_is_tracked_above_threshold() {
        let s = GlonassSample {
            cn0_dbhz: 42,
            ..make_sample()
        };
        assert!(s.is_tracked());
    }

    #[test]
    fn test_is_tracked_at_threshold() {
        let s = GlonassSample {
            cn0_dbhz: GlonassSample::CN0_MIN_TRACKED,
            ..make_sample()
        };
        assert!(s.is_tracked());
    }

    #[test]
    fn test_is_not_tracked_below_threshold() {
        let s = GlonassSample {
            cn0_dbhz: 10,
            ..make_sample()
        };
        assert!(!s.is_tracked());
    }

    #[test]
    fn test_s_not_tracked_zero() {
        let s = GlonassSample {
            cn0_dbhz: 0,
            ..make_sample()
        };
        assert!(!s.is_tracked());
    }

    #[test]
    fn test_full_validate_ok() {
        assert!(make_sample().validate().is_ok());
    }

    #[test]
    fn test_full_validate_bad_slot_fails() {
        let s = GlonassSample {
            slot: -8,
            ..make_sample()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_pseudorange_scale_1mm_precision() {
        // Two ranges differing by exactly 1 mm must be distinguishable
        let a: i64 = 21_500_000_000;
        let b: i64 = 21_500_000_001;

        assert_ne!(a, b);
        assert_eq!(b - a, 1); // 1 mm delta stored losslessly
    }

    #[test]
    fn test_doppler_scale_1mhz_precision() {
        // 1200.500 Hz stored as 1_200_500 mHz
        let raw: i32 = 1_200_500;

        assert_eq!(raw, 1_200_500);
        // 1200.501 Hz stored as 1_200_501 mHz — 1 mHz difference preserved
        assert_eq!(raw + 1, 1_200_501);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_i64_range_covers_all_gnss_pseudoranges() {
        // i64::MAX = 9.22 × 10^12 mm ≈ 9.22 × 10^9 m = 9.22 × 10^6 km
        // GLONASS orbit ≈ 19 100 km -> pseudorange fits with room to spare
        assert!(i64::MAX > GlonassSample::PSEUDORANGE_MAX_MM.0);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_i32_range_covers_all_gnss_doppler() {
        // i32::MAX = 2_147_483_647 mHz ≈ 2.15 MHz — way above ±5000 Hz
        assert!(i32::MAX > GlonassSample::DOPPLER_MAX_MILLIHZ.0);
    }

    #[test]
    fn test_default_zeroed_creates_valid_placeholder() {
        let s = GlonassSample::default_zeroed();

        assert_eq!(s.timestamp_ms, 0);
        assert_eq!(s.slot, 0);
        assert_eq!(s.cn0_dbhz, 0);
        assert_eq!(s.pseudorange_mm.0, 0);
        assert_eq!(s.doppler_millihz.0, 0);
        assert_eq!(s.carrier_phase_cycles, None);
    }

    #[test]
    fn test_stack_array_of_default_zeroed() {
        // Проверяем что можно создать [GlonassSample; N] на стеке
        let arr: [GlonassSample; 128] = core::array::from_fn(|_| GlonassSample::default_zeroed());

        assert_eq!(arr.len(), 128);
    }
}
