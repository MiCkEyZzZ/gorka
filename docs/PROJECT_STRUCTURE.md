# Project structure

```text
gorka
├── .config
│   └── nextest.toml
├── .github
│   ├── ISSUE_TEMPLATE
│   │   └── enhancement.yml
│   ├── workflows
│   │   └── semantic-pull-request.yml
│   ├── CODEOWNERS
│   └── pull_request_template.md
├── benches
│   ├── bitio_bench.rs
│   ├── encode_bench.rs
│   └── README.md
├── docs
│   ├── API.md
│   ├── ARCHITECTURE.md
│   ├── BENCHMARKS.md
│   ├── ENCODING.md
│   ├── FORMAT.md
│   ├── SECURITY_MODEL.md
│   ├── DECODER.md
│   ├── PROJECT_STRUCTURE.md
│   └── TESTING.md
├── examples
│   ├── edge_cases.rs
│   └── encode_decode.rs
├── src
│   ├── bits
│   │   ├── mod.rs
│   │   ├── reader.rs
│   │   └── writer.rs
│   ├── codec
│   │   ├── format
│   │   │   ├── mod.rs
│   │   │   └── version.rs
│   │   ├── decoder.rs
│   │   ├── delta.rs
│   │   ├── encoder.rs
│   │   ├── mod.rs
│   │   └── zigzag.rs
│   ├── gnss
│   │   ├── frame.rs
│   │   ├── glonass.rs
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── io
│   │   └── mod.rs
│   ├── error.rs
│   └── lib.rs
├── tests
│   ├── bit_property.rs
│   ├── codec_property.rs
│   ├── compression_ratio.rs
│   ├── encoder_tests.rs
│   ├── glonass_sample.rs
│   └── test_bitstream.rs
├── .gitignore
├── .editorconfig
├── AUTHOR.md
├── BUGS
├── Cargo.lock
├── Cargo.toml
├── CHANGELOG.md
├── clippy.toml
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── deny.md
├── INSTALL
├── justfile
├── LICENSE.APACHE
├── LICENSE.MIT
├── README.md
├── rust-toolchain.toml
├── rustfmt.toml
├── SECURITY.md
└── taplo.toml
```
