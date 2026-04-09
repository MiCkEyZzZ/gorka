use std::time::Instant;

use gorka::{DbHz, GloSlot, GlonassDecoder, GlonassSample, MilliHz, Millimeter, StreamEncoder};

fn main() {
    println!("=== Gorka StreamEncoder performance test ===");

    let n_samples = 100_000;
    let mut buf = vec![0u8; 1024 * 1024 * 4]; // 4 MB buffer
    let mut enc = StreamEncoder::new(&mut buf);

    let samples: Vec<GlonassSample> = (0..n_samples)
        .map(|i| GlonassSample {
            timestamp_ms: 1_700_000_000_000 + i as u64 * 1000,
            slot: GloSlot::new((i % 14) as i8 - 7).unwrap(), // GloSlot
            cn0_dbhz: DbHz::new(42).unwrap(),                // DbHz
            pseudorange_mm: Millimeter::new(21_500_000_000),
            doppler_millihz: MilliHz::new(1_200_000),
            carrier_phase_cycles: Some(i as i64 * 65_536),
        })
        .collect();

    let start = Instant::now();
    for s in &samples {
        enc.push_sample(s).unwrap();
    }
    let duration_push = start.elapsed();

    let mut out = vec![0u8; 1024 * 1024 * 4];
    let total_bytes = enc.flush(&mut out).unwrap();
    let duration_flush = start.elapsed();

    println!("{} samples encoded → {} bytes", n_samples, total_bytes);
    println!("Push duration: {:.3?}", duration_push);
    println!("Flush duration: {:.3?}", duration_flush);

    // Декодируем и проверяем
    let start_decode = Instant::now();
    let decoded = GlonassDecoder::decode_chunk(&out[..total_bytes]).unwrap();
    let duration_decode = start_decode.elapsed();

    assert_eq!(decoded.len(), n_samples);
    assert_eq!(decoded, samples);
    println!("Decoding OK ✅");
    println!("Decode duration: {:.3?}", duration_decode);
}
