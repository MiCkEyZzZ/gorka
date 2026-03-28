use crate::GorkaError;

pub const CHUNK_MAGIC: u32 = 0x474F524B;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatVersion {
    V1 = 1,
}

impl FormatVersion {
    #[inline]
    pub const fn current() -> Self {
        Self::V1
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
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
        let err = FormatVersion::try_from(2).unwrap_err();

        matches!(err, GorkaError::InvalidVersion(2));
    }

    #[test]
    fn test_chunk_magic() {
        assert_eq!(CHUNK_MAGIC, 0x474F524B);
    }
}
