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
