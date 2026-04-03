# Changelog

All notable changes to **Gorka** are documented in this file.

## [Unreleased] — 0000-00-00

### Added

- **examples**
  - добавлен пример `basic_encode.rs` для проверки базового `encode` и `decode`
    с `GlonassSample`
  - добавлен пример `no_std_demo.rs` для использования `encode`/`decode` без
    стандартной библиотеки
  - добавлен пример `streaming.rs` для записи и чтения chunk-последовательностей
  - добавлен пример `compare.rs` для сравнения сжатия Gorka vs raw vs gzip

- **gnss / types**
  - добавлены методы-геттеры для удобного извлечения внутренних значений:
    - `Millimeter::as_i64()`
    - `MilliHz::as_i32()`

- **io**
  - добавлен модуль `io/mod.rs` для работы с chunk-последовательностями
    - `write_framed` / `read_framed` — запись и чтение length-prefixed фреймов
    - `ChunkWriter` — буферизованная запись последовательности chunk
    - `ChunkReader` — итератор по chunk-фреймам без копирования данных
    - поддержка проверки размера payload (`MAX_FRAME_PAYLOAD = 64 MiB`)
    - корректная обработка ошибок: `UnexpectedEof`, `ValueTooLarge`
    - roundtrip тесты write → read, multi-chunk последовательности
    - интеграция с `GlonassEncoder` / `GlonassDecoder`
  - модуль компилируется только при включённой `std`-фиче

- **docs**
  - добавил жокументацию по encoding

### Changed

- **.github**
  - обновил `CODEOWNERS` файл
- **benches**
  - обновил документацию по коду для `bitio_bench.rs` и `encode_bench.rs`
- **docs**
  - обновил `PROJECT_STRUCTURE.md`

### Notes

## [0.1.0] — 2026-03-31

### Added

- **docs**
  - добавлен документ спецификации формата `FORMAT.md` (Gorka Binary Format v1)

- **tests**
  - добавил интеграционные тесты для глонасс `glonass_sample.rs`
  - добавил интеграционные тесты для encoder `encoder_tests.rs`
  - добавил проперти тесты для codec `codec_property.rs`

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
  - добавлена базовая поддержка `no_std` (через `#![no_std]`)
  - core-модули (`bits`, `codec`, `gnss`, `error`) не зависят от `std`
  - `std` вынесен в feature-флаг (включён по умолчанию)
  - модуль `io` компилируется только при `feature = "std"`
  - подготовлена архитектура для будущей поддержки `alloc` и zero-allocation API

- **codec/encoder**
  - добавлена полноценная реализация `GlonassEncoder` для сжатия GNSS-наблюдений
    в бинарный chunk-формат

  - реализован stateful-encoder (`EncoderState`):
    - хранение предыдущих значений для всех полей (timestamp, slot, CN0,
      pseudorange, doppler, carrier phase)
    - отдельное состояние доплера для каждого GLONASS slot (FDMA-aware)
    - поддержка delta и delta-of-delta схем (timestamp, pseudorange, carrier phase)

  - реализована компрессия chunk:
    - запись заголовка через `VersionUtils::write_chunk_header`
    - первый sample кодируется в verbatim формате
    - последующие sample кодируются побитово через `BitWriter`

  - реализовано кодирование всех полей:
    - `timestamp` — delta-of-delta схема (4 bucket’а)
    - `slot` — 1-битный флаг + индекс (при изменении)
    - `cn0` — delta + ZigZag
    - `pseudorange` — delta-of-delta (mm, 3 bucket’а + verbatim)
    - `doppler` — per-slot delta схема с verbatim fallback (FDMA-aware)
    - `carrier_phase_cycles` — optional поле с поддержкой:
      - появления/потери сигнала
      - delta-of-delta кодирования
      - сброса состояния при больших скачках

  - реализованы ключевые свойства формата:
    - первый sample хранится полностью (verbatim)
    - все последующие кодируются относительно состояния
    - минимизация размера через bit-level encoding
    - поддержка отрицательных значений через ZigZag

  - добавлены вспомогательные функции:
    - `encode_verbatim`
    - `encode_delta`
    - `encode_timestamp`
    - `encode_slot`
    - `encode_cn0`
    - `encode_pseudorange`
    - `encode_doppler`
    - `encode_carrier_phase`

  - добавлены unit- и интеграционные тесты для encoder:
    - тесты заголовка (magic, version, count)
    - тесты verbatim-формата (размер, порядок полей)
    - тесты валидации входных данных (`InvalidSlot`, `EmptyChunk`)
    - тесты всех slot (-7..=6)
    - тесты multi-slot chunk
    - тесты carrier phase:
      - отсутствие
      - появление / потеря / повторное появление
    - тесты timestamp:
      - равномерный шаг (DoD = 0)
      - большие разрывы (verbatim fallback)
    - тесты коэффициента сжатия:
      - constant signal (≥8×)
      - smooth signal (≥3×)
    - тесты устойчивости к большим значениям и edge-case’ам

- **codec/decoder**
  - добавлена полноценная реализация `GlonassDecoder` для декодирования chunk’ов
  - реализован stateful-декодер (`DecoderState`), синхронизированный с encoder:
    - хранение предыдущих значений для всех полей (timestamp, slot, CN0,
      pseudorange, doppler, carrier phase)
    - отдельное состояние доплера для каждого GLONASS slot (FDMA-aware)
    - поддержка delta и delta-of-delta схем (timestamp, pseudorange, carrier phase)

  - реализована декомпрессия chunk:
    - чтение заголовка через `VersionUtils::read_chunk_version`
    - декодирование первого sample в verbatim формате
    - побитовое декодирование остальных sample через `BitReader`

  - реализовано декодирование всех полей:
    - `timestamp` — delta-of-delta схема (4 bucket’а)
    - `slot` — изменение через флаг + индекс
    - `cn0` — delta + ZigZag
    - `pseudorange` — delta-of-delta (mm, 4 bucket’а)
    - `doppler` — per-slot delta схема с verbatim fallback
    - `carrier_phase_cycles` — optional поле с поддержкой:
      - появления/потери сигнала
      - delta-of-delta кодирования
      - сброса состояния

  - обеспечена полная совместимость с encoder (roundtrip без потерь)

  - добавлены вспомогательные функции:
    - `decode_verbatim`
    - `decode_delta`
    - `decode_timestamp`
    - `decode_pseudorange`
    - `decode_doppler`
    - `decode_carrier_phase`

  - добавлены unit- и интеграционные тесты для decoder:
    - roundtrip тесты (1 / 10 / 100 samples)
    - тесты точности (1 mm / 1 mHz)
    - тесты отрицательных значений (doppler)
    - тесты всех slot (-7..=6)
    - тесты смены slot внутри chunk
    - тесты carrier phase:
      - отсутствие
      - постоянное значение
      - появление / потеря / повторное появление
    - тесты timestamp:
      - большие разрывы
      - нерегулярные интервалы
    - тесты ошибок:
      - `UnexpectedEof`
      - `InvalidMagic`
      - `InvalidVersion`

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

- **gorka**
  - обновил файл с командами для разработки `justfile`:
    - сделал команду запуска конкретного проперти теста по имени файла
    - сделал команду запуска сразу всех проперти тестов

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
- **codec**
  - encoder и decoder приведены к полной симметрии по битовому формату
  - уточнена семантика bit-stream (bucket’ы, DoD, verbatim fallback)
  - улучшена согласованность состояния между encode/decode (state machine)

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
