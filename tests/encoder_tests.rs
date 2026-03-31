use gorka::{
    codec::{GlonassDecoder, GlonassEncoder},
    GlonassSample, GorkaError, MilliHz, Millimeter,
};

const BASE_TS: u64 = 1_700_000_000_000;

fn sample(
    i: u64,
    slot: i8,
) -> GlonassSample {
    GlonassSample {
        timestamp_ms: BASE_TS + i,
        slot,
        cn0_dbhz: 40 + (i % 10) as u8,
        pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),
        doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 50),
        carrier_phase_cycles: Some(100_000 + i as i64 * 21 * (1 << 16)),
    }
}

fn constant_sample(
    i: u64,
    slot: i8,
) -> GlonassSample {
    GlonassSample {
        timestamp_ms: BASE_TS + i,
        slot,
        cn0_dbhz: 42,
        pseudorange_mm: Millimeter::new(21_500_000_000),
        doppler_millihz: MilliHz::new(1_200_500),
        carrier_phase_cycles: None,
    }
}

fn roundtrip(samples: &[GlonassSample]) -> Vec<GlonassSample> {
    let encoded = GlonassEncoder::encode_chunk(samples).unwrap();
    GlonassDecoder::decode_chunk(&encoded).unwrap()
}

#[test]
fn test_roundtrip_single_sample() {
    let orig = vec![sample(0, 1)];
    let dec = roundtrip(&orig);

    assert_eq!(orig, dec);
}

#[test]
fn test_roundtrip_512_samples_smooth() {
    let orig: Vec<_> = (0..512).map(|i| sample(i, 1)).collect();
    let dec = roundtrip(&orig);

    assert_eq!(orig, dec);
}

#[test]
fn test_roundtrip_with_gaps_and_signal_loss() {
    let mut samples = Vec::new();

    // нормальные данные
    for i in 0..16 {
        samples.push(sample(i, 1));
    }

    // gap по времени
    samples.push(sample(10_000, 1));

    // потеря фазы
    for i in 10_001..10_010 {
        samples.push(GlonassSample {
            carrier_phase_cycles: None,
            ..constant_sample(i, 1)
        });
    }

    let dec = roundtrip(&samples);

    assert_eq!(samples, dec);
}

#[test]
fn test_all_14_slots_in_one_chunk() {
    let mut samples = Vec::new();

    for i in 0..64u64 {
        let slot = (i % 14) as i8 - 7;
        samples.push(sample(i, slot));
    }

    let dec = roundtrip(&samples);

    assert_eq!(samples, dec);
}

#[test]
fn test_empty_chunk_error() {
    let err = GlonassEncoder::encode_chunk(&[]).unwrap_err();

    assert!(matches!(err, GorkaError::EmptyChunk));
}

#[test]
fn test_invalid_slot_error() {
    let bad = GlonassSample {
        slot: 10,
        ..sample(0, 1)
    };

    let err = GlonassEncoder::encode_chunk(&[bad]).unwrap_err();

    assert!(matches!(err, GorkaError::InvalidSlot(_)));
}

#[test]
fn test_invalid_magic_error() {
    let mut buf = GlonassEncoder::encode_chunk(&[sample(0, 1)]).unwrap();

    buf[0] ^= 0xFF;

    let err = GlonassDecoder::decode_chunk(&buf).unwrap_err();

    assert!(matches!(err, GorkaError::InvalidMagic(_)));
}
