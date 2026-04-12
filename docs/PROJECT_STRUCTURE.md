# Project structure

```text
gorka
├── .config
│   └── nextest.toml
├── .github
│   ├── ISSUE_TEMPLATE
│   │   ├── bug_report.yml
│   │   ├── config.yml
│   │   ├── enhancement.yml
│   │   └── other_stuff.yml
│   ├── workflows
│   │   ├── ci.yml
│   │   └── semantic-pull-request.yml
│   ├── CODEOWNERS
│   └── pull_request_template.md
├── benches
│   ├── decoder_bench.rs
│   ├── encode_bench.rs
│   ├── raw_bitio_bench.rs
│   ├── README.md
│   └── stream_bench.rs
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
│   ├── basic_encode.rs
│   ├── compare.rs
│   ├── no_std_demo_raw.rs
│   ├── stream_basic.rs
│   ├── stream_performance.rs
│   └── streaming.rs
├── src
│   ├── bits
│   │   ├── mod.rs
│   │   ├── raw_writer.rs
│   │   └── reader.rs
│   ├── codec
│   │   ├── cdma.rs
│   │   ├── codec_trait.rs
│   │   ├── delta.rs
│   │   ├── fdma.rs
│   │   ├── mod.rs
│   │   ├── registry.rs
│   │   └── zigzag.rs
│   ├── domain
│   │   ├── gnss
│   │   │   ├── constellation.rs
│   │   │   ├── frame.rs
│   │   │   ├── identifiers.rs
│   │   │   ├── measurement.rs
│   │   │   └── mod.rs
│   │   ├── signal
│   │   │   ├── cn0.rs
│   │   │   └── mod.rs
│   │   ├── units
│   │   │   ├── hertz.rs
│   │   │   ├── millihz.rs
│   │   │   ├── millimeter.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── io
│   │   └── mod.rs
│   ├── pipeline
│   │   ├── decoder.rs
│   │   ├── encoder.rs
│   │   ├── mod.rs
│   │   └── stream.rs
│   ├── wire
│   │   ├── compatibility.rs
│   │   ├── format.rs
│   │   ├── header.rs
│   │   └── mod.rs
│   ├── error.rs
│   ├── lib.rs
│   └── prelude.rs
├── tests
│   ├── bit_raw_property.rs
│   ├── codec_property.rs
│   ├── compression_ratio.rs
│   ├── encoder_tests.rs
│   ├── glonass_sample.rs
│   └── test_raw_bitwriter.rs
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
