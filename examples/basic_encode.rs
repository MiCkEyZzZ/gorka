use gorka::{GlonassDecoder, GlonassEncoder, GlonassSample, MilliHz, Millimeter};

fn main() {
    // Создаём серию наблюдений одного спутника
    // Обычный приёмник ГЛОНАСС выдаёт наблюдения с частотой 1–10 Гц.
    // Здесь моделируем 1 секунду данных (10 эпох по 100 мс).
    let base_ts_ms: u64 = 1_700_000_000_000; // 2023-11-14 22:13:20 UTC

    let samples: Vec<GlonassSample> = (0..10)
        .map(|i| GlonassSample {
            // Timestamp в миллисекундах от Unix epoch
            timestamp_ms: base_ts_ms + i * 100,

            // FDMA-слот k ∈ [−7, +6]. Несущая: 1602 + k × 0.5625 МГц
            slot: 1,

            // Отношение сигнал/шум [дБГц], типично 30–50
            cn0_dbhz: 42 + (i % 5) as u8,

            // Псевдодальность в миллиметрах (1 мм точность)
            // Типично 19 100 000 000 .. 25 600 000 000 мм
            pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),

            // Доплеровский сдвиг в мГц (0.001 Гц точность)
            doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 50),

            // Накопленная фаза несущей (в единицах 2⁻³² цикла), опциональна
            carrier_phase_cycles: Some(100_000_i64 + i as i64 * 65_536),
        })
        .collect();

    // Сжимаем chunk
    let compressed = GlonassEncoder::encode_chunk(&samples).unwrap();

    let raw_bytes: usize = samples
        .iter()
        .map(|s| {
            8 + 1
                + 1
                + 8
                + 4
                + 1
                + if s.carrier_phase_cycles.is_some() {
                    8
                } else {
                    0
                }
        })
        .sum();

    let ratio = raw_bytes as f64 / compressed.len() as f64;

    println!("=== Gorka basic_encode ===");
    println!("Samples:     {}", samples.len());
    println!("Raw size:    {} B", raw_bytes);
    println!("Compressed:  {} B", compressed.len());
    println!("Ratio:       {ratio:.2}×");
    println!();

    // Декодируем обратно
    let decoded = GlonassDecoder::decode_chunk(&compressed).unwrap();

    assert_eq!(decoded.len(), samples.len(), "sample count mismatch");

    // Полная битовая идентичность: ни один бит не потерян
    for (i, (orig, got)) in samples.iter().zip(&decoded).enumerate() {
        assert_eq!(orig.timestamp_ms, got.timestamp_ms, "ts[{i}]");
        assert_eq!(orig.slot, got.slot, "slot[{i}]");
        assert_eq!(orig.cn0_dbhz, got.cn0_dbhz, "cn0[{i}]");
        assert_eq!(orig.pseudorange_mm, got.pseudorange_mm, "pr[{i}]");
        assert_eq!(orig.doppler_millihz, got.doppler_millihz, "dop[{i}]");
        assert_eq!(
            orig.carrier_phase_cycles, got.carrier_phase_cycles,
            "phase[{i}]"
        );
    }

    println!(
        "Roundtrip OK — all {} samples decoded identically ✓",
        decoded.len()
    );

    // Инспектируем несколько полей первого сэмпла
    let s = &decoded[0];
    println!();
    println!("First sample:");
    println!("  timestamp_ms:    {}", s.timestamp_ms);
    println!("  slot:            k={}", s.slot);
    println!("  cn0_dbhz:        {} dBHz", s.cn0_dbhz);
    println!(
        "  pseudorange:     {:.3} m",
        s.pseudorange_mm.0 as f64 / 1_000.0
    );
    println!(
        "  doppler:         {:.3} Hz",
        s.doppler_millihz.0 as f64 / 1_000.0
    );
    println!(
        "  carrier_freq:    {:.4} MHz",
        s.carrier_freq_millihz().unwrap() as f64 / 1_000_000.0
    );
    println!("  tracked:         {}", s.is_tracked());
}
