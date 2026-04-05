//! Benchmarks для BitWriter и BitReader (GORKA-13).
//!
//! Запуск: `cargo bench --bench bitio_bench`
//!
//! Измеряет throughput в МБ/с для write/read 1M значений разной ширины.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gorka::{BitReader, BitWriter};

// BitWriter benchmarks

fn bench_write_bits(c: &mut Criterion) {
    let mut group = c.benchmark_group("BitWriter/write_bits");

    for &n in &[1u8, 3, 7, 8, 9, 16, 32, 64] {
        let value = (1u64 << n.min(63)) - 1;
        let iters = 1_000_000usize;
        let total_bits = iters * n as usize;
        let total_bytes = total_bits.div_ceil(8);

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{n}b")), &n, |b, &n| {
            b.iter(|| {
                let mut w = BitWriter::new();
                for _ in 0..iters {
                    w.write_bits(black_box(value), n).unwrap();
                }
                black_box(w.finish())
            });
        });
    }

    group.finish();
}

fn bench_write_bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("BitWriter/write_bit");
    let iters = 1_000_000usize;

    group.throughput(Throughput::Bytes((iters.div_ceil(8)) as u64));
    group.bench_function("1b", |b| {
        b.iter(|| {
            let mut w = BitWriter::new();
            for i in 0..iters {
                w.write_bit(black_box(i % 2 == 0));
            }
            black_box(w.finish())
        });
    });

    group.finish();
}

fn bench_write_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("BitWriter/mixed_encoder_profile");

    let patterns: &[(u64, u8)] = &[(0, 1), (0, 1), (0, 1), (0, 1), (0b10, 2), (0b00, 2)];
    let iters = 100_000usize;
    let bits_per_iter: usize = patterns.iter().map(|(_, n)| *n as usize).sum();
    let total_bytes = (iters * bits_per_iter).div_ceil(8);

    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.bench_function("constant_signal", |b| {
        b.iter(|| {
            let mut w = BitWriter::new();
            for _ in 0..iters {
                for &(val, n) in patterns {
                    w.write_bits(black_box(val), n).unwrap();
                }
            }
            black_box(w.finish())
        });
    });

    group.finish();
}

// BitReader benchmarks

fn make_buf(n_bits: usize) -> Vec<u8> {
    let bytes = n_bits.div_ceil(8);
    (0..bytes).map(|i| i as u8).collect()
}

fn bench_read_bits(c: &mut Criterion) {
    let mut group = c.benchmark_group("BitReader/read_bits");

    for &n in &[1u8, 3, 7, 8, 9, 16, 32, 64] {
        let iters = 1_000_000usize;
        let total_bits = iters * n as usize;
        let total_bytes = total_bits.div_ceil(8);
        let buf = make_buf(total_bits + 64);

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{n}b")), &n, |b, &n| {
            b.iter(|| {
                let mut r = BitReader::new(&buf);
                let mut sum = 0u64;
                for _ in 0..iters {
                    sum ^= r.read_bits(n).unwrap();
                }
                black_box(sum)
            });
        });
    }

    group.finish();
}

fn bench_read_bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("BitReader/read_bit");
    let iters = 1_000_000usize;
    let buf = make_buf(iters + 64);

    group.throughput(Throughput::Bytes(iters.div_ceil(8) as u64));
    group.bench_function("1b", |b| {
        b.iter(|| {
            let mut r = BitReader::new(&buf);
            let mut sum = 0u64;
            for _ in 0..iters {
                sum ^= r.read_bit().unwrap() as u64;
            }
            black_box(sum)
        });
    });

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("BitWriter+Reader/roundtrip");
    let iters = 100_000usize;
    let data: Vec<u8> = (0..iters as u64)
        .map(|i| ((i * 6364136223846793005) >> 56) as u8)
        .collect();

    group.throughput(Throughput::Bytes(iters as u64));
    group.bench_function("8b", |b| {
        b.iter(|| {
            let mut w = BitWriter::new();
            for &v in &data {
                w.write_bits(v as u64, 8).unwrap();
            }
            let buf = w.finish();

            let mut r = BitReader::new(&buf);
            let mut sum = 0u64;
            for _ in 0..iters {
                sum ^= r.read_bits(8).unwrap();
            }
            black_box(sum)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_write_bits,
    bench_write_bit,
    bench_write_mixed,
    bench_read_bits,
    bench_read_bit,
    bench_roundtrip,
);
criterion_main!(benches);
