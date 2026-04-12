use alloc::{format, vec::Vec};

use crate::FormatVersion;

#[derive(Debug, Clone)]
pub struct CompatibilityInfo {
    pub reader_version: FormatVersion,
    pub dump_version: FormatVersion,
    pub can_read: bool,
    pub can_write: bool,
    pub requires_migration: bool,
    pub warnings: alloc::vec::Vec<alloc::string::String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compatibility_info() {
        let info = CompatibilityInfo::check(FormatVersion::V1, FormatVersion::V1);

        assert!(info.can_read);
        assert!(info.can_write);
        assert!(info.warnings.is_empty());
    }
}
