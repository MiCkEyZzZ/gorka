use crate::GorkaError;

/// GPS satellite PRN (pseudo-random number) identifier.
///
/// # Range
/// 1..=32 (standard GPS PRN numbers)
///
/// # Usage
/// Use [`GpsPrn::new`] to construct with validation. Retrieve inner value via
/// [`GpsPrn::get`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpsPrn(u8);

/// Galileo satellite SVN (space vehicle number).
///
/// # Range
/// 1..=36 (standard Galileo SVN numbers)
///
/// # Usage
/// Use [`GalSvn::new`] to construct with validation. Retrieve inner value via
/// [`GalSvn::get`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GalSvn(u8);

/// BDS satellite PRN (pseudo-random number) identifier.
///
/// # Range
/// 1..=63 (standard BDS PRN numbers)
///
/// # Usage
/// Use [`BdsPrn::new`] to construct with validation. Retrieve inner value via
/// [`BdsPrn::get`]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BdsPrn(u8);

/// GLONASS satellite frequency slot.
///
/// # Range
/// -7..=6
///
/// # Usage
/// Use [`GloSlot::new`] to construct with validation. Retrieve inner value via
/// [`GloSlot::get`].
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GloSlot(i8);

impl GpsPrn {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 32;

    /// Creates a validated [`GpsPrn`].
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidPrn`] if value is outside valid range.
    pub fn new(prn: u8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&prn) {
            Ok(Self(prn))
        } else {
            Err(GorkaError::InvalidPrn(prn))
        }
    }

    /// Returns the PRN value.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl GalSvn {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 36;

    /// Creates a validated [`GalSvn`].
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidSvn`] if value is outside valid range.
    pub fn new(svn: u8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&svn) {
            Ok(Self(svn))
        } else {
            Err(GorkaError::InvalidSvn(svn))
        }
    }

    /// Returns the SVN value.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl BdsPrn {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 63;

    /// Creates a validated [`BdsPrn`].
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidPrn`] if value is outside valid range.
    pub fn new(prn: u8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&prn) {
            Ok(Self(prn))
        } else {
            Err(GorkaError::InvalidPrn(prn))
        }
    }

    /// Returns the PRN value.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl GloSlot {
    pub const MIN: i8 = -7;
    pub const MAX: i8 = 6;

    /// Creates a validated [`GloSlot`].
    ///
    /// # Errors
    /// Returns [`GorkaError::InvalidSlot`] if value is outside valid range.
    pub fn new(slot: i8) -> Result<Self, GorkaError> {
        if (Self::MIN..=Self::MAX).contains(&slot) {
            Ok(Self(slot))
        } else {
            Err(GorkaError::InvalidSlot(slot))
        }
    }

    /// Returns the frequency slot value.
    pub const fn get(self) -> i8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_prn_valid() {
        for prn in GpsPrn::MIN..=GpsPrn::MAX {
            let gps = GpsPrn::new(prn).unwrap();
            assert_eq!(gps.get(), prn);
        }
    }

    #[test]
    fn test_gps_prn_invalid() {
        assert!(GpsPrn::new(0).is_err());
        assert!(GpsPrn::new(33).is_err());
    }

    #[test]
    fn test_gal_svn_valid() {
        for svn in GalSvn::MIN..=GalSvn::MAX {
            let gal = GalSvn::new(svn).unwrap();
            assert_eq!(gal.get(), svn);
        }
    }

    #[test]
    fn test_gal_svn_invalid() {
        assert!(GalSvn::new(0).is_err());
        assert!(GalSvn::new(37).is_err());
    }

    #[test]
    fn test_bds_prn_valid() {
        for prn in BdsPrn::MIN..=BdsPrn::MAX {
            let bds = BdsPrn::new(prn).unwrap();
            assert_eq!(bds.get(), prn);
        }
    }

    #[test]
    fn test_bds_prn_invalid() {
        assert!(BdsPrn::new(0).is_err());
        assert!(BdsPrn::new(64).is_err());
    }

    #[test]
    fn test_glo_slot_valid() {
        for slot in GloSlot::MIN..=GloSlot::MAX {
            let s = GloSlot::new(slot).unwrap();
            assert_eq!(s.get(), slot);
        }
    }

    #[test]
    fn test_glo_slot_invalid() {
        assert!(GloSlot::new(-8).is_err());
        assert!(GloSlot::new(7).is_err());
    }

    #[test]
    fn test_gps_prn_bounds() {
        assert!(GpsPrn::new(GpsPrn::MIN).is_ok());
        assert!(GpsPrn::new(GpsPrn::MAX).is_ok());
    }

    #[test]
    fn test_gal_svn_bounds() {
        assert!(GalSvn::new(GalSvn::MIN).is_ok());
        assert!(GalSvn::new(GalSvn::MAX).is_ok());
    }

    #[test]
    fn test_glo_slot_bounds() {
        assert!(GloSlot::new(GloSlot::MIN).is_ok());
        assert!(GloSlot::new(GloSlot::MAX).is_ok());
    }
}
