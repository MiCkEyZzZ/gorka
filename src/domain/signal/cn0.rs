use crate::GorkaError;

/// Carrier-to-noise density expressed in **dB-Hz** (dB·Hz).
///
/// # Range
/// Typical GNSS signals: 0..=60 dB-Hz. Stored as a raw `u8`.
///
/// # Usage
/// Use this type for all C/N₀ fields in GNSS observations.
/// Convert to `f32` or `f64` only if needed for calculations or plotting.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DbHz(u8);

impl DbHz {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 60;

    /// Threshold for signal tracking (~20 db-Hz).
    pub const TRACKED_THRESHOLD: u8 = 20;

    /// Threshold for strong signal (~40 db-Hz).
    pub const STRONG_THRESHOLD: u8 = 40;

    pub fn new(value: u8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(GorkaError::InvalidDbHz(value))
        }
    }

    #[inline(always)]
    pub const fn is_tracked(self) -> bool {
        self.0 >= Self::TRACKED_THRESHOLD
    }

    #[inline(always)]
    pub const fn is_strong(self) -> bool {
        self.0 >= Self::STRONG_THRESHOLD
    }

    /// Returns the raw C/N₀ value in dB-Hz.
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dbhz_basic() {
        let c = DbHz::new(42).unwrap();

        assert_eq!(c.get(), 42);
    }

    #[test]
    fn test_dbhz_range() {
        let low = DbHz::new(0).unwrap();
        let high = DbHz::new(60).unwrap();

        assert_eq!(low.get(), 0);
        assert_eq!(high.get(), 60);
    }

    #[test]
    fn test_dbhz_tracking_and_strength() {
        let weak = DbHz::new(10).unwrap();
        let tracked = DbHz::new(25).unwrap();
        let strong = DbHz::new(45).unwrap();

        assert!(!weak.is_tracked());
        assert!(tracked.is_tracked());
        assert!(strong.is_tracked());

        assert!(!weak.is_strong());
        assert!(!tracked.is_strong());
        assert!(strong.is_strong());
    }
}
