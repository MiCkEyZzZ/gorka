use alloc::vec::Vec;

use crate::{BitReader, GlonassSample, GorkaError, MilliHz, Millimeter, VersionUtils};

const N_SLOT: usize = 14;

struct DecoderState {
    last_ts: u64,
    last_delta_ts: u64,
    last_slot: i8,
    last_cn0: u8,
    last_pr_mm: Millimeter,
    last_pr_delta: Millimeter,
    last_doppler: [Option<i32>; N_SLOT],
    last_phase: Option<i64>,
    last_phase_delta: Option<i64>,
}

pub struct GlonassDecoder;

impl DecoderState {
    fn from_first(sample: &GlonassSample) -> Self {
        let mut last_doppler = [None; N_SLOT];

        last_doppler[slot_to_idx(sample.slot)] = Some(sample.doppler_millihz.0);

        Self {
            last_ts: sample.timestamp_ms,
            last_delta_ts: 0,
            last_slot: sample.slot,
            last_cn0: sample.cn0_dbhz,
            last_pr_mm: sample.pseudorange_mm,
            last_pr_delta: Millimeter(0),
            last_doppler,
            last_phase: sample.carrier_phase_cycles,
            last_phase_delta: None,
        }
    }
}

impl GlonassDecoder {
    pub fn decode_chunk(data: &[u8]) -> Result<Vec<GlonassSample>, GorkaError> {
        if data.len() < 9 {
            return Err(GorkaError::UnexpectedEof);
        }

        let _version = VersionUtils::read_chunk_version(&data[0..9])?;
        let count = u32::from_le_bytes(data[5..9].try_into().unwrap()) as usize;

        if count == 0 {
            return Ok(Vec::new());
        }

        let (first, first_len) = decode_verbatim(&data[9..])?;
        let mut samples = Vec::with_capacity(count);

        samples.push(first.clone());

        if count == 1 {
            return Ok(samples);
        }

        let bitstream_start = 9 + first_len;

        if data.len() < bitstream_start {
            return Err(GorkaError::UnexpectedEof);
        }

        let mut reader = BitReader::new(&data[bitstream_start..]);
        let mut state = DecoderState::from_first(&first);

        for _ in 1..count {
            let sample = decode_delta(&mut reader, &mut state)?;

            samples.push(sample);
        }

        Ok(samples)
    }
}

fn decode_verbatim(data: &[u8]) -> Result<(GlonassSample, usize), GorkaError> {
    if data.len() < 23 {
        return Err(GorkaError::UnexpectedEof);
    }

    let timestamp_ms = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let slot = data[8] as i8;
    let cn0_dbhz = data[9];
    let pseudorange = i64::from_le_bytes(data[10..18].try_into().unwrap());
    let doppler = i32::from_le_bytes(data[18..22].try_into().unwrap());
    let phase_flag = data[22];

    let (carrier_phase_cycles, consumed) = match phase_flag {
        0 => (None, 23),
        1 => {
            if data.len() < 31 {
                return Err(GorkaError::UnexpectedEof);
            }

            let p = i64::from_le_bytes(data[23..31].try_into().unwrap());
            (Some(p), 31)
        }
        _ => return Err(GorkaError::UnexpectedEof),
    };
    Ok((
        GlonassSample {
            timestamp_ms,
            slot,
            cn0_dbhz,
            pseudorange_mm: Millimeter(pseudorange),
            doppler_millihz: MilliHz(doppler),
            carrier_phase_cycles,
        },
        consumed,
    ))
}

fn decode_delta(
    reader: &mut BitReader,
    state: &mut DecoderState,
) -> Result<GlonassSample, GorkaError> {
    let timestamp_ms = decode_timestamp(reader, state)?;
    let slot = decode_slot(reader, state)?;
    let cn0_dbhz = decode_cn0(reader, state)?;
    let pseudorange_mm = decode_pseudorange(reader, state)?;
    let doppler_millihz = decode_doppler(reader, state, slot)?;
    let carrier_phase_cycles = decode_carrier_phase(reader, state)?;

    Ok(GlonassSample {
        timestamp_ms,
        slot,
        cn0_dbhz,
        pseudorange_mm,
        doppler_millihz,
        carrier_phase_cycles,
    })
}

fn decode_timestamp(
    reader: &mut BitReader,
    state: &mut DecoderState,
) -> Result<u64, GorkaError> {
    let b0 = reader.read_bit()?;

    if !b0 {
        // dod == 0 -> delta не меняется
        let timestamp = state.last_ts.wrapping_add(state.last_delta_ts);

        state.last_ts = timestamp;

        return Ok(timestamp);
    }

    let b1 = reader.read_bit()?;

    if !b1 {
        // '10' + 7b
        let dod = reader.read_bits_signed(7)?;
        let delta = (state.last_delta_ts as i64).wrapping_add(dod) as u64;
        let timestamp = state.last_ts.wrapping_add(delta);

        state.last_delta_ts = delta;
        state.last_ts = timestamp;

        return Ok(timestamp);
    }

    let b2 = reader.read_bit()?;

    if !b2 {
        // '110' + 9b
        let dod = reader.read_bits_signed(9)?;
        let delta = (state.last_delta_ts as i64).wrapping_add(dod) as u64;
        let timestamp = state.last_ts.wrapping_add(delta);

        state.last_delta_ts = delta;
        state.last_ts = timestamp;

        return Ok(timestamp);
    }

    // '111' + 64b verbatim
    let timestamp = reader.read_bits(64)?;
    let delta = timestamp.wrapping_sub(state.last_ts);

    state.last_delta_ts = delta;
    state.last_ts = timestamp;

    Ok(timestamp)
}

fn decode_slot(
    reader: &mut BitReader,
    state: &mut DecoderState,
) -> Result<i8, GorkaError> {
    let changed = reader.read_bit()?;

    if !changed {
        return Ok(state.last_slot);
    }

    let idx = reader.read_bits(4)?;
    let slot = idx_to_slot(idx);

    state.last_slot = slot;

    Ok(slot)
}

fn decode_cn0(
    reader: &mut BitReader,
    state: &mut DecoderState,
) -> Result<u8, GorkaError> {
    let has_delta = reader.read_bit()?;

    if !has_delta {
        return Ok(state.last_cn0);
    }

    let delta = reader.read_bits_signed(9)? as i16;
    let cn0 = (state.last_cn0 as i16 + delta) as u8;

    state.last_cn0 = cn0;

    Ok(cn0)
}

fn decode_pseudorange(
    reader: &mut BitReader,
    state: &mut DecoderState,
) -> Result<Millimeter, GorkaError> {
    let b0 = reader.read_bit()?;

    if !b0 {
        let pr = Millimeter(state.last_pr_mm.0 + state.last_pr_delta.0);

        state.last_pr_mm = pr;

        return Ok(pr);
    }

    let b1 = reader.read_bit()?;

    if !b1 {
        // '10' + 10b
        let dod = reader.read_bits_signed(10)?;
        let delta = state.last_pr_delta.0 + dod;
        let pr = Millimeter(state.last_pr_mm.0 + delta);

        state.last_pr_delta = Millimeter(delta);
        state.last_pr_mm = pr;

        return Ok(pr);
    }

    let b2 = reader.read_bit()?;

    if !b2 {
        let dod = reader.read_bits_signed(20)?;
        let delta = state.last_pr_delta.0 + dod;
        let pr = Millimeter(state.last_pr_mm.0 + delta);

        state.last_pr_delta = Millimeter(delta);
        state.last_pr_mm = pr;

        return Ok(pr);
    }

    // '111' + 64b verbatim
    let raw = reader.read_bits(64)? as i64;
    let pr = Millimeter(raw);
    let delta = raw - state.last_pr_mm.0;

    state.last_pr_delta = Millimeter(delta);
    state.last_pr_mm = pr;

    Ok(pr)
}

fn decode_doppler(
    reader: &mut BitReader,
    state: &mut DecoderState,
    slot: i8,
) -> Result<MilliHz, GorkaError> {
    let idx = slot_to_idx(slot);

    let doppler = match state.last_doppler[idx] {
        None => {
            // Encoder пишет: '0' + 32b verbatim
            let _flag = reader.read_bit()?; // всегда false для первого появления
            let raw = reader.read_bits(32)? as u32 as i32;
            state.last_doppler[idx] = Some(raw);
            MilliHz(raw)
        }
        Some(prev) => {
            // Encoder пишет: '10' | '110' + 14b | '111' + 32b
            let b0 = reader.read_bit()?;
            let b1 = reader.read_bit()?;

            match (b0, b1) {
                // '10' — delta == 0
                (true, false) => MilliHz(prev),

                // '11x' — читаем третий бит
                (true, true) => {
                    let b2 = reader.read_bit()?;
                    if !b2 {
                        // '110' + 14b
                        let delta = reader.read_bits_signed(14)?;
                        let doppler = (prev as i64 + delta) as i32;
                        state.last_doppler[idx] = Some(doppler);
                        MilliHz(doppler)
                    } else {
                        // '111' + 32b verbatim
                        let raw = reader.read_bits(32)? as u32 as i32;
                        state.last_doppler[idx] = Some(raw);
                        MilliHz(raw)
                    }
                }

                // (false, _) — не должно быть: encoder всегда начинает с '1' для seen-before
                _ => return Err(GorkaError::UnexpectedEof),
            }
        }
    };

    state.last_doppler[slot_to_idx(slot)] = Some(doppler.0);

    Ok(doppler)
}

fn decode_carrier_phase(
    reader: &mut BitReader,
    state: &mut DecoderState,
) -> Result<Option<i64>, GorkaError> {
    let b0 = reader.read_bit()?;
    let b1 = reader.read_bit()?;

    let phase = match (b0, b1) {
        // '00' — None → None
        (false, false) => None,

        // '01' — Some → None
        (false, true) => None,

        // '10' — None → Some, verbatim 64b
        (true, false) => {
            let p = reader.read_bits(64)? as i64;

            // last_phase_delta остаётся None: первая пара ещё не случилась
            Some(p)
        }

        // '11' — Some → Some: DoD ветка
        (true, true) => {
            let prev = state.last_phase.ok_or(GorkaError::UnexpectedEof)?;
            let prev_d = state.last_phase_delta.unwrap_or(0);

            let b2 = reader.read_bit()?;

            if !b2 {
                // '110' — dod == 0 → delta не изменилась
                let delta = prev_d;
                let curr = prev + delta;

                state.last_phase_delta = Some(delta);

                Some(curr)
            } else {
                let b3 = reader.read_bit()?;

                if !b3 {
                    // '1110' + 32b zigzag dod
                    let dod = reader.read_bits_signed(32)?;
                    let delta = prev_d + dod;
                    let curr = prev + delta;

                    state.last_phase_delta = Some(delta);

                    Some(curr)
                } else {
                    // '1111' + 64b verbatim (сброс DoD)
                    let curr = reader.read_bits(64)? as i64;

                    state.last_phase_delta = None;

                    Some(curr)
                }
            }
        }
    };

    state.last_phase = phase;

    Ok(phase)
}

#[inline]
fn slot_to_idx(slot: i8) -> usize {
    (slot + 7) as usize
}

#[inline]
fn idx_to_slot(idx: u64) -> i8 {
    idx as i8 - 7
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::codec::GlonassEncoder;

    const BASE_TS: u64 = 1_700_000_000_000;

    fn sample(
        i: u64,
        slot: i8,
    ) -> GlonassSample {
        GlonassSample {
            timestamp_ms: BASE_TS + i,
            slot,
            cn0_dbhz: 40 + (i % 10) as u8,
            pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),
            doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 50),
            carrier_phase_cycles: Some(100_000_i64 + i as i64 * 21 * (1 << 16)),
        }
    }

    fn constant_sample(
        timestamp_offset: u64,
        slot: i8,
    ) -> GlonassSample {
        GlonassSample {
            timestamp_ms: BASE_TS + timestamp_offset,
            slot,
            cn0_dbhz: 42,
            pseudorange_mm: Millimeter::new(21_500_000_000),
            doppler_millihz: MilliHz::new(1_200_500),
            carrier_phase_cycles: None,
        }
    }

    fn roundtrip(sample: &[GlonassSample]) -> Vec<GlonassSample> {
        let encoder = GlonassEncoder::encode_chunk(sample).expect("encode failed");

        GlonassDecoder::decode_chunk(&encoder).expect("decode failed")
    }

    fn assert_eq_sample(
        o: &GlonassSample,
        d: &GlonassSample,
        idx: usize,
    ) {
        assert_eq!(o.timestamp_ms, d.timestamp_ms, "sample[{idx}] timestamp");
        assert_eq!(o.slot, d.slot, "sample[{idx}] slot");
        assert_eq!(o.cn0_dbhz, d.cn0_dbhz, "sample[{idx}] cn0");
        assert_eq!(
            o.pseudorange_mm, d.pseudorange_mm,
            "sample[{idx}] pseudorange"
        );
        assert_eq!(
            o.doppler_millihz, d.doppler_millihz,
            "sample[{idx}] doppler"
        );
        assert_eq!(
            o.carrier_phase_cycles, d.carrier_phase_cycles,
            "sample[{idx}] carrier_phase"
        );
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        assert!(matches!(
            GlonassDecoder::decode_chunk(&[]).unwrap_err(),
            GorkaError::UnexpectedEof
        ));
    }

    #[test]
    fn test_decode_truncated_header_returns_error() {
        assert!(matches!(
            GlonassDecoder::decode_chunk(&[0x4B, 0x52, 0x4F, 0x47]).unwrap_err(),
            GorkaError::UnexpectedEof,
        ));
    }

    #[test]
    fn test_decode_wrong_magic_returns_error() {
        let mut buf = GlonassEncoder::encode_chunk(&[sample(0, 1)]).unwrap();

        buf[0] ^= 0xFF;

        assert!(matches!(
            GlonassDecoder::decode_chunk(&buf).unwrap_err(),
            GorkaError::InvalidMagic(_),
        ));
    }

    #[test]
    fn test_decode_wrong_version_returns_error() {
        let mut buf = GlonassEncoder::encode_chunk(&[sample(0, 1)]).unwrap();

        buf[4] = 99;

        assert!(matches!(
            GlonassDecoder::decode_chunk(&buf).unwrap_err(),
            GorkaError::InvalidVersion(99),
        ));
    }

    #[test]
    fn test_roundtrip_1_sample() {
        let orig = vec![sample(0, 1)];
        let dec = roundtrip(&orig);

        assert_eq!(dec.len(), 1);
        assert_eq_sample(&orig[0], &dec[0], 0);
    }

    #[test]
    fn test_roundtrip_10_samples() {
        let orig: Vec<_> = (0..10).map(|i| sample(i, 1)).collect();
        let dec = roundtrip(&orig);

        assert_eq!(dec.len(), 10);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq_sample(o, d, i);
        }
    }

    #[test]
    fn test_roundtrip_100_samples() {
        let orig: Vec<_> = (0..100).map(|i| sample(i, 2)).collect();
        let dec = roundtrip(&orig);

        assert_eq!(dec.len(), 100);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq_sample(o, d, i);
        }
    }

    #[test]
    fn test_roundtrip_pseudorange_1mm_precision() {
        let orig: Vec<_> = (0..64u64)
            .map(|i| GlonassSample {
                pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64),
                ..constant_sample(i, 1)
            })
            .collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.pseudorange_mm, d.pseudorange_mm, "pr[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_doppler_1mhz_precision() {
        let orig: Vec<_> = (0..64i32)
            .map(|i| GlonassSample {
                doppler_millihz: MilliHz::new(1_200_000 + i),
                ..constant_sample(i as u64, 0)
            })
            .collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.doppler_millihz, d.doppler_millihz, "doppler[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_negative_doppler() {
        let orig: Vec<_> = (0..32u64)
            .map(|i| GlonassSample {
                doppler_millihz: MilliHz::new(-3_000_000 + i as i32 * 100),
                ..constant_sample(i, -5)
            })
            .collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.doppler_millihz, d.doppler_millihz, "neg_doppler[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_cn0_variation() {
        let vals: &[u8] = &[20, 25, 42, 50, 35, 22, 44, 48, 30];
        let orig: Vec<_> = vals
            .iter()
            .enumerate()
            .map(|(i, &cn0)| GlonassSample {
                cn0_dbhz: cn0,
                ..constant_sample(i as u64, 1)
            })
            .collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.cn0_dbhz, d.cn0_dbhz, "cn0[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_all_valid_slots() {
        for slot in -7_i8..=6 {
            let orig: Vec<_> = (0..32).map(|i| sample(i, slot)).collect();
            let dec = roundtrip(&orig);

            for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
                assert_eq_sample(o, d, i);
            }
        }
    }

    #[test]
    fn test_roundtrip_slot_change_within_chunk() {
        let mut orig = Vec::new();

        for i in 0..32u64 {
            orig.push(sample(i * 2, 1));
            orig.push(sample(i * 2 + 1, -3));
        }

        let dec = roundtrip(&orig);

        assert_eq!(dec.len(), orig.len());

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq_sample(o, d, i);
        }
    }

    #[test]
    fn test_roundtrip_no_carrier_phase() {
        let orig: Vec<_> = (0..32).map(|i| constant_sample(i, 0)).collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.carrier_phase_cycles, d.carrier_phase_cycles, "phase[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_carrier_phase_constant() {
        let orig: Vec<_> = (0..32).map(|i| sample(i, 1)).collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.carrier_phase_cycles, d.carrier_phase_cycles, "phase[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_carrier_phase_acquired_mid_stream() {
        let mut orig: Vec<_> = (0..8).map(|i| constant_sample(i, 0)).collect();

        for i in 8..16u64 {
            orig.push(GlonassSample {
                timestamp_ms: BASE_TS + i,
                carrier_phase_cycles: Some(i as i64 * (1 << 16)),
                ..constant_sample(i, 0)
            });
        }

        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.carrier_phase_cycles, d.carrier_phase_cycles, "phase[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_carrier_phase_lost_mid_stream() {
        let mut orig: Vec<_> = (0..8).map(|i| sample(i, 0)).collect();
        for i in 8..16u64 {
            orig.push(GlonassSample {
                carrier_phase_cycles: None,
                ..constant_sample(i, 0)
            });
        }
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.carrier_phase_cycles, d.carrier_phase_cycles, "phase[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_carrier_phase_reacquired() {
        let mk = |ts: u64, phase: Option<i64>| GlonassSample {
            carrier_phase_cycles: phase,
            ..constant_sample(ts, 2)
        };
        let orig = vec![
            mk(0, None),
            mk(1, None),
            mk(2, Some(1_000_000)),
            mk(3, Some(1_021_000)),
            mk(4, None),
            mk(5, None),
            mk(6, Some(2_000_000)),
            mk(7, Some(2_021_000)),
        ];
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.carrier_phase_cycles, d.carrier_phase_cycles, "phase[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_large_timestamp_gap() {
        let orig = vec![
            constant_sample(0, 0),
            constant_sample(1, 0),
            GlonassSample {
                timestamp_ms: BASE_TS + 10_001,
                ..constant_sample(10_001, 0)
            },
        ];

        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq_sample(o, d, i);
        }
    }

    #[test]
    fn test_roundtrip_irregular_timestamps() {
        let offsets = [0u64, 1, 3, 8, 108];
        let orig: Vec<_> = offsets.iter().map(|&t| constant_sample(t, 1)).collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.timestamp_ms, d.timestamp_ms, "ts[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_large_pseudorange_jump() {
        let mut orig = vec![constant_sample(0, 0), constant_sample(1, 0)];

        orig.push(GlonassSample {
            pseudorange_mm: Millimeter::new(21_500_000_000 + 1_000_000),
            ..constant_sample(2, 0)
        });

        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.pseudorange_mm, d.pseudorange_mm, "pr[{i}]");
        }
    }
}
