# Changelog

All notable changes to **Gorka** are documented in this file.

## [Unreleased] — 00-00-0000

### Added

- **no_std**
  - добавлена базовая поддержка `no_std` (через `#![no_std]`)
  - core-модули (`bits`, `codec`, `gnss`, `error`) не зависят от `std`
  - `std` вынесен в feature-флаг (включён по умолчанию)
  - модуль `io` компилируется только при `feature = "std"`
  - подготовлена архитектура для будущей поддержки `alloc` и zero-allocation API

- **codec/format**
  - Добавлена структура `CompatibilityInfo` с методами проверки совместимости
    (`check`) между версиями формата.
  - Добавлены методы в `FormatVersion`:
    - `can_read` / `can_write` для проверки совместимости версий
    - `is_deprecated` для отметки устаревших версий
    - `description` для краткого текстового описания версии

  - Добавлен вспомогательный модуль `VersionUtils`:
    - `read_chunk_version` — чтение версии формата из заголовка chunk
    - `write_chunk_header` — запись заголовка chunk с версией и количеством sample

  - Добавлены unit-тесты для:
    - проверки совместимости версий (`test_compatibility_info`)
    - roundtrip чтения/записи заголовка (`test_chunk_header_roundtrip`)
    - проверки обработки некорректного magic (`test_invalid_magic`)

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
  - `FORMAT.md` - текущая версия формата

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
- **gnss**
  - метод `validate_slot` изменил логику работы, теперь он корректно обрабатывает
    ошибку
- **error**
  - в enum `GorkaError` добавил дополнительную обработку ошибок для версии формата:
    `InvalidVersion`, `InvalidMagic`

### Notes

- Проект на pre-0.1 версии — API активно меняется.
- Настроены property-тесты через `proptest` и интеграционные тесты через
  `cargo test`.
- CI можно запускать через `just check` или `just dev` для полного прогонов
  тестов + форматирования + линтера.
- Начата работа по поддержке embedded-сценариев (`no_std`).
- В текущей версии некоторые компоненты (например, `BitWriter`) всё ещё используют
  аллокации и будут переработаны в будущем.
