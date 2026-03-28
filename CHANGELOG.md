# Changelog

All notable changes to **Gorka** are documented in this file.

## [Unreleased] — 00-00-0000

### Added

- **delta**
  - реализованы функции для вычисления дельт и дельт от дельт (`delta_i64`,
    `delta_of_delta_i64`, `delta_u64`, `delta_of_delta_u64`)
  - реализованы функции для восстановления значений из дельт (`reconstruct_from_delta`,
    `reconstruct_from_dod`, `reconstruct_from_dod_u64`)
  - добавлены unit-тесты для всех функций, включая roundtrip-тесты для проверки
    корректности вычислений

- **zigzag**
  - реализованы функции кодирования и декодирования со схемой ZigZag (`encode_i64`,
    `decode_i64`)
  - добавлены unit-тесты для проверки корректности кодирования/декодирования и roundtrip-тесты

- **bits**
  - базовая реализация `BitWriter` и `BitReader`
  - интеграционные тесты и property-тесты для bits

- **gnss**
  - добавлена структура `GlonassSample` в `glonass.rs`

- **errors**
  - базовая обработка ошибок в `error.rs`

- **.config**
  - `.config/nextest.toml` — конфиг для nextest (таймауты, retries)

- **tests**
  - интеграционные тесты для bits
  - property-тесты для bits (`tests/bit_property.rs`)

- **tooling / formatting**
  - `.editorconfig` — единый стиль редактора
  - `rustfmt.toml` — настройки форматирования Rust-кода
  - `taplo.toml` — единый стиль для Cargo.toml и workspace
  - `clippy.toml` — строгие правила линтинга
  - `deny.toml` — правила запрета небезопасных зависимостей, типов и методов
  - `rust-toolchain.toml` — фиксированный nightly для воспроизводимости

- **docs & project meta**
  - `CODE_OF_CONDUCT.md`
  - `CONTRIBUTING.md`
  - `SECURITY.md`
  - `BUGS` — файл с известными проблемами
  - `INSTALL` — инструкция по установке
  - `CHANGELOG.md` — текущий файл

- **build / dev tools**
  - `Justfile` — команды для форматирования, линтинга, тестов, bench, clean, dev
    workflow

### Changed

- **bits/writer**
  - метод `write_bits_signed` теперь использует `encode_i64` для записи signed
    значений через ZigZag кодирование
- **bits/reader**
  - метод `read_bits_signed` теперь использует `decode_i64` для корректного чтения
    signed значений с ZigZag декодированием

### Notes

- Проект на pre-0.1 версии — API активно меняется.
- Настроены property-тесты через `proptest` и интеграционные тесты через
  `cargo test`.
- CI можно запускать через `just check` или `just dev` для полного прогонов
  тестов + форматирования + линтера.
