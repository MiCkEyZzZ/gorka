use std::{
    fs,
    io::{BufWriter, Read},
    path::Path,
};

use gorka::{
    io::{ChunkReader, ChunkWriter},
    DbHz, GloSlot, GlonassDecoder, GlonassEncoder, GlonassSample, MilliHz, Millimeter,
};

/// Генератор наблюдений для одного спутника.
fn make_track(
    slot: i8,
    n: usize,
    base_ts_ms: u64,
) -> Vec<GlonassSample> {
    let glo_slot = GloSlot::new(slot).unwrap();

    (0..n)
        .map(|i| GlonassSample {
            timestamp_ms: base_ts_ms + i as u64 * 1000, // 1 Гц
            slot: glo_slot,
            cn0_dbhz: DbHz::new(35 + (i % 10) as u8).unwrap(),
            pseudorange_mm: Millimeter::new(20_000_000_000 + i as i64 * 300),
            doppler_millihz: MilliHz::new(500_000 - i as i32 * 100),
            carrier_phase_cycles: Some(i as i64 * 1_048_576),
        })
        .collect()
}

#[cfg(feature = "std")]
fn main() -> std::io::Result<()> {
    let path = Path::new("/tmp/gorka_stream_demo.bin");

    // Конфигурация
    let slots: [i8; 5] = [-7, -3, 0, 3, 6];
    let epochs: usize = 60;
    let base_ts: u64 = 1_700_000_000_000;

    println!("=== Gorka streaming example ===");
    println!("Satellites: {}", slots.len());
    println!("Epochs/sat: {epochs}");
    println!("Output: {}", path.display());
    println!();

    //  Шаг 1: Запись

    let file = fs::File::create(path)?;
    let mut writer = ChunkWriter::new(BufWriter::new(file));

    let mut all_tracks = Vec::new();
    let mut total_raw = 0usize;
    let mut total_samples = 0usize;
    let mut ratios = Vec::new();

    for &slot in &slots {
        let track = make_track(slot, epochs, base_ts);
        let chunk = GlonassEncoder::encode_chunk(&track).expect("encode failed");

        let raw: usize = track
            .iter()
            .map(|s| {
                23 + if s.carrier_phase_cycles.is_some() {
                    8
                } else {
                    0
                }
            })
            .sum();

        total_raw += raw;
        total_samples += track.len();

        let ratio = raw as f64 / chunk.len() as f64;
        ratios.push(ratio);

        println!(
            "  slot k={slot:+}: {} samples, {} B raw → {} B compressed ({:.2}×)",
            track.len(),
            raw,
            chunk.len(),
            ratio
        );

        writer.write_chunk(&chunk)?;
        all_tracks.push(track);
    }

    writer.flush()?;

    let file_size = fs::metadata(path)?.len() as usize;

    println!();
    println!(
        "Written {} chunks, {} B total raw → {} B on disk ({:.2}×)",
        writer.chunks_written(),
        total_raw,
        file_size,
        total_raw as f64 / file_size as f64
    );

    //  Шаг 2: Чтение

    let mut raw_bytes = Vec::new();
    fs::File::open(path)?.read_to_end(&mut raw_bytes)?;

    println!();
    println!("Reading back from disk...");

    let reader = ChunkReader::new(&raw_bytes);
    let mut decoded_tracks = Vec::new();

    for (i, frame) in reader.enumerate() {
        let payload = frame.expect("corrupted frame");
        let track = GlonassDecoder::decode_chunk(payload).expect("decode failed");

        println!(
            "  chunk[{i}]: {} samples, slot k={:+}",
            track.len(),
            track[0].slot.get()
        );

        decoded_tracks.push(track);
    }

    // Шаг 3: Верификация

    println!();
    assert_eq!(decoded_tracks.len(), all_tracks.len());

    for (i, (orig, got)) in all_tracks.iter().zip(&decoded_tracks).enumerate() {
        assert_eq!(orig, got, "chunk[{i}] mismatch");
    }

    println!(
        "All {} chunks verified — lossless roundtrip ✓",
        decoded_tracks.len()
    );

    // Summary

    if !ratios.is_empty() && total_samples > 0 {
        let avg_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
        let min_ratio = ratios.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ratio = ratios.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        println!();
        println!("--- Summary ---");
        println!("Chunks: {}", writer.chunks_written());
        println!("Total samples: {total_samples}");
        println!("Avg compression: {:.2}×", avg_ratio);
        println!("Max chunk ratio: {:.2}×", max_ratio);
        println!("Min chunk ratio: {:.2}×", min_ratio);

        println!(
            "Bytes/sample: {:.2} → {:.2}",
            total_raw as f64 / total_samples as f64,
            file_size as f64 / total_samples as f64
        );
    }

    // Очистка
    let _ = fs::remove_file(path);

    Ok(())
}
