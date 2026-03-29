use core::fmt;

#[derive(Debug)]
pub enum GorkaError {
    EmptyChunk,
    UnexpectedEof,
    InvalidSlot(i8),
    InvalidBitCount(u8),
    ValueTooLarge { value: u64, bits: u8 },
    InvalidVersion(u8),
    InvalidMagic(u32),
}

impl fmt::Display for GorkaError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::EmptyChunk => write!(f, "cannot encode empty chunk"),
            Self::UnexpectedEof => write!(f, "bit stream ended unexpectedly"),
            Self::InvalidSlot(k) => {
                write!(f, "GLONASS slot k={k} out of range [-7, +6]")
            }
            Self::InvalidBitCount(n) => {
                write!(f, "invalid bit count: {n} (must be <= 64)")
            }
            Self::ValueTooLarge { value, bits } => {
                write!(f, "value {value} does not fit into {bits} bits")
            }
            Self::InvalidVersion(v) => {
                write!(f, "invalid version: {v}")
            }
            Self::InvalidMagic(magic) => {
                write!(f, "invalid magic: 0x{magic:08x}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GorkaError {}
