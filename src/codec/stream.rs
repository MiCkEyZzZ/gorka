use crate::{
    encode_i64, FormatVersion, GlonassSample, GorkaError, MilliHz, Millimeter, VersionUtils,
};

pub const STREAM_ENCODER_MIN_BUF_NO_PHASE: usize = 9 + 23;
pub const STREAM_ENCODER_MIN_BUF_WITH_PHASE: usize = 9 + 31;

const N_SLOT: usize = 14;

pub struct StreamEncoder<'buf> {
    buf: &'buf mut [u8],
    bitstream_start: usize,
    byte_pos: usize,
    bit_pos: u8,
    sample_count: u32,
    state: Option<EncoderState>,
}

struct EncoderState {
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

struct StateSnapshot {
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

struct RawBitWriter<'a> {
    buf: &'a mut [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl EncoderState {
    pub fn from_first(sample: &GlonassSample) -> Self {
        let mut last_doppler = [None; N_SLOT];

        last_doppler[slot_idx(sample.slot)] = Some(sample.doppler_millihz.0);

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

    fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            last_ts: self.last_ts,
            last_delta_ts: self.last_delta_ts,
            last_slot: self.last_slot,
            last_cn0: self.last_cn0,
            last_pr_mm: self.last_pr_mm,
            last_pr_delta: self.last_pr_delta,
            last_doppler: self.last_doppler,
            last_phase: self.last_phase,
            last_phase_delta: self.last_phase_delta,
        }
    }

    fn restore(
        &mut self,
        snap: StateSnapshot,
    ) {
        self.last_ts = snap.last_ts;
        self.last_delta_ts = snap.last_delta_ts;
        self.last_slot = snap.last_slot;
        self.last_cn0 = snap.last_cn0;
        self.last_pr_mm = snap.last_pr_mm;
        self.last_pr_delta = snap.last_pr_delta;
        self.last_doppler = snap.last_doppler;
        self.last_phase = snap.last_phase;
        self.last_phase_delta = snap.last_phase_delta;
    }
}

impl<'buf> StreamEncoder<'buf> {
    pub fn new(buf: &'buf mut [u8]) -> Self {
        Self {
            buf,
            bitstream_start: 0,
            byte_pos: 0,
            bit_pos: 0,
            sample_count: 0,
            state: None,
        }
    }

    pub fn push_sample(
        &mut self,
        sample: &GlonassSample,
    ) -> Result<usize, GorkaError> {
        let before = self.bytes_written();
        sample.validate_slot()?;

        if self.state.is_none() {
            self.push_first(sample)?;
        } else {
            self.push_delta(sample)?;
        }

        self.sample_count += 1;

        Ok(self.bytes_written() - before)
    }

    pub fn flush(
        &mut self,
        out: &mut [u8],
    ) -> Result<usize, GorkaError> {
        if self.sample_count == 0 {
            return Err(GorkaError::EmptyChunk);
        }

        let total_bytes = self.bytes_written();

        if out.len() < total_bytes {
            return Err(GorkaError::BufferFull);
        }

        let header = VersionUtils::write_chunk_header(FormatVersion::current(), self.sample_count);

        out[..9].copy_from_slice(&header[..9]);

        let data_len = total_bytes - 9;

        out[9..9 + data_len].copy_from_slice(&self.buf[9..9 + data_len]);

        Ok(total_bytes)
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn bytes_written(&self) -> usize {
        if self.bit_pos > 0 {
            self.byte_pos + 1
        } else {
            self.byte_pos
        }
    }

    fn push_first(
        &mut self,
        sample: &GlonassSample,
    ) -> Result<(), GorkaError> {
        let vlen = verbatim_size(sample);

        // исправлено: допускаем ровно минимальный размер
        if self.buf.len() < 9 + vlen {
            return Err(GorkaError::BufferFull);
        }

        write_verbatim(sample, &mut self.buf[9..9 + vlen]);

        self.bitstream_start = 9 + vlen;
        self.byte_pos = self.bitstream_start;
        self.bit_pos = 0;

        self.state = Some(EncoderState::from_first(sample));

        Ok(())
    }

    fn push_delta(
        &mut self,
        sample: &GlonassSample,
    ) -> Result<(), GorkaError> {
        let state = self.state.as_mut().unwrap();

        // Снимок для отката
        let snap = state.snapshot();
        let save_byte_pos = self.byte_pos;
        let save_bit_pos = self.bit_pos;

        let result = {
            let mut writer = RawBitWriter {
                buf: self.buf,
                byte_pos: self.byte_pos,
                bit_pos: self.bit_pos,
            };

            let reader = encode_delta_fields(&mut writer, state, sample);

            if reader.is_ok() {
                self.byte_pos = writer.byte_pos;
                self.bit_pos = writer.bit_pos;
            }

            reader
        };

        if result.is_err() {
            // Атомарный откат
            self.state.as_mut().unwrap().restore(snap);
            self.byte_pos = save_byte_pos;
            self.bit_pos = save_bit_pos;

            // Зануляем затронутые байты
            let clean_from = save_byte_pos;

            if clean_from < self.buf.len() {
                self.buf[clean_from..].fill(0);
            }
        }

        result
    }
}

impl<'a> RawBitWriter<'a> {
    #[inline(always)]
    fn write_bit(
        &mut self,
        bit: bool,
    ) -> Result<(), GorkaError> {
        if self.byte_pos >= self.buf.len() {
            return Err(GorkaError::BufferFull);
        }

        if bit {
            self.buf[self.byte_pos] |= 1 << (7 - self.bit_pos);
        }

        self.bit_pos += 1;

        if self.bit_pos == 8 {
            self.byte_pos += 1;
            self.bit_pos = 0;

            if self.byte_pos < self.buf.len() {
                self.buf[self.byte_pos] = 0;
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn write_bits(
        &mut self,
        value: u64,
        n: u8,
    ) -> Result<(), GorkaError> {
        if n > 64 {
            return Err(GorkaError::InvalidBitCount(n));
        }

        if n < 64 && value >= (1u64 << n) {
            return Err(GorkaError::ValueTooLarge { value, bits: n });
        }

        let available = self.buf.len().saturating_sub(self.byte_pos) * 8 - self.bit_pos as usize;

        if (n as usize) > available {
            return Err(GorkaError::BufferFull);
        }

        for i in (0..n).rev() {
            self.write_bit((value >> i) & 1 == 1)?;
        }

        Ok(())
    }

    fn write_bits_signed(
        &mut self,
        value: i64,
        n: u8,
    ) -> Result<(), GorkaError> {
        self.write_bits(encode_i64(value), n)
    }
}

#[inline]
fn slot_idx(slot: i8) -> usize {
    (slot + 7) as usize
}

fn verbatim_size(sample: &GlonassSample) -> usize {
    if sample.carrier_phase_cycles.is_none() {
        23 // без фазы
    } else {
        31 // с фазой
    }
}

fn write_verbatim(
    sample: &GlonassSample,
    dst: &mut [u8],
) {
    dst[0..8].copy_from_slice(&sample.timestamp_ms.to_le_bytes());
    dst[8] = sample.slot as u8;
    dst[9] = sample.cn0_dbhz;
    dst[10..18].copy_from_slice(&sample.pseudorange_mm.0.to_le_bytes());
    dst[18..22].copy_from_slice(&sample.doppler_millihz.0.to_le_bytes());

    match sample.carrier_phase_cycles {
        None => dst[22] = 0,
        Some(p) => {
            dst[22] = 1;
            dst[23..31].copy_from_slice(&p.to_le_bytes());
        }
    }
}

fn encode_delta_fields(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    sample: &GlonassSample,
) -> Result<(), GorkaError> {
    enc_ts(writer, state, sample.timestamp_ms)?;
    enc_slot(writer, state, sample.slot)?;
    enc_cn0(writer, state, sample.cn0_dbhz)?;
    enc_pr(writer, state, sample.pseudorange_mm)?;
    enc_dop(writer, state, sample.doppler_millihz, sample.slot)?;
    enc_phase(writer, state, sample.carrier_phase_cycles)?;

    Ok(())
}

fn enc_ts(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    timestamp: u64,
) -> Result<(), GorkaError> {
    let delta = timestamp.wrapping_sub(state.last_ts);
    let dod = delta as i64 - state.last_delta_ts as i64;
    let zz = encode_i64(dod);

    if dod == 0 {
        writer.write_bit(false)?;
    } else if zz < (1u64 << 7) {
        writer.write_bits(0b10, 2)?;
        writer.write_bits_signed(dod, 7)?;
    } else if zz < (1u64 << 9) {
        writer.write_bits(0b110, 3)?;
        writer.write_bits_signed(dod, 9)?;
    } else {
        writer.write_bits(0b111, 3)?;
        writer.write_bits(timestamp, 64)?;
    }

    state.last_delta_ts = delta;
    state.last_ts = timestamp;

    Ok(())
}

fn enc_slot(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    slot: i8,
) -> Result<(), GorkaError> {
    if slot == state.last_slot {
        writer.write_bit(false)?;
    } else {
        writer.write_bit(true)?;
        writer.write_bits(slot_idx(slot) as u64, 4)?;
    }

    state.last_slot = slot;

    Ok(())
}

fn enc_cn0(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    cn0: u8,
) -> Result<(), GorkaError> {
    let delta = cn0 as i16 - state.last_cn0 as i16;

    if delta == 0 {
        writer.write_bit(false)?;
    } else {
        writer.write_bit(true)?;
        writer.write_bits_signed(delta as i64, 9)?;
    }

    state.last_cn0 = cn0;

    Ok(())
}

fn enc_pr(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    pr: Millimeter,
) -> Result<(), GorkaError> {
    let delta = pr.0 - state.last_pr_mm.0;
    let dod = delta - state.last_pr_delta.0;
    let zz = encode_i64(dod);

    if dod == 0 {
        writer.write_bit(false)?;
    } else if zz < (1u64 << 10) {
        writer.write_bits(0b10, 2)?;
        writer.write_bits_signed(dod, 10)?;
    } else if zz < (1u64 << 20) {
        writer.write_bits(0b110, 3)?;
        writer.write_bits_signed(dod, 20)?;
    } else {
        writer.write_bits(0b111, 3)?;
        writer.write_bits(pr.0 as u64, 64)?;
    }

    state.last_pr_delta.0 = delta;
    state.last_pr_mm = pr;

    Ok(())
}

fn enc_dop(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    doppler: MilliHz,
    slot: i8,
) -> Result<(), GorkaError> {
    let idx = slot_idx(slot);

    match state.last_doppler[idx] {
        None => {
            writer.write_bit(false)?;
            writer.write_bits(doppler.0 as u64 & 0xFFFF_FFFF, 32)?;
        }
        Some(prev) => {
            let delta = doppler.0 as i64 - prev as i64;
            let zz = encode_i64(delta);

            if delta == 0 {
                writer.write_bits(0b10, 2)?;
            } else if zz < (1u64 << 14) {
                writer.write_bits(0b110, 3)?;
                writer.write_bits_signed(delta, 14)?;
            } else {
                writer.write_bits(0b111, 3)?;
                writer.write_bits(doppler.0 as u64 & 0xFFFF_FFFF, 32)?;
            }
        }
    }

    state.last_doppler[idx] = Some(doppler.0);

    Ok(())
}

fn enc_phase(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    phase: Option<i64>,
) -> Result<(), GorkaError> {
    match (state.last_phase, phase) {
        (None, None) => {
            writer.write_bits(0b00, 2)?;
        }
        (Some(_), None) => {
            writer.write_bits(0b01, 2)?;
        }
        (None, Some(p)) => {
            writer.write_bits(0b10, 2)?;
            writer.write_bits(p as u64, 64)?;
        }
        (Some(prev), Some(cur)) => {
            let delta = cur - prev;
            let prev_d = state.last_phase_delta.unwrap_or(0);
            let dod = delta - prev_d;
            let zz = encode_i64(dod);

            writer.write_bits(0b11, 2)?;

            if dod == 0 {
                writer.write_bit(false)?;
                state.last_phase_delta = Some(delta);
            } else if zz < (1u64 << 32) {
                writer.write_bits(0b10, 2)?;
                writer.write_bits_signed(dod, 32)?;
                state.last_phase_delta = Some(delta);
            } else {
                writer.write_bits(0b11, 2)?;
                writer.write_bits(cur as u64, 64)?;
            }
        }
    }

    state.last_phase = phase;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{GlonassDecoder, GlonassEncoder};

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

    fn constant(
        i: u64,
        slot: i8,
    ) -> GlonassSample {
        GlonassSample {
            timestamp_ms: BASE_TS + i,
            slot,
            cn0_dbhz: 42,
            pseudorange_mm: Millimeter::new(21_500_000_000),
            doppler_millihz: MilliHz::new(1_200_500),
            carrier_phase_cycles: None,
        }
    }

    fn roundtrip_stream(samples: &[GlonassSample]) -> Vec<GlonassSample> {
        let mut buf = vec![0u8; 65536];
        let mut enc = StreamEncoder::new(&mut buf);

        for s in samples {
            enc.push_sample(s).expect("push_sample");
        }

        let mut out = vec![0u8; 65536];
        let n = enc.flush(&mut out).expect("flush");

        GlonassDecoder::decode_chunk(&out[..n]).expect("decode")
    }

    #[test]
    fn test_single_no_phase() {
        let orig = [constant(0, 0)];

        assert_eq!(roundtrip_stream(&orig), orig);
    }

    #[test]
    fn test_single_with_phase() {
        let orig = [sample(0, 1)];

        assert_eq!(roundtrip_stream(&orig), orig);
    }

    #[test]
    fn test_10_samples() {
        let orig: Vec<_> = (0..10).map(|i| sample(i, 1)).collect();

        assert_eq!(roundtrip_stream(&orig), orig);
    }

    #[test]
    fn test_128_smooth() {
        let orig: Vec<_> = (0..128).map(|i| sample(i, 2)).collect();

        assert_eq!(roundtrip_stream(&orig), orig);
    }

    #[test]
    fn test_all_14_slots() {
        let orig: Vec<_> = (0..56u64).map(|i| sample(i, (i % 14) as i8 - 7)).collect();

        assert_eq!(roundtrip_stream(&orig), orig);
    }

    #[test]
    fn test_carrier_phase_reacquired() {
        let mk = |ts: u64, ph: Option<i64>| GlonassSample {
            carrier_phase_cycles: ph,
            ..constant(ts, 2)
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

        assert_eq!(roundtrip_stream(&orig), orig);
    }

    #[test]
    fn test_large_timestamp_gap() {
        let orig = vec![
            constant(0, 0),
            constant(1, 0),
            GlonassSample {
                timestamp_ms: BASE_TS + 10_001,
                ..constant(10_001, 0)
            },
        ];

        assert_eq!(roundtrip_stream(&orig), orig);
    }

    #[test]
    fn test_output_identical_to_batch_encoder() {
        let orig: Vec<_> = (0..32).map(|i| sample(i, 1)).collect();
        let batch = GlonassEncoder::encode_chunk(&orig).unwrap();

        let mut buf = vec![0u8; 65536];
        let mut enc = StreamEncoder::new(&mut buf);

        for s in &orig {
            enc.push_sample(s).unwrap();
        }

        let mut out = vec![0u8; 65536];
        let n = enc.flush(&mut out).unwrap();

        assert_eq!(&out[..n], batch.as_slice());
    }

    #[test]
    fn test_flush_empty_returns_error() {
        let mut buf = [0u8; 64];
        let mut out = [0u8; 64];

        let mut enc = StreamEncoder::new(&mut buf);

        assert!(matches!(enc.flush(&mut out), Err(GorkaError::EmptyChunk)));
    }

    #[test]
    fn test_buffer_too_small_no_phase() {
        let mut buf = [0u8; STREAM_ENCODER_MIN_BUF_NO_PHASE - 1];
        let mut enc = StreamEncoder::new(&mut buf);

        assert!(matches!(
            enc.push_sample(&constant(0, 0)),
            Err(GorkaError::BufferFull)
        ));
    }

    #[test]
    fn test_buffer_too_small_with_phase() {
        let mut buf = [0u8; STREAM_ENCODER_MIN_BUF_WITH_PHASE - 1];
        let mut enc = StreamEncoder::new(&mut buf);

        assert!(matches!(
            enc.push_sample(&sample(0, 1)),
            Err(GorkaError::BufferFull)
        ));
    }

    #[test]
    fn test_invalid_slot_rejected() {
        let mut buf = [0u8; 64];
        let mut enc = StreamEncoder::new(&mut buf);
        let bad = GlonassSample {
            slot: 99,
            ..constant(0, 0)
        };

        assert!(matches!(
            enc.push_sample(&bad),
            Err(GorkaError::InvalidSlot(99))
        ));
        assert_eq!(enc.sample_count(), 0);
    }

    #[test]
    fn test_rollback_preserves_previous_samples() {
        let mut buf = [0u8; 64];
        let mut enc = StreamEncoder::new(&mut buf);

        // Первый сэмпл (40B verbatim) влезает
        enc.push_sample(&sample(0, 1)).unwrap();

        assert_eq!(enc.sample_count(), 1);

        let bytes_after_first = enc.bytes_written();

        let mut pushed = 1u32;

        for i in 1..100u64 {
            match enc.push_sample(&sample(i, 1)) {
                Ok(_) => pushed += 1,
                Err(GorkaError::BufferFull) => break,
                Err(e) => panic!("{e:?}"),
            }
        }

        assert!(enc.bytes_written() >= bytes_after_first);
        assert_eq!(enc.sample_count(), pushed);

        let mut out = [0u8; 64];
        let n = enc.flush(&mut out).unwrap();

        let dec = GlonassDecoder::decode_chunk(&out[..n]).unwrap();

        assert_eq!(dec.len() as u32, pushed);

        assert_eq!(dec[0], sample(0, 1));
    }

    #[test]
    fn test_minimum_buffer_no_phase() {
        let mut buf = [0u8; STREAM_ENCODER_MIN_BUF_NO_PHASE];
        let mut enc = StreamEncoder::new(&mut buf);

        enc.push_sample(&constant(0, 0)).unwrap();
        assert_eq!(enc.sample_count(), 1);
    }

    #[test]
    fn test_minimum_buffer_with_phase() {
        let mut buf = [0u8; STREAM_ENCODER_MIN_BUF_WITH_PHASE];
        let mut enc = StreamEncoder::new(&mut buf);

        enc.push_sample(&sample(0, 1)).unwrap();
        assert_eq!(enc.sample_count(), 1);
    }

    #[test]
    fn test_sample_count_tracking() {
        let mut buf = [0u8; 1024];
        let mut enc = StreamEncoder::new(&mut buf);

        for i in 0..5u64 {
            enc.push_sample(&constant(i, 0)).unwrap();
            assert_eq!(enc.sample_count(), i as u32 + 1);
        }
    }
}
