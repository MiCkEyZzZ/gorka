use alloc::vec::Vec;

use crate::{
    BitReader, DbHz, GloSlot, GlonassSample, GorkaError, MilliHz, Millimeter, VersionUtils,
};

const N_SLOT: usize = 14;

struct DecoderState {
    last_ts: u64,
    last_delta_ts: u64,
    last_slot: GloSlot,
    last_cn0: DbHz,
    last_pr_mm: Millimeter,
    last_pr_delta: Millimeter,
    last_doppler: [Option<i32>; N_SLOT],
    last_phase: Option<i64>,
    last_phase_delta: Option<i64>,
}

pub struct GlonassDecoder;

pub struct DecodeIter<'a> {
    reader: BitReader<'a>,
    state: Option<DecoderState>,
    remaining: usize,
    errored: bool,
    pending: Option<GlonassSample>,
}

impl DecoderState {
    fn from_first(sample: &GlonassSample) -> Self {
        let mut last_doppler = [None; N_SLOT];

        last_doppler[slot_to_idx(sample.slot)] = Some(sample.doppler_millihz.as_i32());

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

    #[allow(clippy::needless_range_loop)]
    pub fn decode_into(
        data: &[u8],
        out: &mut [GlonassSample],
    ) -> Result<usize, GorkaError> {
        if data.len() < 9 {
            return Err(GorkaError::UnexpectedEof);
        }

        let _version = VersionUtils::read_chunk_version(&data[0..9])?;
        let count = u32::from_le_bytes(data[5..9].try_into().unwrap()) as usize;

        if count == 0 {
            return Ok(0);
        }

        if out.len() < count {
            return Err(GorkaError::BufferFull);
        }

        let (first, first_len) = decode_verbatim(&data[9..])?;

        out[0] = first.clone();

        if count == 1 {
            return Ok(1);
        }

        let bitstream_start = 9 + first_len;

        if data.len() < bitstream_start {
            return Err(GorkaError::UnexpectedEof);
        }

        let mut reader = BitReader::new(&data[bitstream_start..]);
        let mut state = DecoderState::from_first(&first);

        for i in 1..count {
            out[i] = decode_delta(&mut reader, &mut state)?;
        }

        Ok(count)
    }

    pub fn iter_chunk(data: &[u8]) -> Result<DecodeIter<'_>, GorkaError> {
        if data.len() < 9 {
            return Err(GorkaError::UnexpectedEof);
        }

        let _version = VersionUtils::read_chunk_version(&data[0..9])?;
        let count = u32::from_le_bytes(data[5..9].try_into().unwrap()) as usize;

        if count == 0 {
            // Пустой chunk: итератор сразу исчерпан
            return Ok(DecodeIter {
                reader: BitReader::new(&data[data.len()..]),
                state: None,
                remaining: 0,
                errored: false,
                // первый сэмпл "отдадим" через pending
                pending: None,
            });
        }

        let (first, first_len) = decode_verbatim(&data[9..])?;

        let bitstream_start = 9 + first_len;
        if data.len() < bitstream_start {
            return Err(GorkaError::UnexpectedEof);
        }

        let reader = BitReader::new(&data[bitstream_start..]);
        let state = DecoderState::from_first(&first);

        Ok(DecodeIter {
            reader,
            state: Some(state),
            remaining: count - 1, // первый уже "прочитан", осталось count-1
            errored: false,
            pending: Some(first), // первый сэмпл ждёт в очереди
        })
    }
}

impl<'a> DecodeIter<'a> {
    pub fn remaining(&self) -> usize {
        self.pending.is_some() as usize + self.remaining
    }
}

impl<'a> Iterator for DecodeIter<'a> {
    type Item = Result<GlonassSample, GorkaError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }

        // Первый сэмпл - verbatim, уже разобран, ждёт в `pending`
        if let Some(first) = self.pending.take() {
            return Some(Ok(first));
        }

        if self.remaining == 0 {
            return None;
        }

        let state = match self.state.as_mut() {
            Some(s) => s,
            None => {
                self.errored = true;

                return Some(Err(GorkaError::UnexpectedEof));
            }
        };

        match decode_delta(&mut self.reader, state) {
            Ok(sample) => {
                self.remaining -= 1;

                Some(Ok(sample))
            }
            Err(e) => {
                self.errored = true;

                Some(Err(e))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.remaining();

        (n, Some(n))
    }
}

impl<'a> ExactSizeIterator for DecodeIter<'a> {}

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
    let slot = GloSlot::new(slot)?;
    let cn0_dbhz = DbHz::new(cn0_dbhz)?;

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
) -> Result<GloSlot, GorkaError> {
    let changed = reader.read_bit()?;

    if !changed {
        return Ok(state.last_slot);
    }

    let idx = reader.read_bits(4)?;
    let slot = idx_to_slot(idx);

    if !(-7..=6).contains(&slot.get()) {
        return Err(GorkaError::UnexpectedEof);
    }

    state.last_slot = slot;

    Ok(slot)
}

fn decode_cn0(
    reader: &mut BitReader,
    state: &mut DecoderState,
) -> Result<DbHz, GorkaError> {
    let has_delta = reader.read_bit()?;

    if !has_delta {
        return Ok(state.last_cn0);
    }

    let delta = reader.read_bits_signed(9)? as i16;
    let cn0 = (state.last_cn0.get() as i16 + delta) as u8;

    let cn0 = DbHz::new(cn0)?;
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
    slot: GloSlot,
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
                    // '1111' + 64b verbatim (DoD reset)
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
fn slot_to_idx(slot: GloSlot) -> usize {
    (slot.get() + 7) as usize
}

#[inline]
fn idx_to_slot(idx: u64) -> GloSlot {
    GloSlot::new(idx as i8 - 7).unwrap()
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::{codec::GlonassEncoder, DbHz, GloSlot};

    const BASE_TS: u64 = 1_700_000_000_000;

    fn sample(
        i: u64,
        slot: GloSlot,
    ) -> GlonassSample {
        GlonassSample {
            timestamp_ms: BASE_TS + i,
            slot,
            cn0_dbhz: DbHz::new(40 + (i % 10) as u8).unwrap(),
            pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),
            doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 50),
            carrier_phase_cycles: Some(100_000_i64 + i as i64 * 21 * (1 << 16)),
        }
    }

    fn constant_sample(
        timestamp_offset: u64,
        slot: GloSlot,
    ) -> GlonassSample {
        GlonassSample {
            timestamp_ms: BASE_TS + timestamp_offset,
            slot,
            cn0_dbhz: DbHz::new(42).unwrap(),
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
        let mut buf = GlonassEncoder::encode_chunk(&[sample(0, GloSlot::new(1).unwrap())]).unwrap();

        buf[0] ^= 0xFF;

        assert!(matches!(
            GlonassDecoder::decode_chunk(&buf).unwrap_err(),
            GorkaError::InvalidMagic(_),
        ));
    }

    #[test]
    fn test_decode_wrong_version_returns_error() {
        let mut buf = GlonassEncoder::encode_chunk(&[sample(0, GloSlot::new(1).unwrap())]).unwrap();

        buf[4] = 99;

        assert!(matches!(
            GlonassDecoder::decode_chunk(&buf).unwrap_err(),
            GorkaError::InvalidVersion(99),
        ));
    }

    #[test]
    fn test_roundtrip_1_sample() {
        let orig = vec![sample(0, GloSlot::new(1).unwrap())];
        let dec = roundtrip(&orig);

        assert_eq!(dec.len(), 1);
        assert_eq_sample(&orig[0], &dec[0], 0);
    }

    #[test]
    fn test_roundtrip_10_samples() {
        let orig: Vec<_> = (0..10)
            .map(|i| sample(i, GloSlot::new(1).unwrap()))
            .collect();
        let dec = roundtrip(&orig);

        assert_eq!(dec.len(), 10);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq_sample(o, d, i);
        }
    }

    #[test]
    fn test_roundtrip_100_samples() {
        let orig: Vec<_> = (0..100)
            .map(|i| sample(i, GloSlot::new(2).unwrap()))
            .collect();
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
                ..constant_sample(i, GloSlot::new(1).unwrap())
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
                ..constant_sample(i as u64, GloSlot::new(0).unwrap())
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
                ..constant_sample(i, GloSlot::new(-5).unwrap())
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
                cn0_dbhz: DbHz::new(cn0).unwrap(),
                ..constant_sample(i as u64, GloSlot::new(1).unwrap())
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
            let orig: Vec<_> = (0..32)
                .map(|i| sample(i, GloSlot::new(slot).unwrap()))
                .collect();
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
            orig.push(sample(i * 2, GloSlot::new(1).unwrap()));
            orig.push(sample(i * 2 + 1, GloSlot::new(-3).unwrap()));
        }

        let dec = roundtrip(&orig);

        assert_eq!(dec.len(), orig.len());

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq_sample(o, d, i);
        }
    }

    #[test]
    fn test_roundtrip_no_carrier_phase() {
        let orig: Vec<_> = (0..32)
            .map(|i| constant_sample(i, GloSlot::new(0).unwrap()))
            .collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.carrier_phase_cycles, d.carrier_phase_cycles, "phase[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_carrier_phase_constant() {
        let orig: Vec<_> = (0..32)
            .map(|i| sample(i, GloSlot::new(1).unwrap()))
            .collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.carrier_phase_cycles, d.carrier_phase_cycles, "phase[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_carrier_phase_acquired_mid_stream() {
        let mut orig: Vec<_> = (0..8)
            .map(|i| constant_sample(i, GloSlot::new(0).unwrap()))
            .collect();

        for i in 8..16u64 {
            orig.push(GlonassSample {
                timestamp_ms: BASE_TS + i,
                carrier_phase_cycles: Some(i as i64 * (1 << 16)),
                ..constant_sample(i, GloSlot::new(0).unwrap())
            });
        }

        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.carrier_phase_cycles, d.carrier_phase_cycles, "phase[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_carrier_phase_lost_mid_stream() {
        let mut orig: Vec<_> = (0..8)
            .map(|i| sample(i, GloSlot::new(0).unwrap()))
            .collect();
        for i in 8..16u64 {
            orig.push(GlonassSample {
                carrier_phase_cycles: None,
                ..constant_sample(i, GloSlot::new(0).unwrap())
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
            ..constant_sample(ts, GloSlot::new(2).unwrap())
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
            constant_sample(0, GloSlot::new(0).unwrap()),
            constant_sample(1, GloSlot::new(0).unwrap()),
            GlonassSample {
                timestamp_ms: BASE_TS + 10_001,
                ..constant_sample(10_001, GloSlot::new(0).unwrap())
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
        let orig: Vec<_> = offsets
            .iter()
            .map(|&t| constant_sample(t, GloSlot::new(1).unwrap()))
            .collect();
        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.timestamp_ms, d.timestamp_ms, "ts[{i}]");
        }
    }

    #[test]
    fn test_roundtrip_large_pseudorange_jump() {
        let mut orig = vec![
            constant_sample(0, GloSlot::new(0).unwrap()),
            constant_sample(1, GloSlot::new(0).unwrap()),
        ];

        orig.push(GlonassSample {
            pseudorange_mm: Millimeter::new(21_500_000_000 + 1_000_000),
            ..constant_sample(2, GloSlot::new(0).unwrap())
        });

        let dec = roundtrip(&orig);

        for (i, (o, d)) in orig.iter().zip(&dec).enumerate() {
            assert_eq!(o.pseudorange_mm, d.pseudorange_mm, "pr[{i}]");
        }
    }

    #[test]
    fn test_decode_into_matches_decode_chunk() {
        let orig: Vec<_> = (0..32)
            .map(|i| sample(i, GloSlot::new(1).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        let expected = GlonassDecoder::decode_chunk(&chunk).unwrap();
        let mut out = vec![GlonassSample::default_zeroed(); 64];
        let n = GlonassDecoder::decode_into(&chunk, &mut out).unwrap();

        assert_eq!(n, expected.len());
        assert_eq!(&out[..n], expected.as_slice());
    }

    #[test]
    fn test_decode_into_single_sample() {
        let orig = [constant_sample(0, GloSlot::new(0).unwrap())];
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        let mut out: [GlonassSample; 1] = core::array::from_fn(|_| GlonassSample::default_zeroed());
        let n = GlonassDecoder::decode_into(&chunk, &mut out).unwrap();

        assert_eq!(n, 1);
        assert_eq!(out[0], orig[0]);
    }

    #[test]
    fn test_decode_into_all_slots() {
        for slot in -7_i8..=6 {
            let orig: Vec<_> = (0..16)
                .map(|i| sample(i, GloSlot::new(slot).unwrap()))
                .collect();
            let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
            let mut out: [GlonassSample; 16] =
                core::array::from_fn(|_| GlonassSample::default_zeroed());
            let n = GlonassDecoder::decode_into(&chunk, &mut out).unwrap();

            assert_eq!(n, 16);
            assert_eq!(&out[..n], orig.as_slice(), "slot {slot}");
        }
    }

    #[test]
    fn test_decode_into_buffer_too_small_returns_buffer_full() {
        let orig: Vec<_> = (0..32)
            .map(|i| sample(i, GloSlot::new(1).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        // Буфер вмещает только 10 сэмплов при 32 в chunk
        let mut out =
            core::array::from_fn::<GlonassSample, 10, _>(|_| GlonassSample::default_zeroed());

        assert!(matches!(
            GlonassDecoder::decode_into(&chunk, &mut out),
            Err(GorkaError::BufferFull)
        ));
    }

    #[test]
    fn test_decode_into_exact_size_buffer() {
        let orig: Vec<_> = (0..16)
            .map(|i| sample(i, GloSlot::new(2).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        let mut out: [GlonassSample; 16] =
            core::array::from_fn(|_| GlonassSample::default_zeroed());
        let n = GlonassDecoder::decode_into(&chunk, &mut out).unwrap();

        assert_eq!(n, 16);
    }

    #[test]
    fn test_decode_into_stack_buffer_no_alloc() {
        // Этот тест демонстрирует использование без Vec
        let orig: Vec<_> = (0..10)
            .map(|i| constant_sample(i, GloSlot::new(0).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        // Стековый массив — никаких Vec
        let mut out: [GlonassSample; 32] =
            core::array::from_fn(|_| GlonassSample::default_zeroed());
        let n = GlonassDecoder::decode_into(&chunk, &mut out).unwrap();

        assert_eq!(n, 10);

        for (i, (got, expected)) in out[..n].iter().zip(&orig).enumerate() {
            assert_eq!(got, expected, "sample[{i}]");
        }
    }

    #[test]
    fn test_iter_chunk_matches_decode_chunk() {
        let orig: Vec<_> = (0..32)
            .map(|i| sample(i, GloSlot::new(1).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        let expected = GlonassDecoder::decode_chunk(&chunk).unwrap();
        let iter_result: Vec<GlonassSample> = GlonassDecoder::iter_chunk(&chunk)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(iter_result, expected);
    }

    #[test]
    fn test_iter_chunk_single_sample() {
        let orig = [constant_sample(0, GloSlot::new(3).unwrap())];
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        let mut iter = GlonassDecoder::iter_chunk(&chunk).unwrap();
        let first = iter.next().unwrap().unwrap();

        assert_eq!(first, orig[0]);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_chunk_size_hint() {
        let orig: Vec<_> = (0..20)
            .map(|i| sample(i, GloSlot::new(1).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        let mut iter = GlonassDecoder::iter_chunk(&chunk).unwrap();

        assert_eq!(iter.size_hint(), (20, Some(20)));

        iter.next().unwrap().unwrap();

        assert_eq!(iter.size_hint(), (19, Some(19)));

        iter.next().unwrap().unwrap();

        assert_eq!(iter.size_hint(), (18, Some(18)));
    }

    #[test]
    fn test_iter_chunk_exact_size() {
        let orig: Vec<_> = (0..10)
            .map(|i| constant_sample(i, GloSlot::new(0).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        let iter = GlonassDecoder::iter_chunk(&chunk).unwrap();

        assert_eq!(iter.len(), 10); // ExactSizeIterator
    }

    #[test]
    fn test_iter_chunk_all_slots() {
        for slot in -7_i8..=6 {
            let orig: Vec<_> = (0..16)
                .map(|i| sample(i, GloSlot::new(slot).unwrap()))
                .collect();
            let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
            let decoded: Vec<_> = GlonassDecoder::iter_chunk(&chunk)
                .unwrap()
                .map(|r| r.unwrap())
                .collect();

            assert_eq!(decoded, orig, "slot {slot}");
        }
    }

    #[test]
    fn test_iter_chunk_no_alloc_style() {
        // Демонстрация без Vec — обрабатываем сэмплы по одному
        let orig: Vec<_> = (0..50)
            .map(|i| constant_sample(i, GloSlot::new(1).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();
        let mut count = 0usize;

        for (i, result) in GlonassDecoder::iter_chunk(&chunk).unwrap().enumerate() {
            let s = result.unwrap();

            assert_eq!(s.timestamp_ms, BASE_TS + i as u64, "ts[{i}]");

            count += 1;
        }

        assert_eq!(count, 50);
    }

    #[test]
    fn test_iter_chunk_carrier_phase_transitions() {
        let mk = |ts, ph| GlonassSample {
            carrier_phase_cycles: ph,
            ..constant_sample(ts, GloSlot::new(0).unwrap())
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
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();

        let decoded: Vec<_> = GlonassDecoder::iter_chunk(&chunk)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(decoded, orig);
    }

    #[test]
    fn test_all_three_apis_are_consistent() {
        let orig: Vec<_> = (0..64)
            .map(|i| sample(i, GloSlot::new((i % 5) as i8 - 2).unwrap()))
            .collect();
        let chunk = GlonassEncoder::encode_chunk(&orig).unwrap();

        // 1. decode_chunk
        let from_chunk = GlonassDecoder::decode_chunk(&chunk).unwrap();

        // 2. decode_into
        let mut buf = vec![GlonassSample::default_zeroed(); 64];
        let n = GlonassDecoder::decode_into(&chunk, &mut buf).unwrap();
        let from_into = &buf[..n];

        // 3. iter_chunk
        let from_iter: Vec<_> = GlonassDecoder::iter_chunk(&chunk)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(from_chunk, orig);
        assert_eq!(from_into, orig.as_slice());
        assert_eq!(from_iter, orig);
    }
}
