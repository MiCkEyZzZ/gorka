# Gorka Benchmarks — Baseline

- Measured on: Intel Core i5-9400F (6C/6T, up to 4.1 GHz), x86_64
- OS: Linux 6.1 (Debian)
- Date: 2026-04-01
- Rust: rustc 1.95.0-nightly (release build, criterion)

---

## encode/smooth

| N samples | Throughput   | Time/iter |
| --------- | ------------ | --------- |
| 128       | ~1.037 GiB/s | ~3.56 µs  |
| 1024      | ~1.080 GiB/s | ~27.37 µs |
| 8192      | ~1.071 GiB/s | ~220.6 µs |

## encode/constant (best case)

| N samples | Throughput   | Time/iter |
| --------- | ------------ | --------- |
| 128       | ~1.557 GiB/s | ~1.76 µs  |
| 1024      | ~1.586 GiB/s | ~13.83 µs |
| 8192      | ~1.556 GiB/s | ~112.8 µs |

## encode/multi_slot

| N samples | Throughput | Time/iter |
| --------- | ---------- | --------- |
| 128       | ~738 MiB/s | ~4.46 µs  |
| 1024      | ~767 MiB/s | ~34.35 µs |
| 8192      | ~766 MiB/s | ~275.3 µs |

---

## decode/smooth

| N samples | Throughput |
| --------- | ---------- |
| 128       | ~161 MiB/s |
| 1024      | ~151 MiB/s |
| 8192      | ~149 MiB/s |

## decode/constant

| N samples | Throughput  |
| --------- | ----------- |
| 128       | ~69.7 MiB/s |
| 1024      | ~59.3 MiB/s |
| 8192      | ~57.3 MiB/s |

## decode/multi_slot

| N samples | Throughput |
| --------- | ---------- |
| 128       | ~232 MiB/s |
| 1024      | ~230 MiB/s |
| 8192      | ~228 MiB/s |

---

## roundtrip (encode + decode)

| N samples | Throughput |
| --------- | ---------- |
| 128       | ~532 MiB/s |
| 1024      | ~555 MiB/s |

---

## bit-level primitives

### write_bits

| Bits | Throughput   |
| ---- | ------------ |
| 8    | ~214.8 MiB/s |
| 9    | ~146.1 MiB/s |
| 16   | ~292.5 MiB/s |
| 32   | ~495.3 MiB/s |
| 64   | ~859.3 MiB/s |

### write stream

| Pattern      | Throughput |
| ------------ | ---------- |
| 8-bit writes | ~461 MiB/s |
| 1-bit writes | ~107 MiB/s |

### read stream

| Pattern     | Throughput   |
| ----------- | ------------ |
| 8-bit reads | ~1.074 GiB/s |
| 1-bit reads | ~81 MiB/s    |

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
