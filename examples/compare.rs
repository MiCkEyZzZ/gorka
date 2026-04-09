use std::io::Write;

use flate2::{write::GzEncoder, Compression};
use gorka::{DbHz, GloSlot, GlonassEncoder, GlonassSample, MilliHz, Millimeter};

fn make_data(n: usize) -> Vec<GlonassSample> {
    let base_ts = 1_700_000_000_000;

    (0..n)
        .map(|i| GlonassSample {
            timestamp_ms: base_ts + i as u64 * 1000,
            slot: GloSlot::new(1).unwrap(),
            cn0_dbhz: DbHz::new(40 + (i % 5) as u8).unwrap(),
            pseudorange_mm: Millimeter::new(21_000_000_000 + i as i64 * 200),
            doppler_millihz: MilliHz::new(1_000_000 - i as i32 * 50),
            carrier_phase_cycles: Some(i as i64 * 1_000_000),
        })
        .collect()
}

fn estimate_raw_size(samples: &[GlonassSample]) -> usize {
    samples
        .iter()
        .map(|s| {
            23 + if s.carrier_phase_cycles.is_some() {
                8
            } else {
                0
            }
        })
        .sum()
}

fn main() {
    println!("=== Gorka vs gzip ===\n");

    let samples = make_data(300);

    // RAW
    let raw_size = estimate_raw_size(&samples);

    // GORKA
    let gorka = GlonassEncoder::encode_chunk(&samples).expect("encode failed");
    let gorka_size = gorka.len();

    // GZIP
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());

    // важно: gzip'им сырые байты, а не структуру
    // делаем простой бинарный dump (как будто "raw stream")
    for s in &samples {
        gz.write_all(&s.timestamp_ms.to_le_bytes()).unwrap();
        gz.write_all(&s.slot.get().to_le_bytes()).unwrap();
        gz.write_all(&[s.cn0_dbhz.get()]).unwrap();
        gz.write_all(&s.pseudorange_mm.as_i64().to_le_bytes())
            .unwrap();
        gz.write_all(&s.doppler_millihz.as_i32().to_le_bytes())
            .unwrap();

        if let Some(cp) = s.carrier_phase_cycles {
            gz.write_all(&[1]).unwrap();
            gz.write_all(&cp.to_le_bytes()).unwrap();
        } else {
            gz.write_all(&[0]).unwrap();
        }
    }

    let gzip_bytes = gz.finish().unwrap();
    let gzip_size = gzip_bytes.len();

    // OUTPUT

    println!("Samples: {}", samples.len());
    println!();

    println!("Raw:     {} B", raw_size);
    println!(
        "Gorka:   {} B ({:.2}×)",
        gorka_size,
        raw_size as f64 / gorka_size as f64
    );
    println!(
        "gzip:    {} B ({:.2}×)",
        gzip_size,
        raw_size as f64 / gzip_size as f64
    );

    println!();

    println!(
        "Gorka vs gzip: {:.2}× better",
        gzip_size as f64 / gorka_size as f64
    );

    println!(
        "Bytes/sample: {:.2} → {:.2} (gorka), {:.2} (gzip)",
        raw_size as f64 / samples.len() as f64,
        gorka_size as f64 / samples.len() as f64,
        gzip_size as f64 / samples.len() as f64
    );
}
