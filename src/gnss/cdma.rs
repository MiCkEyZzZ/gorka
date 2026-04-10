use crate::{encode_i64, BitReader, BitWrite, GorkaError, MilliHz, RawBitWriter};

#[derive(Debug, Clone, Copy, Default)]
pub struct CdmaState {
    last: Option<i32>,
}

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
