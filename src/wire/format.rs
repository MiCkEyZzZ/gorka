use crate::GorkaError;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatVersion {
    V1 = 1,
}

impl FormatVersion {
    #[inline]
    pub const fn current() -> Self {
        Self::V1
    }

    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn can_read(
        &self,
        other: FormatVersion,
    ) -> bool {
        *self >= other
    }

    pub fn can_write(
        &self,
        target: FormatVersion,
    ) -> bool {
        *self >= target
    }

    pub fn is_deprecated(&self) -> bool {
        false
    }

    pub fn description(&self) -> &'static str {
        match self {
            FormatVersion::V1 => "Version 1 (initial Gorka encoding)",
        }
    }
}

impl TryFrom<u8> for FormatVersion {
    type Error = GorkaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::V1),
            other => Err(GorkaError::InvalidVersion(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version() {
        assert_eq!(FormatVersion::current(), FormatVersion::V1);
    }

    #[test]
    fn test_as_u8() {
        assert_eq!(FormatVersion::V1.as_u8(), 1);
    }

    #[test]
    fn test_try_from_valid() {
        let v = FormatVersion::try_from(1).unwrap();

        assert_eq!(v, FormatVersion::V1);
    }

    #[test]
    fn test_try_from_invalid() {
        let err = FormatVersion::try_from(99).unwrap_err();

        assert!(matches!(err, GorkaError::InvalidVersion(99)));
    }
}
