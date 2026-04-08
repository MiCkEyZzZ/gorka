#[allow(deprecated)]
use gorka::{
    decode_i64, delta_of_delta_i64, encode_i64, BitReader, BitWriter, GlonassSample, GnssFrame,
    GorkaError, MilliHz, Millimeter,
};
use gorka::{GlonassDecoder, GlonassEncoder};

fn main() {
    println!("=== Gorka no_std demo ===");
    println!();

    // Кодек без IO
    // encode_chunk / decode_chunk не используют std::fs, std::net, и т.д.
    // Они работают с &[u8] и Vec<u8> из alloc.

    let samples = vec![
        GlonassSample {
            timestamp_ms: 1_700_000_000_000,
            slot: -3,
            cn0_dbhz: 38,
            pseudorange_mm: Millimeter::new(22_100_000_000),
            doppler_millihz: MilliHz::new(-2_500_000),
            carrier_phase_cycles: Some(987_654_321),
        },
        GlonassSample {
            timestamp_ms: 1_700_000_001_000,
            slot: -3,
            cn0_dbhz: 39,
            pseudorange_mm: Millimeter::new(22_100_000_300),
            doppler_millihz: MilliHz::new(-2_499_950),
            carrier_phase_cycles: Some(987_654_321 + 65_536),
        },
    ];

    let encoded = GlonassEncoder::encode_chunk(&samples).expect("encode failed");

    let decoded = GlonassDecoder::decode_chunk(&encoded).expect("decode failed");

    assert_eq!(samples, decoded);
    println!("Codec (encode + decode): OK");
    println!(
        "  {} samples → {} B compressed",
        samples.len(),
        encoded.len()
    );

    // Прямой доступ к bit-IO
    // BitWriter и BitReader — pure no_std примитивы.

    #[allow(deprecated)]
    let mut w = BitWriter::new();
    w.write_bits(0b1011, 4).unwrap();
    w.write_bits_signed(-42i64, 16).unwrap();
    let data = w.finish();

    let mut r = BitReader::new(&data);
    let bits = r.read_bits(4).unwrap();
    let val = r.read_bits_signed(16).unwrap();

    assert_eq!(bits, 0b1011);
    assert_eq!(val, -42);
    println!("BitWriter / BitReader: OK");

    // Zigzag и delta-арифметика
    // Все вычислительные примитивы — чистые функции, без зависимостей.

    let zz = encode_i64(-1_200_500);
    assert_eq!(decode_i64(zz), -1_200_500);

    let dod = delta_of_delta_i64(
        21_500_000_222, // current pseudorange
        21_500_000_000, // previous
        222,            // previous delta
    );
    assert_eq!(dod, 0); // постоянное ускорение → DoD = 0

    println!("encode_i64 / decode_i64 / delta_of_delta_i64: OK");

    // GnssFrame — буфер эпохи без аллокаций на стеке
    // GnssFrame хранит до 14 наблюдений в fixed-size массиве на стеке.
    // Не требует ни heap, ни std.

    let ts = 1_700_000_000_000u64;
    let mut frame = GnssFrame::new(ts);

    for slot in [-7i8, -3, 0, 3] {
        frame
            .push(GlonassSample {
                timestamp_ms: ts,
                slot,
                cn0_dbhz: 40,
                pseudorange_mm: Millimeter::new(21_500_000_000),
                doppler_millihz: MilliHz::new(1_000_000),
                carrier_phase_cycles: None,
            })
            .unwrap();
    }

    assert_eq!(frame.len(), 4);
    assert!(frame.contains_slot(-7));
    assert!(frame.validate_all().is_ok());

    println!("GnssFrame (stack-allocated, 14 slots): OK");

    // Обработка ошибок без std::error::Error
    // GorkaError реализует core::fmt::Display в no_std.
    // (std::error::Error доступен только при feature = "std")

    let err = GorkaError::InvalidSlot(99);
    // В no_std можно матчить варианты напрямую:
    assert!(matches!(err, GorkaError::InvalidSlot(99)));

    // В std-окружении Display тоже доступен:
    println!("GorkaError matching: OK");

    println!();
    println!("All no_std-compatible APIs verified ✓");
    println!();
    println!("To use gorka without std, add to Cargo.toml:");
    println!(
        "  gorka = {{ version = \"0.4.1\", default-features = false, features = [\"alloc\"] }}"
    );
    println!();
    println!("The gorka::io module (ChunkWriter/ChunkReader) requires std.");
}
