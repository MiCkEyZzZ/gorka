//! CDMA Doppler encoding - GPS, Galileo, BeiDou.
//!
//! GPS, Galileo and BeiDou use **Code Division Multiple Access**: all
//! satellites in the same band share a single carrier frequency.  Unlike
//! GLONASS FDMA, there is no per-satellite frequency offset, so no baseline
//! correction is needed.
//!
//! [`CdmaState`] simply tracks the previous Doppler value for simple
//! delta coding without any EMA filter.
//!
//! # Wire format
//!
//! | Prefix | Payload         | Meaning             |
//! |--------|-----------------|---------------------|
//! | `0`    | 32-bit verbatim | First observation   |
//! | `10`   | —               | Delta == 0          |
//! | `110`  | 16-bit zigzag   | \|delta\| < 32 768  |
//! | `111`  | 32-bit verbatim | Large delta         |

use crate::{encode_i64, BitReader, BitWrite, GorkaError, MilliHz, RawBitWriter};

/// CDMA per-satellite Doppler delta state.
///
/// Stores the last observed Doppler for a single satellite. Create one
/// `CdmaState` per satellite track.
#[derive(Debug, Clone, Copy)]
pub struct CdmaState {
    last: Option<i32>,
}

impl CdmaState {
    /// Creates a new state (no previous observation).
    pub const fn new() -> Self {
        Self { last: None }
    }

    /// Resets state (call at the start of a new chunk).
    pub fn reset(&mut self) {
        self.last = None;
    }
}

impl Default for CdmaState {
    fn default() -> Self {
        Self::new()
    }
}

/// Encodes a CDMA Doppler value (GPS, Galileo and BeiDou).
///
/// No baseline correction - just simple delta coding.
pub fn encode_doppler_cdma(
    writer: &mut RawBitWriter,
    state: &mut CdmaState,
    observed: i32,
) -> Result<(), GorkaError> {
    match state.last {
        None => {
            writer.write_bit(false)?;
            writer.write_bits(observed as u64 & 0xFFFF_FFFF, 32)?;

            state.last = Some(observed);
        }
        Some(prev) => {
            let delta = observed as i64 - prev as i64;
            let zz = encode_i64(delta);

            if delta == 0 {
                writer.write_bits(0b10, 2)?;
            } else if zz < (1u64 << 16) {
                writer.write_bits(0b110, 3)?;
                writer.write_bits_signed(delta, 16)?;
            } else {
                writer.write_bits(0b111, 3)?;
                writer.write_bits(observed as u64 & 0xFFFF_FFFF, 32)?;
            }

            state.last = Some(observed);
        }
    }

    Ok(())
}

/// Decode a CDMA Doppler value.
pub fn decode_doppler_cdma(
    reader: &mut BitReader,
    state: &mut CdmaState,
) -> Result<MilliHz, GorkaError> {
    match state.last {
        None => {
            let _flag = reader.read_bit()?;
            let raw = reader.read_bits(32)? as u32 as i32;

            state.last = Some(raw);

            Ok(MilliHz(raw))
        }
        Some(prev) => {
            let b0 = reader.read_bit()?;
            let b1 = reader.read_bit()?;

            match (b0, b1) {
                (true, false) => Ok(MilliHz(prev)),
                (true, true) => {
                    let b2 = reader.read_bit()?;
                    let observed = if !b2 {
                        let delta = reader.read_bits_signed(16)? as i32;

                        prev.wrapping_add(delta)
                    } else {
                        reader.read_bits(32)? as u32 as i32
                    };

                    state.last = Some(observed);

                    Ok(MilliHz(observed))
                }
                _ => Err(GorkaError::UnexpectedEof),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn cdma_roundtrip(values: &[i32]) -> alloc::vec::Vec<i32> {
        let mut buf = vec![0u8; 4096];
        let mut state = CdmaState::new();
        let mut writer = RawBitWriter::new(&mut buf);
        for &v in values {
            encode_doppler_cdma(&mut writer, &mut state, v).unwrap();
        }
        let n = writer.bytes_written();
        let mut reader = BitReader::new(&buf[..n]);
        let mut state2 = CdmaState::new();
        let mut out = vec![];
        for _ in values {
            out.push(decode_doppler_cdma(&mut reader, &mut state2).unwrap().0);
        }
        out
    }

    #[test]
    fn test_cdma_state_new() {
        let s = CdmaState::default();

        assert_eq!(s.last, None);
    }

    #[test]
    fn test_cdma_state_reset() {
        let mut s = CdmaState::new();

        s.last = Some(1_500_000);
        s.reset();

        assert_eq!(s.last, None);
    }

    #[test]
    fn test_cdma_roundtrip_single() {
        assert_eq!(cdma_roundtrip(&[1_575_420_000]), [1_575_420_000]);
    }

    #[test]
    fn test_cdma_roundtrip_smooth() {
        let vals: alloc::vec::Vec<i32> = (0..64).map(|i| 1_500_000 + i * 5).collect();

        assert_eq!(cdma_roundtrip(&vals), vals);
    }

    #[test]
    fn test_cdma_roundtrip_negative() {
        let vals: alloc::vec::Vec<i32> = (0..32).map(|i| -2_500_000 + i * 100).collect();

        assert_eq!(cdma_roundtrip(&vals), vals);
    }

    #[test]
    fn test_cdma_roundtrip_large_delta() {
        let vals = [0i32, 0, 5_000_000, 5_000_000, -3_000_000];

        assert_eq!(cdma_roundtrip(&vals), vals);
    }

    #[test]
    fn test_cdma_roundtrip_zero_delta_path() {
        // All same → all '10' prefix after first
        let vals = [1_200_000i32; 16];

        assert_eq!(cdma_roundtrip(&vals), vals);
    }

    #[test]
    fn test_cdma_no_ema_no_baseline_drift() {
        // Unlike FDMA, CDMA has no baseline that could drift —
        // decoded value must exactly equal observed for any input.
        let vals = [0i32, 1000, 2000, 1500, 500, 0, -500, 1_000_000];

        assert_eq!(cdma_roundtrip(&vals), vals);
    }
}
