/// Frequency expressed as an integer number of **millihertz** (mHz).
///
/// 1 Hz = 1 000 mHz, so a Doppler shift of 1 200.5 Hz is stored as
/// `MilliHz(1_200_500)`. Using millihertz preserves sub-Hz precision
/// without floating-point noise.
///
/// # Range
/// Inner value is `i32`, roughly ±2.1 × 10⁶ Hz, well above max GLONASS Doppler
/// ±5 000 Hz.
///
/// # Usage
/// Use `MilliHz` for Doppler / carrier offsets. Convert to `f64` Hz only for
/// display/debug.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MilliHz(pub i32);

impl MilliHz {
    /// Creates a new [`MilliHz`] from a raw `i32` millihertz value.
    ///
    /// # Example
    /// ```
    /// use gorka::MilliHz;
    ///
    /// // 1 200.5 Hz expressed in millihertz
    /// let doppler = MilliHz::new(1_200_500);
    /// assert_eq!(doppler.as_i32(), 1_200_500);
    /// ```
    pub const fn new(v: i32) -> Self {
        Self(v)
    }

    /// Returns the raw inner value in millihertz.
    pub const fn as_i32(self) -> i32 {
        self.0
    }

    /// Returns the absolute value as a new [`MilliHz`].
    ///
    /// Useful for magnitude comparisons that are sign-agnostic, such as
    /// checking whether a Doppler shift exceeds a threshold.
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Converts the value to hertz as `f64`.
    pub const fn as_hz(self) -> f64 {
        self.0 as f64 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_millihz_basic() {
        let hz = MilliHz::new(1_234_567);

        assert_eq!(hz.as_i32(), 1_234_567);
        assert_eq!(hz.abs().as_i32(), 1_234_567);

        let hz_neg = MilliHz::new(-500_000);

        assert_eq!(hz_neg.abs().as_i32(), 500_000);
    }

    #[test]
    fn test_millihz_to_hz() {
        let hz = MilliHz::new(1_500_000);

        assert_eq!(hz.as_hz(), 1500.0);
    }

    #[test]
    fn test_millihz_ordering() {
        let a = MilliHz::new(-1000);
        let b = MilliHz::new(500);

        assert!(a < b);
    }

    #[test]
    fn test_millihz_abs_and_as_hz() {
        let hz = MilliHz::new(-1_500);

        assert_eq!(hz.abs().as_i32(), 1_500);
        assert_eq!(hz.as_hz(), -1.5);

        let hz2 = MilliHz::new(2_500);

        assert_eq!(hz2.as_hz(), 2.5);
    }

    #[test]
    fn test_millihz_ordering_edge() {
        let a = MilliHz::new(-1_000_000);
        let b = MilliHz::new(0);
        let c = MilliHz::new(1_000_000);

        assert!(a < b);
        assert!(b < c);
        assert!(a < c);
    }
}
