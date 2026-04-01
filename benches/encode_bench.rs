//! Benchmarks: encode и decode throughput для `GlonassEncoder` /
//! `GlonassDecoder`.
//!
//! Запуск:
//! ```zsh
//! cargo bench --bench encode_bench
//! cargo bench --bench encode_bench -- --save-baseline main
//! ```
//!
//! HTML-отчёт после запуска: `target/criterion/report/index.html`

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gorka::{
    codec::{GlonassDecoder, GlonassEncoder},
    GlonassSample, MilliHz, Millimeter,
};

const BASE_TS: u64 = 1_700_000_000_000;

/// Плавный реалистичный сигнал одного спутника.
/// Моделирует типичный GLONASS поток: монотонный timestamp, медленный дрейф
/// полей.
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

/// Постоянный сигнал — лучший случай для компрессора.
/// Все поля константны, только timestamp растёт по 1 мс.
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

/// Многоспутниковый chunk: чередование 4 слотов.
/// Проверяет overhead per-slot state tracking.
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

fn bench_encode_smooth(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode/smooth");

    for &n in &[128usize, 1024, 8192] {
        let samples = smooth_samples(n);
        let raw_bytes = n * 31; // 31 байт/сэмпл с фазой

        group.throughput(Throughput::Bytes(raw_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &samples, |b, s| {
            b.iter(|| GlonassEncoder::encode_chunk(black_box(s)).unwrap())
        });
    }

    group.finish();
}

fn bench_encode_constant(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode/constant");

    for &n in &[128usize, 1024, 8192] {
        let samples = constant_samples(n);
        let raw_bytes = n * 23; // 23 байт/сэмпл без фазы

        group.throughput(Throughput::Bytes(raw_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &samples, |b, s| {
            b.iter(|| GlonassEncoder::encode_chunk(black_box(s)).unwrap())
        });
    }

    group.finish();
}

fn bench_encode_multi_slot(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode/multi_slot");

    for &n in &[128usize, 1024, 8192] {
        let samples = multi_slot_samples(n);
        let raw_bytes = n * 27; // ~27 байт/сэмпл (смешанная фаза)

        group.throughput(Throughput::Bytes(raw_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &samples, |b, s| {
            b.iter(|| GlonassEncoder::encode_chunk(black_box(s)).unwrap())
        });
    }

    group.finish();
}

fn bench_decode_smooth(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode/smooth");

    for &n in &[128usize, 1024, 8192] {
        let encoded = GlonassEncoder::encode_chunk(&smooth_samples(n)).unwrap();
        let enc_bytes = encoded.len() as u64;

        // throughput считаем по сжатым байтам (это то, что читает decoder)
        group.throughput(Throughput::Bytes(enc_bytes));
        group.bench_with_input(BenchmarkId::from_parameter(n), &encoded, |b, e| {
            b.iter(|| GlonassDecoder::decode_chunk(black_box(e)).unwrap())
        });
    }

    group.finish();
}

fn bench_decode_constant(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode/constant");

    for &n in &[128usize, 1024, 8192] {
        let encoded = GlonassEncoder::encode_chunk(&constant_samples(n)).unwrap();
        let enc_bytes = encoded.len() as u64;

        group.throughput(Throughput::Bytes(enc_bytes));
        group.bench_with_input(BenchmarkId::from_parameter(n), &encoded, |b, e| {
            b.iter(|| GlonassDecoder::decode_chunk(black_box(e)).unwrap())
        });
    }

    group.finish();
}

fn bench_decode_multi_slot(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode/multi_slot");

    for &n in &[128usize, 1024, 8192] {
        let encoded = GlonassEncoder::encode_chunk(&multi_slot_samples(n)).unwrap();
        let enc_bytes = encoded.len() as u64;

        group.throughput(Throughput::Bytes(enc_bytes));
        group.bench_with_input(BenchmarkId::from_parameter(n), &encoded, |b, e| {
            b.iter(|| GlonassDecoder::decode_chunk(black_box(e)).unwrap())
        });
    }

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    for &n in &[128usize, 1024] {
        let samples = smooth_samples(n);
        let raw_bytes = (n * 31) as u64;

        group.throughput(Throughput::Bytes(raw_bytes));
        group.bench_with_input(BenchmarkId::from_parameter(n), &samples, |b, s| {
            b.iter(|| {
                let enc = GlonassEncoder::encode_chunk(black_box(s)).unwrap();
                GlonassDecoder::decode_chunk(black_box(&enc)).unwrap()
            })
        });
    }

    group.finish();
}

criterion_group!(
    encode_benches,
    bench_encode_smooth,
    bench_encode_constant,
    bench_encode_multi_slot,
);

criterion_group!(
    decode_benches,
    bench_decode_smooth,
    bench_decode_constant,
    bench_decode_multi_slot,
);

criterion_group!(roundtrip_benches, bench_roundtrip);

criterion_main!(encode_benches, decode_benches, roundtrip_benches);
