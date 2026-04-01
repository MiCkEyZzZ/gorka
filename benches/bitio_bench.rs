//! Benchmarks: низкоуровневые примитивы bit-IO и вычислений.
//!
//! Позволяет отслеживать базовую скорость:
//! - `BitWriter::write_bits` — запись произвольного числа бит
//! - `BitWriter::write_bits_signed` — zigzag + запись
//! - `delta_of_delta_i64` — вычисление DoD
//! - `encode_i64` / `decode_i64` — zigzag encoding
//!
//! Запуск:
//! ```zsh
//! cargo bench --bench bitio_bench
//! ```

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gorka::{decode_i64, delta_of_delta_i64, encode_i64, BitReader, BitWriter};

// ── BitWriter: write_bits
// ─────────────────────────────────────────────────────

fn bench_write_bits_aligned(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitio/write_bits");

    // 8-битная запись: типичный cn0_dbhz / slot-индекс
    group.throughput(Throughput::Elements(1));
    group.bench_function("8bit", |b| {
        b.iter(|| {
            let mut w = BitWriter::new();
            w.write_bits(black_box(0b1011_0101), 8).unwrap();
            w.finish()
        })
    });

    // 10-битная запись: pseudorange bucket '10' + 10b
    group.bench_function("10bit", |b| {
        b.iter(|| {
            let mut w = BitWriter::new();
            w.write_bits(black_box(0b10_0000_0001), 10).unwrap();
            w.finish()
        })
    });

    // 32-битная запись: doppler verbatim
    group.bench_function("32bit", |b| {
        b.iter(|| {
            let mut w = BitWriter::new();
            w.write_bits(black_box(1_200_500u64), 32).unwrap();
            w.finish()
        })
    });

    // 64-битная запись: timestamp verbatim / carrier phase verbatim
    group.bench_function("64bit", |b| {
        b.iter(|| {
            let mut w = BitWriter::new();
            w.write_bits(black_box(1_700_000_000_000u64), 64).unwrap();
            w.finish()
        })
    });

    group.finish();
}

/// Пишем N бит подряд, измеряем throughput в байтах сырых данных.
fn bench_write_bits_stream(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitio/write_stream");

    for &n_bits in &[128usize, 1024, 8192] {
        let n_bytes = (n_bits / 8) as u64;
        group.throughput(Throughput::Bytes(n_bytes));

        group.bench_with_input(BenchmarkId::new("8bit_each", n_bits), &n_bits, |b, &n| {
            b.iter(|| {
                let mut w = BitWriter::new();
                for i in 0..(n / 8) {
                    w.write_bits(black_box((i & 0xFF) as u64), 8).unwrap();
                }
                w.finish()
            })
        });

        group.bench_with_input(BenchmarkId::new("1bit_each", n_bits), &n_bits, |b, &n| {
            b.iter(|| {
                let mut w = BitWriter::new();
                for i in 0..n {
                    w.write_bit(black_box(i % 2 == 0));
                }
                w.finish()
            })
        });
    }

    group.finish();
}

// ── BitReader: read_bits
// ──────────────────────────────────────────────────────

fn bench_read_bits_stream(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitio/read_stream");

    for &n_bytes in &[16usize, 128, 1024] {
        let data: Vec<u8> = (0..n_bytes).map(|i| (i & 0xFF) as u8).collect();
        let n_bits = n_bytes as u64 * 8;

        group.throughput(Throughput::Bytes(n_bytes as u64));

        group.bench_with_input(BenchmarkId::new("8bit_each", n_bits), &data, |b, d| {
            b.iter(|| {
                let mut r = BitReader::new(black_box(d));
                let mut sum = 0u64;
                while r.bits_remaining() >= 8 {
                    sum = sum.wrapping_add(r.read_bits(8).unwrap());
                }
                sum
            })
        });

        group.bench_with_input(BenchmarkId::new("1bit_each", n_bits), &data, |b, d| {
            b.iter(|| {
                let mut r = BitReader::new(black_box(d));
                let mut count = 0u32;
                while r.bits_remaining() >= 1 {
                    if r.read_bit().unwrap() {
                        count += 1;
                    }
                }
                count
            })
        });
    }

    group.finish();
}

// ── delta_of_delta_i64
// ────────────────────────────────────────────────────────

/// Скорость вычисления одного DoD — ключевая операция в горячем пути encoder.
fn bench_delta_of_delta(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitives/delta_of_delta_i64");

    // Типичный случай: псевдодальность, медленный дрейф
    group.throughput(Throughput::Elements(1));
    group.bench_function("single", |b| {
        b.iter(|| {
            delta_of_delta_i64(
                black_box(21_500_000_222),
                black_box(21_500_000_000),
                black_box(222),
            )
        })
    });

    // Batch: 1024 последовательных DoD, измеряем throughput элементов
    let n = 1024usize;
    let values: Vec<i64> = (0..n).map(|i| 21_500_000_000 + i as i64 * 222).collect();

    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("batch_1024", |b| {
        b.iter(|| {
            let mut prev = values[0];
            let mut prev_d = 0i64;
            let mut checksum = 0i64;

            for &curr in &values[1..] {
                let dod = delta_of_delta_i64(curr, prev, prev_d);
                checksum = checksum.wrapping_add(dod);
                let delta = curr - prev;
                prev_d = delta;
                prev = curr;
            }
            checksum
        })
    });

    group.finish();
}

// ── zigzag encode/decode
// ──────────────────────────────────────────────────────

fn bench_zigzag(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitives/zigzag");
    group.throughput(Throughput::Elements(1));

    group.bench_function("encode_i64/zero", |b| {
        b.iter(|| encode_i64(black_box(0i64)))
    });
    group.bench_function("encode_i64/positive", |b| {
        b.iter(|| encode_i64(black_box(1_200_500i64)))
    });
    group.bench_function("encode_i64/negative", |b| {
        b.iter(|| encode_i64(black_box(-3_500_000i64)))
    });
    group.bench_function("decode_i64/zero", |b| {
        b.iter(|| decode_i64(black_box(0u64)))
    });
    group.bench_function("decode_i64/typical", |b| {
        b.iter(|| decode_i64(black_box(2_401_000u64)))
    });

    // Batch roundtrip: encode затем decode
    let n = 1024usize;
    let values: Vec<i64> = (0..n as i64).map(|i| i * 1000 - 512_000).collect();

    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("roundtrip_batch_1024", |b| {
        b.iter(|| {
            let mut checksum = 0i64;
            for &v in &values {
                checksum = checksum.wrapping_add(decode_i64(encode_i64(black_box(v))));
            }
            checksum
        })
    });

    group.finish();
}

// ── write_bits_signed (zigzag + write) ───────────────────────────────────────

fn bench_write_bits_signed(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitio/write_bits_signed");
    group.throughput(Throughput::Elements(1));

    // Покрываем все bucket-ширины из encoder:
    // 7b (timestamp bucket 2), 9b (timestamp/cn0), 10b (pr), 14b (doppler), 32b
    // (phase)
    for &(label, val, bits) in &[
        ("7b_zero", 0i64, 7u8),
        ("7b_small", 42i64, 7u8),
        ("9b", -255i64, 9u8),
        ("10b", 511i64, 10u8),
        ("14b", 8191i64, 14u8),
        ("32b", 2_147_483i64, 32u8),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| {
                let mut w = BitWriter::new();
                w.write_bits_signed(black_box(val), bits).unwrap();
                w.finish()
            })
        });
    }

    group.finish();
}

// ── регистрация
// ───────────────────────────────────────────────────────────────

criterion_group!(
    bitio_benches,
    bench_write_bits_aligned,
    bench_write_bits_stream,
    bench_read_bits_stream,
    bench_write_bits_signed,
);

criterion_group!(primitive_benches, bench_delta_of_delta, bench_zigzag,);

criterion_main!(bitio_benches, primitive_benches);
