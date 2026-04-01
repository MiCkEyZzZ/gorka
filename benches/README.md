# Gorka Benchmarks — Baseline

Measured on: Intel Core i5-9400F (6C/6T, up to 4.1 GHz), x86_64
OS: Linux 6.1 (Debian)
Date: 2026-04-01
Rust: rustc 1.95.0-nightly (release build, criterion)

---

## encode/smooth

| N samples | Throughput | Time/iter |
| --------- | ---------- | --------- |
| 128       | ~820 MiB/s | ~4.6 µs   |
| 1024      | ~905 MiB/s | ~33.4 µs  |
| 8192      | ~921 MiB/s | ~263 µs   |

## encode/constant (best case)

| N samples | Throughput  | Time/iter |
| --------- | ----------- | --------- |
| 128       | ~2.10 GiB/s | ~1.30 µs  |
| 1024      | ~2.38 GiB/s | ~9.23 µs  |
| 8192      | ~2.50 GiB/s | ~70.1 µs  |

## encode/multi_slot

| N samples | Throughput | Time/iter |
| --------- | ---------- | --------- |
| 128       | ~440 MiB/s | ~7.49 µs  |
| 1024      | ~420 MiB/s | ~62.7 µs  |
| 8192      | ~406 MiB/s | ~519 µs   |

---

## decode/smooth

| N samples | Throughput  |
| --------- | ----------- |
| 128       | ~61 MiB/s   |
| 1024      | ~57.5 MiB/s |
| 8192      | ~57.1 MiB/s |

## decode/constant

| N samples | Throughput  |
| --------- | ----------- |
| 128       | ~72.8 MiB/s |
| 1024      | ~61.5 MiB/s |
| 8192      | ~60.5 MiB/s |

## decode/multi_slot

| N samples | Throughput  |
| --------- | ----------- |
| 128       | ~67.3 MiB/s |
| 1024      | ~65.3 MiB/s |
| 8192      | ~65.0 MiB/s |

---

## roundtrip (encode + decode)

| N samples | Throughput |
| --------- | ---------- |
| 128       | ~272 MiB/s |
| 1024      | ~286 MiB/s |

---

## bit-level primitives

### write_bits

| Bits | Throughput    |
| ---- | ------------- |
| 8    | ~34.7 M ops/s |
| 10   | ~36.0 M ops/s |
| 32   | ~21.9 M ops/s |
| 64   | ~9.38 M ops/s |

### write stream

| Pattern      | Throughput     |
| ------------ | -------------- |
| 8-bit writes | ~100–110 MiB/s |
| 1-bit writes | ~64–74 MiB/s   |

### read stream

| Pattern     | Throughput     |
| ----------- | -------------- |
| 8-bit reads | ~2.1–2.5 GiB/s |
| 1-bit reads | ~42–43 MiB/s   |

---

## core primitives

### delta_of_delta_i64

- ~1 ns per operation
- ~3.08 billion ops/sec (batch)

### zigzag encoding

- encode: ~1.8 billion ops/sec
- decode: ~1.3–1.75 billion ops/sec

---

## Notes

- Benchmarks use `criterion` with statistical analysis.
- Throughput for encode is measured on **raw input size**.
- Throughput for decode is measured on **compressed size**.
- Results depend on signal characteristics (smooth vs constant vs multi-slot).
