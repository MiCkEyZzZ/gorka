use gorka::{GlonassDecoder, GlonassEncoder, GlonassSample, MilliHz, Millimeter};

fn main() {
    let samples = vec![GlonassSample {
        timestamp_ms: 1_700_000_000_000,
        slot: -2,
        cn0_dbhz: 45,
        pseudorange_mm: Millimeter::new(21_500_000_000),
        doppler_millihz: MilliHz::new(-1200),
        carrier_phase_cycles: Some(123456789),
    }];

    let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();
    let decoded = GlonassDecoder::decode_chunk(&encoded).unwrap();

    assert_eq!(samples, decoded);
    println!(
        "Basic encode/decode successful! Encoded {} bytes",
        encoded.len()
    );
}
