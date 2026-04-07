use crate::GorkaError;

/// Distance expressed as an integer number of **millimetres**.
///
/// Using an integer newtype instead of `f64` ensures lossless arithmetic
/// throughout the positioning pipeline and avoids floating-point rounding
/// errors when comparing or accumulating pseudoranges.
///
/// # Range
/// The inner value is `i64`, so the representable range is roughly
/// ±9.2 × 10¹² mm (≈ ±9.2 × 10⁹ m), which comfortably covers all GNSS
/// pseudoranges (GLONASS orbit altitude ≈ 19 100 km).
///
/// # Usage
/// Prefer `Millimeter` for all range / pseudorange fields inside gorka.
/// Convert to `f64` metres **only** for display or debug output via
/// [`crate::gnss::glonass::GlonassSample::pseudorange_m_approx`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MilliHz(pub i32);

/// Frequency expressed as an integer number of **millihertz** (mHz).
///
/// 1 Hz = 1 000 mHz, so a Doppler shift of 1 200.5 Hz is stored as
/// `MilliHz(1_200_500)`.  Using millihertz preserves sub-Hz precision
/// without floating-point noise.
///
/// # Range
/// The inner value is `i32`, covering roughly ±2.1 × 10⁶ Hz (≈ ±2.1 MHz),
/// which is well above the maximum expected GLONASS Doppler of ±5 000 Hz.
///
/// # Usage
/// Use `MilliHz` for Doppler and carrier-frequency offsets.  Convert to
/// `f64` Hz **only** for display or debug output via
/// [`crate::gnss::glonass::GlonassSample::doppler_hz_approx`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Millimeter(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hertz(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpsPrn(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GalSvn(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BdsPrn(pub u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GloSlot(pub i8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DbHz(pub u8);

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
    pub fn new(v: i64) -> Self {
        Self(v)
    }

    /// Returns the raw inner value in millimetres.
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

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
    pub fn new(v: i32) -> Self {
        Self(v)
    }

    /// Returns the raw inner value in millihertz.
    pub fn as_i32(&self) -> i32 {
        self.0
    }

    /// Returns the absolute value as a new [`MilliHz`].
    ///
    /// Useful for magnitude comparisons that are sign-agnostic, such as
    /// checking whether a Doppler shift exceeds a threshold.
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
}

impl GpsPrn {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 32;

    pub fn new(value: u8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(GorkaError::InvalidPrn(value))
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl GalSvn {
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl BdsPrn {
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl GloSlot {
    pub const MIN: i8 = -7;
    pub const MAX: i8 = 6;

    pub fn new(slot: i8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&slot) {
            Ok(Self(slot))
        } else {
            Err(GorkaError::InvalidSlot(slot))
        }
    }

    #[inline]
    pub fn get(self) -> i8 {
        self.0
    }
}

impl DbHz {
    pub const fn get(self) -> u8 {
        self.0
    }
}
