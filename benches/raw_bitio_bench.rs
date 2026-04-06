//! Benchmarks для RawBitWriter и BitReader.
//!
//! Запуск: `cargo bench --bench raw_bitio_bench`
//!
//! Измеряет throughput в МБ/с для write/read 1M значений разной ширины.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gorka::{BitReader, BitWrite, RawBitWriter};

// BitWriter benchmarks

fn bench_write_bits(c: &mut Criterion) {
    let mut group = c.benchmark_group("RawBitWriter/write_bits");

    for &n in &[1u8, 3, 7, 8, 9, 16, 32, 64] {
        let value = (1u64 << n.min(63)) - 1;
        let iters = 1_000_000usize;
        let total_bits = iters * n as usize;
        #[allow(clippy::manual_div_ceil)]
        let total_bytes = (total_bits + 7) / 8;

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{n}b")), &n, |b, &n| {
            b.iter(|| {
                let mut buf = vec![0u8; total_bytes];
                let mut w = RawBitWriter::new(&mut buf);

                for _ in 0..iters {
                    w.write_bits(black_box(value), n).unwrap();
                }

                let bytes = w.bytes_written();

                black_box(&buf[..bytes]);
            });
        });
    }

    group.finish();
}

fn bench_write_bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("RawBitWriter/write_bit");
    let iters = 1_000_000usize;
    #[allow(clippy::manual_div_ceil)]
    let total_bytes = (iters + 7) / 8;

    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.bench_function("1b", |b| {
        b.iter(|| {
            let mut buf = vec![0u8; total_bytes];
            let mut w = RawBitWriter::new(&mut buf);

            for i in 0..iters {
                w.write_bit(black_box(i % 2 == 0)).unwrap();
            }

            let bytes = w.bytes_written();

            black_box(&buf[..bytes]);
        });
    });

    group.finish();
}

fn bench_write_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("RawBitWriter/mixed_encoder_profile");

    let patterns: &[(u64, u8)] = &[(0, 1), (0, 1), (0, 1), (0, 1), (0b10, 2), (0b00, 2)];
    let iters = 100_000usize;
    let bits_per_iter: usize = patterns.iter().map(|(_, n)| *n as usize).sum();
    #[allow(clippy::manual_div_ceil)]
    let total_bytes = (iters * bits_per_iter + 7) / 8;

    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.bench_function("constant_signal", |b| {
        b.iter(|| {
            let mut buf = vec![0u8; total_bytes];
            let mut w = RawBitWriter::new(&mut buf);

            for _ in 0..iters {
                for &(val, n) in patterns {
                    w.write_bits(black_box(val), n).unwrap();
                }
            }

            let bytes = w.bytes_written();

            black_box(&buf[..bytes]);
        });
    });

    group.finish();
}

// BitReader benchmarks

fn make_buf(n_bits: usize) -> Vec<u8> {
    #[allow(clippy::manual_div_ceil)]
    let bytes = (n_bits + 7) / 8;

    (0..bytes).map(|i| i as u8).collect()
}

fn bench_read_bits(c: &mut Criterion) {
    let mut group = c.benchmark_group("BitReader/read_bits");

    for &n in &[1u8, 3, 7, 8, 9, 16, 32, 64] {
        let iters = 1_000_000usize;
        let total_bits = iters * n as usize;
        #[allow(clippy::manual_div_ceil)]
        let total_bytes = (total_bits + 7) / 8;
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

    #[allow(clippy::manual_div_ceil)]
    group.throughput(Throughput::Bytes(((iters + 7) / 8) as u64));
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
    let mut group = c.benchmark_group("RawBitWriter+BitReader/roundtrip");
    let iters = 100_000usize;
    let data: Vec<u8> = (0..iters as u64)
        .map(|i| ((i * 6364136223846793005) >> 56) as u8)
        .collect();
    let total_bytes = iters;

    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.bench_function("8b", |b| {
        b.iter(|| {
            let mut buf = vec![0u8; total_bytes];
            let mut w = RawBitWriter::new(&mut buf);

            for &v in &data {
                w.write_bits(v as u64, 8).unwrap();
            }

            let bytes_written = w.bytes_written();
            let mut r = BitReader::new(&buf[..bytes_written]);
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
