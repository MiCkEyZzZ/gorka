# Gorka — Glonass Time-Series Compression

**Gorka** is a Rust library for efficient compression and
decompression of GNSS/Glonass time-series data.

## Usage

```toml
[dependencies]
gorka = "0.1"
```

```rust
use gorka::bits::{BitWriter, BitReader};

let mut w = BitWriter::new();
w.write_bits(42, 8).unwrap();
let data = w.finish();

let mut r = BitReader::new(&data);
assert_eq!(r.read_bits(8).unwrap(), 42);
```

## Features

- Bit-level writing and reading (bits module)
- Signed and unsigned values
- Zigzag and delta encoding
- GNSS/Glonass sample support (`gnss` module)
- IO utilities (`io` module)
- Error handling (`error` module)

## Development

```zsh
just dev       # run formatting, lint and all tests
just fmt-all   # format Rust and TOML
just test-next # run tests via nextest
```

## License

MIT OR Apache-2.0
