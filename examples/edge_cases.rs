use gorka::{
    codec::{GlonassDecoder, GlonassEncoder},
    GlonassSample, MilliHz, Millimeter,
};

fn main() {
    let samples = vec![
        GlonassSample {
            timestamp_ms: 1_700_000_000_000,
            slot: -7,
            cn0_dbhz: 0,
            pseudorange_mm: Millimeter::new(0),
            doppler_millihz: MilliHz::new(0),
            carrier_phase_cycles: None, // edge case: missing carrier phase
        },
        GlonassSample {
            timestamp_ms: 1_700_000_100_000, // large timestamp gap
            slot: 6,
            cn0_dbhz: 50,
            pseudorange_mm: Millimeter::new(22_000_000_000),
            doppler_millihz: MilliHz::new(500),
            carrier_phase_cycles: Some(123456789),
        },
    ];

    let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();
    let decoded = GlonassDecoder::decode_chunk(&encoded).unwrap();

    assert_eq!(samples, decoded);
    println!("Edge cases passed! Encoded {} bytes", encoded.len());
}
