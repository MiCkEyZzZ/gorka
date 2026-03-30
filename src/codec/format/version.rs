use crate::GorkaError;

pub const CHUNK_MAGIC: u32 = 0x474F524B;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatVersion {
    V1 = 1,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct CompatibilityInfo {
    pub reader_version: FormatVersion,
    pub dump_version: FormatVersion,
    pub can_read: bool,
    pub can_write: bool,
    pub requires_migration: bool,
    pub warnings: alloc::vec::Vec<alloc::string::String>,
}

pub struct VersionUtils;

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

impl CompatibilityInfo {
    pub fn check(
        reader_version: FormatVersion,
        dump_version: FormatVersion,
    ) -> Self {
        let can_read = reader_version.can_read(dump_version);
        let can_write = reader_version.can_write(dump_version);
        let requires_migration = dump_version.is_deprecated();

        let mut warnings = Vec::new();

        if requires_migration {
            warnings.push(format!(
                "Dump version {} is deprecated. Consider upgrating.",
                dump_version.as_u8(),
            ));
        }

        if dump_version > reader_version {
            warnings.push(format!(
                "Dump version {} is newer than reader version {}. Some features may not be supported",
                dump_version.as_u8(),
                reader_version.as_u8(),
            ));
        }

        if !can_read {
            warnings.push(format!(
                "Reader version {} cannot read dump version {}",
                reader_version.as_u8(),
                dump_version.as_u8(),
            ));
        }

        CompatibilityInfo {
            reader_version,
            dump_version,
            can_read,
            can_write,
            requires_migration,
            warnings,
        }
    }
}

impl VersionUtils {
    pub fn read_chunk_version(header: &[u8]) -> Result<FormatVersion, GorkaError> {
        if header.len() < 5 {
            return Err(GorkaError::UnexpectedEof);
        }

        let magic = u32::from_le_bytes(header[0..4].try_into().unwrap());

        if magic != CHUNK_MAGIC {
            return Err(GorkaError::InvalidMagic(magic));
        }

        FormatVersion::try_from(header[4])
    }

    pub fn write_chunk_header(
        version: FormatVersion,
        sample_count: u32,
    ) -> [u8; 9] {
        let mut buf = [0u8; 9];

        buf[0..4].copy_from_slice(&CHUNK_MAGIC.to_le_bytes());
        buf[4] = version.as_u8();
        buf[5..9].copy_from_slice(&sample_count.to_le_bytes());
        buf
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
    fn test_compatibility_info() {
        let info = CompatibilityInfo::check(FormatVersion::V1, FormatVersion::V1);

        assert!(info.can_read);
        assert!(info.can_write);
        assert!(info.warnings.is_empty());
    }

    #[test]
    fn test_chunk_header_roundtrip() {
        let header = VersionUtils::write_chunk_header(FormatVersion::V1, 42);

        assert_eq!(header.len(), 9);

        let ver = VersionUtils::read_chunk_version(&header).unwrap();

        assert_eq!(ver, FormatVersion::V1);
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
    fn test_invalid_magic() {
        let mut header = VersionUtils::write_chunk_header(FormatVersion::V1, 1);

        header[0] = 0x00;

        let err = VersionUtils::read_chunk_version(&header).unwrap_err();

        assert!(matches!(err, GorkaError::InvalidMagic(_)));
    }

    #[test]
    fn test_try_from_invalid() {
        let err = FormatVersion::try_from(99).unwrap_err();

        assert!(matches!(err, GorkaError::InvalidVersion(99)));
    }

    #[test]
    fn test_chunk_magic() {
        assert_eq!(CHUNK_MAGIC, 0x474F524B);
    }
}
