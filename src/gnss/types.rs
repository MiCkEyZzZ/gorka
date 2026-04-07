use crate::GorkaError;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MilliHz(pub i32);

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Millimeter(pub i64);

/// Frequency expressed as an integer number of **hertz** (Hz).
///
/// This type is mainly for general-purpose frequency representation.
/// For Doppler offsets, use [`MilliHz`] instead.
///
/// # Range
/// Inner value is `i64`, large enough to cover all GNSS carrier frequencies.
///
/// # Usage
/// Can be used in calculations or conversions, display as `f64` Hz if needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hertz(pub i64);

/// GPS satellite PRN (pseudo-random number) identifier.
///
/// # Range
/// 1..=32 (standard GPS PRN numbers)
///
/// # Usage
/// Use [`GpsPrn::new`] to construct with validation. Retrieve inner value via
/// [`GpsPrn::get`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpsPrn(pub u8);

/// Galileo satellite SVN (space vehicle number).
///
/// # Range
/// 1..=36 (standard Galileo SVN numbers)
///
/// # Usage
/// Use [`GalSvn::new`] to construct with validation. Retrieve inner value via
/// [`GalSvn::get`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GalSvn(pub u8);

/// BDS satellite PRN (pseudo-random number) identifier.
///
/// # Range
/// 1..=63 (standard BDS PRN numbers)
///
/// # Usage
/// Use [`BdsPrn::new`] to construct with validation. Retrieve inner value via
/// [`BdsPrn::get`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BdsPrn(pub u8);

/// GLONASS satellite frequency slot.
///
/// # Range
/// -7..=6
///
/// # Usage
/// Use [`GloSlot::new`] to construct with validation. Retrieve inner value via
/// [`GloSlot::get`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GloSlot(pub i8);

/// Carrier-to-noise density expressed in **dB-Hz** (dB·Hz).
///
/// # Range
/// Typical GNSS signals: 0..=60 dB-Hz. Stored as a raw `u8`.
///
/// # Usage
/// Use this type for all C/N₀ fields in GNSS observations.
/// Convert to `f32` or `f64` only if needed for calculations or plotting.
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

    pub fn new(prn: u8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&prn) {
            Ok(Self(prn))
        } else {
            Err(GorkaError::InvalidPrn(prn))
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl GalSvn {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 36;

    pub fn new(svn: u8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&svn) {
            Ok(Self(svn))
        } else {
            Err(GorkaError::InvalidSvn(svn))
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl BdsPrn {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 63;

    pub fn new(prn: u8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&prn) {
            Ok(Self(prn))
        } else {
            Err(GorkaError::InvalidPrn(prn))
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millimeter_basic() {
        let m = Millimeter::new(42);
        assert_eq!(m.as_i64(), 42);

        let m_neg = Millimeter::new(-10);
        assert_eq!(m_neg.as_i64(), -10);
    }

    #[test]
    fn millihz_basic() {
        let hz = MilliHz::new(1_234_567);
        assert_eq!(hz.as_i32(), 1_234_567);
        assert_eq!(hz.abs().as_i32(), 1_234_567);

        let hz_neg = MilliHz::new(-500_000);
        assert_eq!(hz_neg.abs().as_i32(), 500_000);
    }

    #[test]
    fn gps_prn_valid() {
        for prn in GpsPrn::MIN..=GpsPrn::MAX {
            let gps = GpsPrn::new(prn).unwrap();
            assert_eq!(gps.get(), prn);
        }
    }

    #[test]
    fn gps_prn_invalid() {
        assert!(GpsPrn::new(0).is_err());
        assert!(GpsPrn::new(33).is_err());
    }

    #[test]
    fn gal_svn_valid() {
        for svn in GalSvn::MIN..=GalSvn::MAX {
            let gal = GalSvn::new(svn).unwrap();
            assert_eq!(gal.get(), svn);
        }
    }

    #[test]
    fn gal_svn_invalid() {
        assert!(GalSvn::new(0).is_err());
        assert!(GalSvn::new(37).is_err());
    }

    #[test]
    fn bds_prn_valid() {
        for prn in BdsPrn::MIN..=BdsPrn::MAX {
            let bds = BdsPrn::new(prn).unwrap();
            assert_eq!(bds.get(), prn);
        }
    }

    #[test]
    fn bds_prn_invalid() {
        assert!(BdsPrn::new(0).is_err());
        assert!(BdsPrn::new(64).is_err());
    }

    #[test]
    fn glo_slot_valid() {
        for slot in GloSlot::MIN..=GloSlot::MAX {
            let s = GloSlot::new(slot).unwrap();
            assert_eq!(s.get(), slot);
        }
    }

    #[test]
    fn glo_slot_invalid() {
        assert!(GloSlot::new(-8).is_err());
        assert!(GloSlot::new(7).is_err());
    }

    #[test]
    fn dbhz_basic() {
        let c = DbHz(42);
        assert_eq!(c.get(), 42);
    }
}
