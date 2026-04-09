use gorka::{codec::GlonassEncoder, DbHz, GloSlot, GlonassSample, MilliHz, Millimeter};

const BASE_TS: u64 = 1_700_000_000_000;

fn raw_sample_size(sample: &GlonassSample) -> usize {
    // timestamp(8) + slot(1) + cn0(1) + pr(8) + doppler(4) + phase_flag(1) +
    // phase(0/8)
    8 + 1
        + 1
        + 8
        + 4
        + 1
        + if sample.carrier_phase_cycles.is_some() {
            8
        } else {
            0
        }
}

fn raw_size(samples: &[GlonassSample]) -> usize {
    samples.iter().map(raw_sample_size).sum()
}

fn print_stats(
    name: &str,
    samples: &[GlonassSample],
    compressed: usize,
) {
    let raw = raw_size(samples);
    let ratio = raw as f64 / compressed as f64;
    let bits_per_sample = (compressed as f64 * 8.0) / samples.len() as f64;

    println!(
        "{name}: raw={raw}B compressed={compressed}B ratio={ratio:.2}× bits/sample={bits_per_sample:.2}"
    );
}

fn constant_samples(count: usize) -> Vec<GlonassSample> {
    (0..count)
        .map(|i| GlonassSample {
            timestamp_ms: BASE_TS + i as u64,
            slot: GloSlot::new(1).unwrap(),
            cn0_dbhz: DbHz::new(42).unwrap(),
            pseudorange_mm: Millimeter::new(21_500_000_000),
            doppler_millihz: MilliHz::new(1_200_500),
            carrier_phase_cycles: None,
        })
        .collect()
}

fn smooth_samples(count: usize) -> Vec<GlonassSample> {
    (0..count)
        .map(|i| GlonassSample {
            timestamp_ms: BASE_TS + i as u64,
            slot: GloSlot::new(1).unwrap(),
            cn0_dbhz: DbHz::new(45 + (i % 3) as u8).unwrap(),
            pseudorange_mm: Millimeter::new(21_500_000_000 + (i as i64 * 5)),
            doppler_millihz: MilliHz::new(1_200_000 + (i as i32 * 2)),
            carrier_phase_cycles: Some(100_000_i64 + (i as i64 * 16_384)),
        })
        .collect()
}

fn noisy_samples(count: usize) -> Vec<GlonassSample> {
    let mut x: u64 = 0xC0FFEE_u64;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };

    let mut ts = BASE_TS;
    let mut out = Vec::with_capacity(count);

    for i in 0..count {
        let r1 = next();
        let r2 = next();
        let r3 = next();
        let r4 = next();

        ts += 1 + (r1 % 4);

        let slot = GloSlot::new((r2 % 14) as i8 - 7).unwrap();
        let cn0 = DbHz::new(20 + (r3 % 40) as u8).unwrap();

        let pr_jitter = (r4 % 2_001) as i64 - 1_000; // [-1000; 1000]
        let doppler_jitter = (next() % 20_001) as i32 - 10_000; // [-10000; 10000]

        let phase = if i % 5 == 0 {
            None
        } else {
            Some(1_000_000_i64 + (next() % 50_001) as i64 - 25_000)
        };

        out.push(GlonassSample {
            timestamp_ms: ts,
            slot,
            cn0_dbhz: cn0,
            pseudorange_mm: Millimeter::new(21_500_000_000 + pr_jitter),
            doppler_millihz: MilliHz::new(1_200_000 + doppler_jitter),
            carrier_phase_cycles: phase,
        });
    }

    out
}

#[test]
fn compression_ratio_constant() {
    let samples = constant_samples(512);
    let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();

    print_stats("constant", &samples, encoded.len());

    let ratio = raw_size(&samples) as f64 / encoded.len() as f64;
    assert!(
        ratio >= 10.0,
        "constant signal must compress ≥10×, got {ratio:.2}×"
    );
}

#[test]
fn compression_ratio_smooth() {
    let samples = smooth_samples(512);
    let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();

    print_stats("smooth", &samples, encoded.len());

    let ratio = raw_size(&samples) as f64 / encoded.len() as f64;
    assert!(
        ratio >= 4.0,
        "smooth signal must compress ≥4×, got {ratio:.2}×"
    );
}

#[test]
fn compression_ratio_noisy() {
    let samples = noisy_samples(512);
    let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();

    print_stats("noisy", &samples, encoded.len());

    let ratio = raw_size(&samples) as f64 / encoded.len() as f64;
    assert!(
        ratio >= 2.0,
        "noisy signal must compress ≥2×, got {ratio:.2}×"
    );
}
