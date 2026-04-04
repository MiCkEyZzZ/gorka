use gorka::{GlonassDecoder, GlonassSample, MilliHz, Millimeter, StreamEncoder};

fn main() {
    println!("=== Gorka StreamEncoder basic demo ===");

    let mut buf = vec![0u8; 1024 * 64];
    let mut enc = StreamEncoder::new(&mut buf);

    // Генерируем простые сэмплы
    let samples: Vec<GlonassSample> = (0..10)
        .map(|i| GlonassSample {
            timestamp_ms: 1_700_000_000_000 + i * 1000,
            slot: (i % 14) as i8 - 7,
            cn0_dbhz: 40 + (i % 5) as u8,
            pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),
            doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 50),
            carrier_phase_cycles: if i % 2 == 0 {
                Some(100_000 + i as i64 * 65_536)
            } else {
                None
            },
        })
        .collect();

    for s in &samples {
        let written = enc.push_sample(s).expect("push_sample");
        println!(
            "Pushed sample {}, bytes written: {}",
            s.timestamp_ms, written
        );
    }

    let mut out = vec![0u8; 1024 * 64];
    let total_bytes = enc.flush(&mut out).expect("flush");
    println!(
        "Flushed {} samples → {} bytes",
        enc.sample_count(),
        total_bytes
    );

    let decoded = GlonassDecoder::decode_chunk(&out[..total_bytes]).expect("decode_chunk");

    assert_eq!(decoded.len(), samples.len());
    assert_eq!(decoded, samples);
    println!("Roundtrip OK ✅ — all samples match exactly");
}
