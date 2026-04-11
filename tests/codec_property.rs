use gorka::{DbHz, GloSlot, GlonassDecoder, GlonassEncoder, GlonassSample, MilliHz, Millimeter};
use proptest::prelude::*;

const BASE_TS: u64 = 1_700_000_000_000;

// Генератор валидного sample (случайный)
fn arb_sample() -> impl Strategy<Value = GlonassSample> {
    (
        0u64..10_000,                      // timestamp offset
        -7i8..=6,                          // slot
        20u8..60,                          // cn0
        21_000_000_000i64..22_000_000_000, // pseudorange
        -5_000_000i32..5_000_000,          // doppler
        proptest::option::of(-1_000_000_000i64..1_000_000_000),
    )
        .prop_map(|(ts, slot, cn0, pr, doppler, phase)| GlonassSample {
            timestamp_ms: BASE_TS + ts,
            slot: GloSlot::new(slot).unwrap(),
            cn0_dbhz: DbHz::new(cn0).unwrap(),
            pseudorange_mm: Millimeter::new(pr),
            doppler_millihz: MilliHz::new(doppler),
            carrier_phase_cycles: phase,
        })
}

// Генератор случайного вектора samples
fn arb_samples() -> impl Strategy<Value = Vec<GlonassSample>> {
    prop::collection::vec(arb_sample(), 1..128)
}

// Генератор монотонного, реалистичного временного ряда
fn arb_timeseries() -> impl Strategy<Value = Vec<GlonassSample>> {
    (1..128usize)
        .prop_flat_map(|len| {
            (
                Just(len),
                -7i8..=6,     // slot
                0u64..10_000, // стартовый timestamp offset
            )
        })
        .prop_map(|(len, slot, start)| {
            let mut ts = BASE_TS + start;
            let mut samples = Vec::with_capacity(len);

            for i in 0..len {
                ts += 1; // монотонный рост timestamp

                samples.push(GlonassSample {
                    timestamp_ms: ts,
                    slot: GloSlot::new(slot).unwrap(),
                    cn0_dbhz: DbHz::new(40 + (i % 5) as u8).unwrap(),
                    pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 10),
                    doppler_millihz: MilliHz::new(1_200_000 + i as i32),
                    carrier_phase_cycles: Some(i as i64 * 1000),
                });
            }

            samples
        })
}

proptest! {
    #[test]
    fn prop_roundtrip(samples in arb_samples()) {
        let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();
        let decoded = GlonassDecoder::decode_chunk(&encoded).unwrap();

        prop_assert_eq!(samples, decoded);
    }

    #[test]
    fn prop_roundtrip_timeseries(samples in arb_timeseries()) {
        let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();
        let decoded = GlonassDecoder::decode_chunk(&encoded).unwrap();

        prop_assert_eq!(samples, decoded);
    }
}
