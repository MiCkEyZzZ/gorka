# Changelog

All notable changes to **Gorka** are documented in this file.

## [Unreleased] — 00-00-0000

### Added

- **tests**
  - добавил интеграционные тесты для глонасс `glonass_sample.rs`

- **gorka/src/types**
  - добавил общие типы для удобаства использования `GlonassSample`, `GpsSample`,
    `GalileoSample`, `BeidouSample`

- **gnss**
  - добавлена начальная реализация `GnssFrame` — контейнера фиксированной
    вместимости для наблюдений GLONASS (`MAX_GLONASS_SATS = 14`)
    - поддерживаются:
      - добавление наблюдений через `push()` с валидацией (slot, timestamp,
        вместимость, уникальность)
      - создание фрейма из среза через `from_samples()`
      - итерация по наблюдениям через `iter()`
      - доступ по слоту через `get_by_slot()`
      - проверка наличия слота через `contains_slot()`
    - гарантируется:
      - все наблюдения имеют одинаковый `timestamp_ms` (эпоха фрейма)
      - отсутствие дублирующихся слотов
      - соблюдение ограничения вместимости
    - наблюдения автоматически поддерживаются отсортированными по slot (по возрастанию)
    - добавлен метод `validate_all()` для полной проверки всех наблюдений
    - добавлены unit-тесты, покрывающие:
      - логику `push()` (валидные и невалидные сценарии)
      - обнаружение дублирующихся слотов
      - обработку несовпадения timestamp
      - переполнение фрейма (`FrameFull`)
      - корректность сортировки
      - работу методов доступа и итерации
  - добавлено поле `carrier_phase_cycles: Option<i64>` для хранения накопленной
    фазы несущей (фиксированная точка, 2⁻³² cycles)
  - добавлены методы:
    - `validate()` — комплексная проверка всех полей
    - `validate_pseudorange()`
    - `validate_doppler()`
    - `carrier_freq_mhz()` — вычисление частоты несущей по FDMA slot
    - `is_tracked()` — проверка наличия сигнала (по CN0)
  - добавлены константы физически допустимых диапазонов:
    - `PSEUDORANGE_MIN_MM`, `PSEUDORANGE_MAX_MM`
    - `DOPPLER_MAX_MHZ`
    - `CN0_MIN_TRACKED`
    - `BASE_FREQ_MHZ`, `FREQ_STEP_MHZ`
    - `SLOT_MIN`, `SLOT_MAX`
  - добавлены unit-тесты для:
    - валидации диапазонов (slot, pseudorange, doppler)
    - расчёта частоты несущей
    - логики `is_tracked`
    - проверки точности (1 mm / 1 mHz)
  - добавлены newtype-структуры для точной физической модели:
    - `MilliHz(pub i32)` — миллигерцы для Doppler (повышенная точность и расширенный
      диапазон)
    - `Millimeter(pub i64)` — миллиметры для псевдодальности (повышенная точность,
      поддержка отрицательных значений)
    - newtype используются в `GlonassSample` для хранения `doppler_millihz` и `pseudorange_mm`

- **no_std**
  - добавил базовую реализацию encoder
  - добавил базовую реализацию decoder
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
  - добавил дополнительные ошибки: `InvalidPseudorange`, `InvalidDoppler`,
    `TimestampMismatch`, `DuplicateSlot`, `FrameFull` и для проверки логики работы
    покрыл тестами
  - **gnss (BREAKING)**
    - поле `pseudorange_m: u32` заменено на `pseudorange_mm: i64`
      - увеличена точность до миллиметров
      - добавлена поддержка отрицательных значений (для валидации)
    - поле `doppler_hz: i16` заменено на `doppler_mhz: i32`
      - увеличена точность до миллигерц (mHz)
      - расширен диапазон значений
    - метод `validate_slot()` больше не является единственным методом валидации;
      добавлен `validate()` для полной проверки структуры
    - обновлена семантика комментариев и единиц измерения (Hz → mHz, m → mm)
  - добавил два новых типа ошибок: `InvalidPrn`, `InvalidCn0`

### Notes

- Проект на pre-0.1 версии — API активно меняется.
- Настроены property-тесты через `proptest` и интеграционные тесты через
  `cargo test`.
- CI можно запускать через `just check` или `just dev` для полного прогонов
  тестов + форматирования + линтера.
- Начата работа по поддержке embedded-сценариев (`no_std`).
- В текущей версии некоторые компоненты (например, `BitWriter`) всё ещё используют
  аллокации и будут переработаны в будущем.
- Переход на фиксированную точку (`mm`, `mHz`) устраняет ошибки округления
  и делает формат пригодным для точной GNSS-обработки и сжатия.
