/// Frequency expressed as an integer number of **hertz** (Hz).
///
/// This type is mainly for general-purpose frequency representation.
/// For Doppler offsets, use [`crate::domain::units::MilliHz`] instead.
///
/// # Range
/// Inner value is `i64`, large enough to cover all GNSS carrier frequencies.
///
/// # Usage
/// Can be used in calculations or conversions, display as `f64` Hz if needed.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hertz(pub i64);

impl Hertz {
    /// Creates a new [`Hertz`] from a raw `i64` value.
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// Returns the raw inner value in hertz.
    pub const fn as_i64(self) -> i64 {
        self.0
    }

    /// Converts the value to `f64`.
    pub const fn as_f64(self) -> f64 {
        self.0 as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hertz_methods() {
        let h = Hertz::new(1_000_000);

        assert_eq!(h.as_i64(), 1_000_000);
        assert_eq!(h.as_f64(), 1_000_000.0);
    }
}
