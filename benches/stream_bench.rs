use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gorka::{DbHz, GloSlot, GlonassSample, MilliHz, Millimeter, StreamEncoder};

const BASE_TS: u64 = 1_700_000_000_000;

// Генерация с фазой
fn sample_with_phase(
    i: u64,
    slot: i8,
) -> GlonassSample {
    GlonassSample {
        timestamp_ms: BASE_TS + i * 1000,
        slot: GloSlot::new(slot).unwrap(),
        cn0_dbhz: DbHz::new(42).unwrap(),
        pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),
        doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 50),
        carrier_phase_cycles: Some(100_000_i64 + i as i64 * 21 * (1 << 16)),
    }
}

// Генерация без фазы
fn sample_no_phase(
    i: u64,
    slot: i8,
) -> GlonassSample {
    GlonassSample {
        timestamp_ms: BASE_TS + i * 1000,
        slot: GloSlot::new(slot).unwrap(),
        cn0_dbhz: DbHz::new(42).unwrap(),
        pseudorange_mm: Millimeter::new(21_500_000_000),
        doppler_millihz: MilliHz::new(1_200_500),
        carrier_phase_cycles: None,
    }
}

fn bench_case(
    c: &mut Criterion,
    size: usize,
    with_phase: bool,
) {
    let name = if with_phase {
        format!("encode/{size}/with_phase")
    } else {
        format!("encode/{size}/no_phase")
    };

    // Создаём безопасные слоты - всегда в диапазоне [-7, 6]
    let samples: Vec<GlonassSample> = (0..size as u64)
        .map(|i| {
            let slot = ((i % 14) as i8) - 7; // диапазон -7..6
            if with_phase {
                sample_with_phase(i, slot)
            } else {
                sample_no_phase(i, slot)
            }
        })
        .collect();

    c.bench_with_input(
        BenchmarkId::new("StreamEncoder", name),
        &samples,
        |b, samples| {
            let mut enc_buf = vec![0u8; 1024 * 1024];
            let mut out_buf = vec![0u8; 1024 * 1024];

            b.iter(|| {
                let mut enc = StreamEncoder::new(enc_buf.as_mut_slice());

                for s in samples.iter() {
                    black_box(enc.push_sample(s).unwrap());
                }

                black_box(enc.flush(out_buf.as_mut_slice()).unwrap());
            });
        },
    );
}

fn bench_stream_encoder(c: &mut Criterion) {
    let sizes = [128, 1024, 8192];

    for &size in &sizes {
        bench_case(c, size, true);
        bench_case(c, size, false);
    }
}

criterion_group!(benches, bench_stream_encoder);
criterion_main!(benches);
