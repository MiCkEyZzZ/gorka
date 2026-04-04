//! Benchmarks: decode APIs for `GlonassDecoder`.
//!
//! Запуск:
//! ```zsh
//! cargo bench --bench decoder_bench
//! ```

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gorka::{
    codec::{GlonassDecoder, GlonassEncoder},
    GlonassSample, MilliHz, Millimeter,
};

const BASE_TS: u64 = 1_700_000_000_000;

fn smooth_samples(n: usize) -> Vec<GlonassSample> {
    (0..n)
        .map(|i| GlonassSample {
            timestamp_ms: BASE_TS + i as u64,
            slot: 1,
            cn0_dbhz: 42 + (i % 5) as u8,
            pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),
            doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 10),
            carrier_phase_cycles: Some(100_000_i64 + i as i64 * 21 * (1 << 16)),
        })
        .collect()
}

fn constant_samples(n: usize) -> Vec<GlonassSample> {
    (0..n)
        .map(|i| GlonassSample {
            timestamp_ms: BASE_TS + i as u64,
            slot: 0,
            cn0_dbhz: 42,
            pseudorange_mm: Millimeter::new(21_500_000_000),
            doppler_millihz: MilliHz::new(1_200_500),
            carrier_phase_cycles: None,
        })
        .collect()
}

fn multi_slot_samples(n: usize) -> Vec<GlonassSample> {
    let slots: [i8; 4] = [-3, 0, 3, 6];

    (0..n)
        .map(|i| GlonassSample {
            timestamp_ms: BASE_TS + i as u64,
            slot: slots[i % 4],
            cn0_dbhz: 38 + (i % 8) as u8,
            pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 150),
            doppler_millihz: MilliHz::new(1_100_000 + i as i32 * 20),
            carrier_phase_cycles: if i % 3 == 0 {
                None
            } else {
                Some(i as i64 * 65536)
            },
        })
        .collect()
}

fn bench_decode_chunk(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_chunk");

    for &n in &[128usize, 1024, 8192] {
        // smooth
        let samples = smooth_samples(n);
        let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();
        group.throughput(Throughput::Bytes(encoded.len() as u64));

        group.bench_with_input(BenchmarkId::new("smooth", n), &encoded, |b, e| {
            b.iter(|| GlonassDecoder::decode_chunk(black_box(e)).unwrap())
        });

        // constant (используем)
        let samples = constant_samples(n);
        let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();

        group.bench_with_input(BenchmarkId::new("constant", n), &encoded, |b, e| {
            b.iter(|| GlonassDecoder::decode_chunk(black_box(e)).unwrap())
        });
    }

    group.finish();
}

fn bench_decode_into(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_into");

    for &n in &[128usize, 1024, 8192] {
        // smooth
        let samples = smooth_samples(n);
        let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();
        let mut out = vec![GlonassSample::default_zeroed(); n];

        group.throughput(Throughput::Bytes(encoded.len() as u64));
        group.bench_with_input(BenchmarkId::new("smooth", n), &encoded, |b, e| {
            b.iter(|| {
                let written =
                    GlonassDecoder::decode_into(black_box(e), black_box(&mut out)).unwrap();
                black_box(written);
            })
        });

        // constant (используем)
        let samples = constant_samples(n);
        let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();

        group.bench_with_input(BenchmarkId::new("constant", n), &encoded, |b, e| {
            b.iter(|| {
                let written =
                    GlonassDecoder::decode_into(black_box(e), black_box(&mut out)).unwrap();
                black_box(written);
            })
        });
    }

    group.finish();
}

fn bench_iter_chunk(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_chunk");

    for &n in &[128usize, 1024, 8192] {
        let samples = multi_slot_samples(n);
        let encoded = GlonassEncoder::encode_chunk(&samples).unwrap();

        group.throughput(Throughput::Bytes(encoded.len() as u64));
        group.bench_with_input(BenchmarkId::new("multi_slot", n), &encoded, |b, e| {
            b.iter(|| {
                let iter = GlonassDecoder::iter_chunk(black_box(e)).unwrap();
                let mut count = 0usize;

                for item in iter {
                    black_box(item.unwrap());
                    count += 1;
                }

                black_box(count);
            })
        });
    }

    group.finish();
}

criterion_group!(
    decode_benches,
    bench_decode_chunk,
    bench_decode_into,
    bench_iter_chunk
);
criterion_main!(decode_benches);
