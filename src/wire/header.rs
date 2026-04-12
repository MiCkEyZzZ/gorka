use crate::{FormatVersion, GorkaError};

pub const CHUNK_MAGIC: u32 = 0x474F524B;

pub struct VersionUtils;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_header_roundtrip() {
        let header = VersionUtils::write_chunk_header(FormatVersion::V1, 42);

        assert_eq!(header.len(), 9);

        let ver = VersionUtils::read_chunk_version(&header).unwrap();

        assert_eq!(ver, FormatVersion::V1);
    }

    #[test]
    fn test_invalid_magic() {
        let mut header = VersionUtils::write_chunk_header(FormatVersion::V1, 1);

        header[0] = 0x00;

        let err = VersionUtils::read_chunk_version(&header).unwrap_err();

        assert!(matches!(err, GorkaError::InvalidMagic(_)));
    }

    #[test]
    fn test_chunk_magic() {
        assert_eq!(CHUNK_MAGIC, 0x474F524B);
    }
}
