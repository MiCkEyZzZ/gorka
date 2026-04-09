//! GLONASS encoder (черновая реализация)
//!
//! ⚠️ Это базовая реализация, предназначенная для проверки модели сжатия и
//! структуры битового представления.
//!
//! Известные ограничения:
//! - Пока отсутствует энтропийное кодирование (Huffman/ANS)
//! - Нет обобщения на другие созвездия GNSS
//! - Нет потокового / инкрементального API
//! - Минимальная обработка ошибок
//!
//! TODO:
//! - Выделить общее ядро кодировщика GNSS
//! - Добавить декодер + тесты roundtrip
//! - Провести бенчммарки на реальных GNSS-логах
//! - Оптимизировать упаковку битов (SIMD / без ветвлений)
//!
//! ПРИМЕЧАНИЕ:
//! В этом модуле приоритет отдан читаемости, а не максимальной
//! производительности.

use alloc::{vec, vec::Vec};

use crate::{
    encode_i64, BitWrite, DbHz, FormatVersion, GloSlot, GlonassSample, GorkaError, MilliHz,
    Millimeter, RawBitWriter, VersionUtils,
};

// Количество частотных слотов ГЛОНАСС: k ∈ [−7, +6].
const N_SLOT: usize = 14;

/// Изменяемое состояние, сохраняемое для каждого фрагмента, передается между
/// сэмплами.
struct EncoderState {
    // timestamp
    last_ts: u64,
    last_delta_ts: u64,
    // slot
    last_slot: GloSlot,
    // C/N0
    last_cn0: DbHz,
    // pseudorange (mm)
    last_pr_mm: Millimeter,
    last_pr_delta: Millimeter,
    // Доплеровский сдвиг (мГц) — одна запись на каждый слот ГЛОНАСС, отсутствует до первого
    // наблюдения index = slot + 7 (slot ∈ -7..+6 → index 0..13)
    last_doppler: [Option<i32>; N_SLOT],
    // carrier phase
    last_phase: Option<i64>,
    last_phase_delta: Option<i64>, // для DoD None до первой пары
}

pub struct GlonassEncoder;

impl EncoderState {
    fn from_first(sample: &GlonassSample) -> Self {
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
}

#[allow(deprecated)]
impl GlonassEncoder {
    // Кодирует фрагмент `GlonassSample` в сжатый байтовый блок.
    pub fn encode_chunk(samples: &[GlonassSample]) -> Result<Vec<u8>, GorkaError> {
        if samples.is_empty() {
            return Err(GorkaError::EmptyChunk);
        }

        // Проверяем слоты заранее — быстро выявляя ошибки, прежде вызывать writer
        for sample in samples {
            sample.validate_slot()?;
        }

        let first = &samples[0];
        let count = samples.len() as u32;

        // 9-байтовый заголовок
        let header = VersionUtils::write_chunk_header(FormatVersion::current(), count);

        // Предварительное выделение памяти: заголовок + дословный текст (≤ 31 Б) + ~4 Б
        // на каждый оставшийся фрагмент.
        let capasity = 9 + 31 + (samples.len().saturating_sub(1)) * 4;
        let mut out: Vec<u8> = Vec::with_capacity(capasity);

        out.extend_from_slice(&header);

        // Verbatim первый сэмпл
        encode_verbatim(first, &mut out);

        if samples.len() == 1 {
            return Ok(out);
        }

        let delta_count = samples.len() - 1;
        let mut tmp = vec![0u8; delta_count * 32];
        let mut writer = RawBitWriter::new(&mut tmp);
        let mut state = EncoderState::from_first(first);

        for sample in &samples[1..] {
            encode_delta(&mut writer, &mut state, sample)?;
        }

        let written = writer.bytes_written();
        out.extend_from_slice(&tmp[..written]);

        Ok(out)
    }
}

fn encode_verbatim(
    sample: &GlonassSample,
    out: &mut Vec<u8>,
) {
    out.extend_from_slice(&sample.timestamp_ms.to_le_bytes());
    out.push(sample.slot.get() as u8);
    out.push(sample.cn0_dbhz.get());
    out.extend_from_slice(&sample.pseudorange_mm.0.to_le_bytes());
    out.extend_from_slice(&sample.doppler_millihz.0.to_le_bytes());

    match sample.carrier_phase_cycles {
        None => out.push(0),
        Some(p) => {
            out.push(1);
            out.extend_from_slice(&p.to_le_bytes());
        }
    }
}

#[allow(deprecated)]
fn encode_delta(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    sample: &GlonassSample,
) -> Result<(), GorkaError> {
    encode_timestamp(writer, state, sample.timestamp_ms)?;
    encode_slot(writer, state, sample.slot)?;
    encode_cn0(writer, state, sample.cn0_dbhz)?;
    encode_pseudorange(writer, state, sample.pseudorange_mm)?;
    encode_doppler(writer, state, sample.doppler_millihz, sample.slot)?;
    encode_carrier_phase(writer, state, sample.carrier_phase_cycles)?;

    Ok(())
}

#[allow(deprecated)]
// Timestamp: дельта-из-дельты по 4-компонентной схеме.
fn encode_timestamp(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    timestamp: u64,
) -> Result<(), GorkaError> {
    let delta = timestamp.wrapping_sub(state.last_ts);
    let dod = delta as i64 - state.last_delta_ts as i64;
    let zz = encode_i64(dod);

    if dod == 0 {
        writer.write_bit(false)?; // '0'
    } else if zz < (1u64 << 7) {
        writer.write_bits(0b10, 2)?; // '10' + 7b
        writer.write_bits_signed(dod, 7)?;
    } else if zz < (1u64 << 9) {
        writer.write_bits(0b110, 3)?; // '110' + 9b
        writer.write_bits_signed(dod, 9)?;
    } else {
        writer.write_bits(0b111, 3)?; // '111' + 64b verbatim
        writer.write_bits(timestamp, 64)?;
    }

    state.last_delta_ts = delta;
    state.last_ts = timestamp;

    Ok(())
}

#[allow(deprecated)]
// Slot: 1-битный флаг, новое значение в 4 битах, если изменено.
fn encode_slot(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    slot: GloSlot,
) -> Result<(), GorkaError> {
    if slot == state.last_slot {
        writer.write_bit(false)?; // '0' same
    } else {
        writer.write_bit(true)?; // '1' + 4b index
        writer.write_bits(slot_idx(slot) as u64, 4)?;
    }

    state.last_slot = slot;

    Ok(())
}

#[allow(deprecated)]
// C/N0: простой дельта-зигзаг.
// Максимальное значение дельты для поля u8 составляет ±255; зигзаг(255) = 510
// < 512 = 2^9.
fn encode_cn0(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    cn0: DbHz,
) -> Result<(), GorkaError> {
    let delta = cn0.get() as i16 - state.last_cn0.get() as i16; // range -255..=255

    if delta == 0 {
        writer.write_bit(false)?; // '0'
    } else {
        writer.write_bit(true)?; // '1' + 9b zigzag
        writer.write_bits_signed(delta as i64, 9)?;
    }

    state.last_cn0 = cn0;

    Ok(())
}

#[allow(deprecated)]
// Pseudorange: разница дельта-дельта в миллиметрах, схема с 4 интервалами.
fn encode_pseudorange(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    pr_mm: Millimeter,
) -> Result<(), GorkaError> {
    let delta = pr_mm.0 - state.last_pr_mm.0;
    let dod = delta - state.last_pr_delta.0;
    let zz = encode_i64(dod);

    if dod == 0 {
        writer.write_bit(false)?; // '0'
    } else if zz < (1u64 << 10) {
        writer.write_bits(0b10, 2)?; // '10' + 10b
        writer.write_bits_signed(dod, 10)?;
    } else if zz < (1u64 << 20) {
        writer.write_bits(0b110, 3)?; // '110' + 20b
        writer.write_bits_signed(dod, 20)?;
    } else {
        writer.write_bits(0b111, 3)?; // '111' + 64b verbatim
        writer.write_bits(pr_mm.0 as u64, 64)?;
    }

    state.last_pr_delta.0 = delta;
    state.last_pr_mm = pr_mm;

    Ok(())
}

#[allow(deprecated)]
// Доплеровский эффект: дельта-эффект для каждого слота с коррекцией FDMA.
// Каждый k-slot ГЛОНАСС отслеживает свой последний доплеровский сдвиг
// независимо, так что чередующиеся многоспутниковые фрагменты не создают
// больших межслотовых дельт.
fn encode_doppler(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    doppler: MilliHz,
    slot: GloSlot,
) -> Result<(), GorkaError> {
    let idx = slot_idx(slot);

    match state.last_doppler[idx] {
        // Первое наблюдение для этого слота в этом блоке — запись дословно.
        // декодер также отслеживает состояние slot-first и считывает дословно здесь.
        None => {
            writer.write_bit(false)?; // '0' flag: verbatim
            writer.write_bits(doppler.0 as u64 & 0xFFFF_FFFF, 32)?;
        }
        Some(prev) => {
            let delta = doppler.0 as i64 - prev as i64;
            let zz = encode_i64(delta);

            if delta == 0 {
                writer.write_bits(0b10, 2)?;
            } else if zz < (1u64 << 14) {
                writer.write_bits(0b110, 3)?; // '110' + 14b
                writer.write_bits_signed(delta, 14)?;
            } else {
                writer.write_bits(0b111, 3)?; // '111' + 32b verbatim
                writer.write_bits(doppler.0 as u64 & 0xFFFF_FFFF, 32)?;
            }
        }
    }

    state.last_doppler[idx] = Some(doppler.0);

    Ok(())
}

#[allow(deprecated)]
// Carrier phase: необязательная, дельта-дельта.
fn encode_carrier_phase(
    writer: &mut RawBitWriter,
    state: &mut EncoderState,
    phase: Option<i64>,
) -> Result<(), GorkaError> {
    match (state.last_phase, phase) {
        (None, None) => {
            writer.write_bits(0b00, 2)?; // '00' None -> None
        }
        (Some(_), None) => {
            writer.write_bits(0b01, 2)?; // '01' Some -> None (phase lost)
        }
        (None, Some(p)) => {
            writer.write_bits(0b10, 2)?; // '10' + 64b verbatim
            writer.write_bits(p as u64, 64)?;
        }
        (Some(prev), Some(curr)) => {
            let delta = curr - prev;
            let prev_d = state.last_phase_delta.unwrap_or(0);
            let dod = delta - prev_d;
            let zz = encode_i64(dod);

            writer.write_bits(0b11, 2)?; // prev '11', then branch

            if dod == 0 {
                writer.write_bit(false)?; // '110' dod == 0
                state.last_phase_delta = Some(delta);
            } else if zz < (1u64 << 32) {
                writer.write_bits(0b10, 2)?; // '1110' + 32b
                writer.write_bits_signed(dod, 32)?;
                state.last_phase_delta = Some(delta);
            } else {
                writer.write_bits(0b11, 2)?; // '1111' + 64b verbatim (reset DoD)
                writer.write_bits(curr as u64, 64)?;
                state.last_phase_delta = None; // reset: next delta is "first"
            }
        }
    }

    state.last_phase = phase;

    Ok(())
}

#[inline]
fn slot_idx(slot: GloSlot) -> usize {
    let s = slot.get(); // получаем i8 из GloSlot
    debug_assert!((-7..=6).contains(&s));

    (s + 7) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DbHz, GloSlot, CHUNK_MAGIC};

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

    fn read_magic(buf: &[u8]) -> u32 {
        u32::from_le_bytes(buf[0..4].try_into().unwrap())
    }

    fn read_version(buf: &[u8]) -> u8 {
        buf[4]
    }

    fn read_count(buf: &[u8]) -> u32 {
        u32::from_le_bytes(buf[5..9].try_into().unwrap())
    }

    #[test]
    fn test_empty_chunk_returns_error() {
        let err = GlonassEncoder::encode_chunk(&[]).unwrap_err();

        assert!(matches!(err, GorkaError::EmptyChunk));
    }

    #[test]
    fn header_magic_is_correct() {
        let buf = GlonassEncoder::encode_chunk(&[sample(0, GloSlot::new(1).unwrap())]).unwrap();
        assert_eq!(read_magic(&buf), CHUNK_MAGIC);
    }

    #[test]
    fn header_version_is_v1() {
        let buf = GlonassEncoder::encode_chunk(&[sample(0, GloSlot::new(1).unwrap())]).unwrap();
        assert_eq!(read_version(&buf), 1);
    }

    #[test]
    fn test_header_count_single_sample() {
        let buf = GlonassEncoder::encode_chunk(&[sample(0, GloSlot::new(1).unwrap())]).unwrap();

        assert_eq!(read_count(&buf), 1);
    }

    #[test]
    fn test_single_sample_no_phase_size() {
        // Header(9) + verbatim(8+1+1+8+4+1) = 9 + 23 = 32 bytes
        let s = constant_sample(0, GloSlot::new(0).unwrap());
        let buf = GlonassEncoder::encode_chunk(&[s]).unwrap();

        assert_eq!(buf.len(), 9 + 23, "unexpected size: {}", buf.len());
    }

    #[test]
    fn test_single_sample_with_phase_size() {
        // Header(9) + verbatim(8+1+1+8+4+1+8) = 9 + 31 = 40 bytes
        let s = sample(0, GloSlot::new(1).unwrap());
        let buf = GlonassEncoder::encode_chunk(&[s]).unwrap();

        assert_eq!(buf.len(), 9 + 31, "unexpected size: {}", buf.len());
    }

    #[test]
    fn test_verbatim_timestamp_round_trips_in_header() {
        let s = constant_sample(999, GloSlot::new(3).unwrap());
        let buf = GlonassEncoder::encode_chunk(&[s]).unwrap();
        let ts = u64::from_le_bytes(buf[9..17].try_into().unwrap());

        assert_eq!(ts, BASE_TS + 999);
    }

    #[test]
    fn test_verbatim_slot_encodes_correctly() {
        for slot in -7_i8..=6 {
            let s = constant_sample(0, GloSlot::new(slot).unwrap());
            let buf = GlonassEncoder::encode_chunk(&[s]).unwrap();
            // slot is at offset 17 (after 9-byte header + 8-byte timestamp)
            let encode_slot = buf[17] as i8;

            assert_eq!(encode_slot, slot, "slot {slot} mismatch");
        }
    }

    #[test]
    fn test_constant_signal_compresses_well() {
        // Все поля постоянные, кроме метки времени (+1 мс на каждом шаге). Ожидается:
        // значение DoD метки времени = 0 → 1 бит, все остальные поля → по 1 биту
        // каждое. Накладные расходы на выборку ≈ 5–6 бит → очень компактно.
        let samples: Vec<_> = (0..256)
            .map(|i| constant_sample(i, GloSlot::new(1).unwrap()))
            .collect();

        let buf = GlonassEncoder::encode_chunk(&samples).unwrap();
        let raw_size = samples.len() * (8 + 1 + 1 + 8 + 4 + 1); // 23 B per sample
        let ratio = raw_size as f64 / buf.len() as f64;

        println!(
            "constant: raw={raw_size}B  compressed={}B  ratio={ratio:.2}×",
            buf.len()
        );

        assert!(
            ratio >= 8.0,
            "constant signal must compress ≥8× (got {ratio:.2}×)"
        );
    }

    #[test]
    fn test_smooth_signal_compresses_well() {
        let samples: Vec<_> = (0..512)
            .map(|i| sample(i, GloSlot::new(1).unwrap()))
            .collect();
        let buf = GlonassEncoder::encode_chunk(&samples).unwrap();
        let raw_size = samples.len() * (8 + 1 + 1 + 8 + 4 + 1 + 8); // 31 B with phase
        let ratio = raw_size as f64 / buf.len() as f64;

        println!(
            "smooth: raw={raw_size}B  compressed={}B  ratio={ratio:.2}×",
            buf.len()
        );

        assert!(
            ratio >= 3.0,
            "smooth signal must compress ≥3× (got {ratio:.2}×)"
        );
    }

    #[test]
    fn test_more_samples_give_better_ratio_than_less() {
        let small: Vec<_> = (0..8)
            .map(|i| sample(i, GloSlot::new(2).unwrap()))
            .collect();
        let large: Vec<_> = (0..128)
            .map(|i| sample(i, GloSlot::new(2).unwrap()))
            .collect();

        let small_buf = GlonassEncoder::encode_chunk(&small).unwrap();
        let large_buf = GlonassEncoder::encode_chunk(&large).unwrap();

        let small_ratio = small.len() as f64 / small_buf.len() as f64;
        let large_ratio = large.len() as f64 / large_buf.len() as f64;

        // Больше образцов -> лучшая амортизация первого verbatim образца
        assert!(
            large_ratio > small_ratio,
            "large ratio {large_ratio:.2}× should exceed small ratio {small_ratio:.2}×"
        );
    }

    #[test]
    fn test_all_valid_slots_encode_without_error() {
        for slot in -7_i8..=6 {
            let samples: Vec<_> = (0..16)
                .map(|i| sample(i, GloSlot::new(slot).unwrap()))
                .collect();

            GlonassEncoder::encode_chunk(&samples).expect("slot {slot} encode failed");
        }
    }

    #[test]
    fn test_multi_slot_chunk_encodes_without_error() {
        // Чередование двух спутников (слоты 1 и -3)
        let mut samples = Vec::new();

        for i in 0..32u64 {
            samples.push(sample(i * 2, GloSlot::new(1).unwrap()));
            samples.push(sample(i * 2 + 1, GloSlot::new(-3).unwrap()));
        }

        // Все используют одинаковую метку времени — просто проверяем, не вызовет ли это
        // панику.
        GlonassEncoder::encode_chunk(&samples).expect("multi-slot encode failed");
    }

    #[test]
    fn test_no_carrier_phase_throughout() {
        let samples: Vec<_> = (0..32)
            .map(|i| constant_sample(i, GloSlot::new(0).unwrap()))
            .collect();

        GlonassEncoder::encode_chunk(&samples).expect("no-phase encode failed");
    }

    #[test]
    fn test_carrier_phase_acquired_mid_stream() {
        let mut samples: Vec<_> = (0..8)
            .map(|i| constant_sample(i, GloSlot::new(0).unwrap()))
            .collect();

        // Вводим фазу, начиная с сэмпла 8.
        for i in 8..16u64 {
            samples.push(GlonassSample {
                timestamp_ms: BASE_TS + i,
                carrier_phase_cycles: Some(i as i64 * (1 << 16)),
                ..constant_sample(i, GloSlot::new(0).unwrap())
            });
        }

        GlonassEncoder::encode_chunk(&samples).expect("phase-acquired encode failed");
    }

    #[test]
    fn test_carrier_phase_lost_mid_stream() {
        let mut samples: Vec<_> = (0..8)
            .map(|i| sample(i, GloSlot::new(0).unwrap()))
            .collect();

        for i in 8..16u64 {
            samples.push(GlonassSample {
                carrier_phase_cycles: None,
                ..constant_sample(i, GloSlot::new(0).unwrap())
            });
        }

        GlonassEncoder::encode_chunk(&samples).expect("phase-lost encode failed");
    }

    #[test]
    fn test_timestamp_dod_zero_emits_one_bit() {
        // При равномерном шаге в 1 мс глубина различимости всегда равна 0 после первой
        // пары. Отсчеты 0, 1, 2 -> δ₁=1, δ₂=1 -> глубина различимости = 0.
        // Проверяем, уменьшается ли выходной сигнал при использовании двух отсчетов.
        let two = (0..2)
            .map(|i| constant_sample(i, GloSlot::new(0).unwrap()))
            .collect::<Vec<_>>();
        let three = (0..3)
            .map(|i| constant_sample(i, GloSlot::new(0).unwrap()))
            .collect::<Vec<_>>();

        let two_len = GlonassEncoder::encode_chunk(&two).unwrap().len();
        let three_len = GlonassEncoder::encode_chunk(&three).unwrap().len();

        // Добавление третьего сэмпла (DoD=0 для всех полей) должно добавить ≤ 1 байт
        // (все поля выдают биты «0» -> возможно, один дополнительный байт для частично
        // заполненного байта).
        assert!(
            three_len <= two_len + 1,
            "adding DoD=0 sample grew by {} bytes (expected ≤1)",
            three_len - two_len
        );
    }

    #[test]
    fn test_large_timestamp_gap_uses_verbatim_bucket() {
        // Задержка в 10 000 мс -> DoD огромен -> дословный 64-битный сегмент
        let s0 = constant_sample(0, GloSlot::new(0).unwrap());
        let s1 = constant_sample(1, GloSlot::new(0).unwrap());
        let s2 = GlonassSample {
            timestamp_ms: BASE_TS + 10_001,
            ..constant_sample(10_001, GloSlot::new(0).unwrap())
        };

        let buf = GlonassEncoder::encode_chunk(&[s0, s1, s2]).unwrap();

        // Должно пройти без ошибок — в блоке verbatim обрабатывается любой пробел.
        assert!(buf.len() > 9);
    }
}
