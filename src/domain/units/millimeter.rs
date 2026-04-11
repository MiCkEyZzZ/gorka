/// Distance expressed as an integer number of **millimetres**.
///
/// Using integer newtype instead of `f64` ensures lossless arithmetic
/// and avoids floating-point rounding errors for pseudoranges.
///
/// # Range
/// Inner value is `i64`, roughly ±9.2 × 10¹² mm (~ ±9.2 × 10⁹ m),
/// enough to cover all GNSS pseudoranges.
///
/// # Usage
/// Prefer `Millimeter` for all range/pseudorange fields. Convert to `f64` m for
/// display/debug.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Millimeter(pub i64);

impl Millimeter {
    /// Creates a new [`Millimeter`] from a raw `i64` millimetre value.
    ///
    /// # Example
    /// ```
    /// use gorka::Millimeter;
    ///
    /// // 21 500 km expressed in millimetres
    /// let range = Millimeter::new(21_500_000_000);
    /// assert_eq!(range.as_i64(), 21_500_000_000);
    /// ```
    pub const fn new(v: i64) -> Self {
        Self(v)
    }

    /// Returns the raw inner value in millimetres.
    pub const fn as_i64(self) -> i64 {
        self.0
    }

    /// Converts the value to meters as `f64`
    pub const fn as_m(self) -> f64 {
        self.0 as f64 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_millimeter_basic() {
        let m = Millimeter::new(42);

        assert_eq!(m.as_i64(), 42);

        let m_neg = Millimeter::new(-10);

        assert_eq!(m_neg.as_i64(), -10);
    }

    #[test]
    fn test_millimeter_to_meters() {
        let m = Millimeter::new(1_500);

        assert_eq!(m.as_m(), 1.5);
    }

    #[test]
    fn test_millimeter_ordering() {
        let a = Millimeter::new(100);
        let b = Millimeter::new(200);

        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_millimeter_copy() {
        let a = Millimeter::new(100);
        let b = a;

        assert_eq!(a.as_i64(), b.as_i64());
    }

    #[test]
    fn test_millimeter_negative_to_meters() {
        let m = Millimeter::new(-2000);

        assert_eq!(m.as_m(), -2.0);
    }

    #[test]
    fn test_millimeter_ordering_edge() {
        let a = Millimeter::new(-100);
        let b = Millimeter::new(0);
        let c = Millimeter::new(100);

        assert!(a < b);
        assert!(b < c);
        assert!(a < c);
    }
}
