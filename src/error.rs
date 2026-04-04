use core::fmt;

// ИЗМЕНЕНИЯ v0.3.0:
// 1. Добавлен #[non_exhaustive] - теперь добавление новых вариантов не
//    сявляется breaking change. Пользователь v0.2.0 обязаны добавить `_ => {
//    ... }` ветку в свой match - это единственный breaking change при
//    обновлении до v0.3.0, о котором мы предупреждаем в CHANGELOG.
// 2. Добавлен вариант BufferFull для StreamEncoder.
//
// Все существующие варианты и их типы не изменились.
#[non_exhaustive]
#[derive(Debug)]
pub enum GorkaError {
    /// Attempted to encode a chunk with no sample
    EmptyChunk,

    /// Bit stream ended before all expected bits were read
    UnexpectedEof,

    /// GLONASS frequency slot `k` is outside the valid range [-7, +6]
    InvalidSlot(i8),

    /// Invalid satellite PRN (out of allowed range)
    InvalidPrn(u8),

    /// Invalid C/N0 value (carrier-to-noise ratio)
    /// Typically expected to be within a range like [0..=60] dB-Hz
    /// (implementation-dependent)
    InvalidCn0(u8),

    /// Requested bit count exceeds 64
    InvalidBitCount(u8),

    /// Value does not fit into the requested number of bits.
    ValueTooLarge { value: u64, bits: u8 },

    /// Value does not fit into the requested number of bits
    InvalidVersion(u8),

    /// Chunk header contains an unrecognised format version byte
    InvalidMagic(u32),

    /// Pseudorange (in mm) is outside the physically plausible range for a
    /// GLONASS satellite [PSEUDORANGE_MIN_MM, PSEUDORANGE_MAX_MM]
    ///
    /// Stored value is the rejected raw millimetre value
    InvalidPseudorange(i64),

    /// Doppler shift (in mHz) exceeds the plausible magnitude bound
    /// `±DOPPLER_MAX_MHZ` (±5 000 000 mHz = ±5000 Hz).
    ///
    /// Stored value is the rejected raw millihertz value.
    InvalidDoppler(i32),

    /// A sample's `timestamp` does not match the frame's epoch timestamp
    TimestampMismatch { frame: u64, sample: u64 },

    /// A `GnssFrame` already contains an observation for the given slot.
    DuplicateSlot(i8),

    /// A `GnssFrame` is already at capacity (`MAX_GLONASS_SATS` observations).
    FrameFull,

    /// The fixed-size output buffer provided to \[`StreamEncoder`\] is full.
    ///
    /// The sample was NOT written. Encoder state is unchanged — the caller
    /// can flush the current chunk and retry with a fresh buffer.
    BufferFull,
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
            Self::InvalidPrn(prn) => {
                write!(f, "invalid satellite PRN: {prn}")
            }
            Self::InvalidCn0(cn0) => {
                write!(f, "invalid C/N0 value: {cn0} dB-Hz")
            }
            Self::InvalidBitCount(n) => {
                write!(f, "invalid bit count: {n} (must be <= 64)")
            }
            Self::ValueTooLarge { value, bits } => {
                write!(f, "value {value} does not fit into {bits} bits")
            }
            Self::InvalidVersion(v) => {
                write!(f, "invalid format version: {v}")
            }
            Self::InvalidMagic(magic) => {
                write!(f, "invalid chunk magic: 0x{magic:08x}")
            }
            Self::InvalidPseudorange(mm) => {
                write!(
                    f,
                    "pseudorange {mm} mm is outside plausible GLONASS range \
                     [19_100_000_000, 25_600_000_000] mm"
                )
            }
            Self::InvalidDoppler(mhz) => {
                write!(
                    f,
                    "doppler {mhz} mHz exceeds plausible magnitude bound \
                     ±5_000_000 mHz (±5000 Hz)"
                )
            }
            Self::TimestampMismatch { frame, sample } => {
                write!(
                    f,
                    "sample timestamp {sample} ms does not match frame epoch {frame} ms"
                )
            }
            Self::DuplicateSlot(k) => {
                write!(f, "frame already contains an observation for slot k={k}")
            }
            Self::FrameFull => {
                write!(f, "GnssFrame is full (capacity: 14 observation)")
            }
            Self::BufferFull => write!(f, "StreamEncoder output buffer is full; flush and retry"),
            // NOTE: при добавлении нового варианта сюда, также добавьте Display-ветку. Компилятор
            // выдаст warning если забудете.
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GorkaError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;

    #[test]
    fn display_invalid_pseudorange() {
        let e = GorkaError::InvalidPseudorange(-1);

        assert!(e.to_string().contains("-1 mm"));
    }

    #[test]
    fn test_display_invalid_doppler() {
        let e = GorkaError::InvalidDoppler(9_999_999);

        assert!(e.to_string().contains("9999999 mHz"));
    }

    #[test]
    fn test_display_timestamp_mismatch() {
        let e = GorkaError::TimestampMismatch {
            frame: 1000,
            sample: 2000,
        };
        let s = e.to_string();

        assert!(s.contains("2000") && s.contains("1000"));
    }

    #[test]
    fn test_display_duplicate_slot() {
        let e = GorkaError::DuplicateSlot(-3);

        assert!(e.to_string().contains("k=-3"));
    }

    #[test]
    fn test_display_frame_full() {
        assert!(GorkaError::FrameFull.to_string().contains("14"));
    }

    #[test]
    fn test_display_buffer_full() {
        let s = GorkaError::BufferFull.to_string();
        assert!(s.contains("flush") && s.contains("StreamEncoder"));
    }
}
