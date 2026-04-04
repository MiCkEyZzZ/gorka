use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use gorka::{GlonassSample, MilliHz, Millimeter, StreamEncoder};

const BASE_TS: u64 = 1_700_000_000_000;

// генератор с фазой
fn sample(
    i: u64,
    slot: i8,
) -> GlonassSample {
    GlonassSample {
        timestamp_ms: BASE_TS + i,
        slot,
        cn0_dbhz: 42,
        pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),
        doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 50),
        carrier_phase_cycles: Some(100_000_i64 + i as i64 * 21 * (1 << 16)),
    }
}

fn constant(
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

fn bench_stream_encoder(c: &mut Criterion) {
    let mut group = c.benchmark_group("StreamEncoder");

    // 128 samples с фазой
    let samples_with_phase: Vec<_> = (0..128).map(|i| sample(i, (i % 14) as i8 - 7)).collect();
    group.bench_function("encode 128 samples with phase", |b| {
        b.iter(|| {
            let mut buf = vec![0u8; 65536];
            let mut enc = StreamEncoder::new(&mut buf);
            for s in &samples_with_phase {
                black_box(enc.push_sample(s).unwrap());
            }
            let mut out = vec![0u8; 65536];
            black_box(enc.flush(&mut out).unwrap());
        })
    });

    // 128 samples без фазы
    let samples_no_phase: Vec<_> = (0..128).map(|i| constant(i, (i % 14) as i8 - 7)).collect();
    group.bench_function("encode 128 samples no phase", |b| {
        b.iter(|| {
            let mut buf = vec![0u8; 65536];
            let mut enc = StreamEncoder::new(&mut buf);
            for s in &samples_no_phase {
                black_box(enc.push_sample(s).unwrap());
            }
            let mut out = vec![0u8; 65536];
            black_box(enc.flush(&mut out).unwrap());
        })
    });

    group.finish();
}

criterion_group!(benches, bench_stream_encoder);
criterion_main!(benches);
