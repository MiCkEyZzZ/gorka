# Changelog

All notable changes to **Gorka** are documented in this file.

## [Unreleased] — 0000-00-00

### Added

- **.github**
  - добавлен шаблон issue `Bug report` для удобного оформления сообщений о багах:
    - поля: `description`, `steps`, `environment`
    - пояснения по заполнению и placeholders для примеров
  - добавлен файл `config.yml`:
    - `blank_issues_enabled: false`
    - `contact_links`:
      - **Gorka Community Pachca** — [join link](https://app.pachca.com/join?invite_token=EakW3nIgma_ezqFc&company_name=Gorka+Community&company_id=416542), Get free help from the Gorka community
      - **Gorka Community Discussion** — [GitHub Discussions](https://github.com/MiCkEyZzZ/gorka/discussions), Get free help from the Gorka community
  - добавлен файл `other_stuff.yml`:
    - шаблон **Other**:
      - `name: Other`
      - `about: Can't find the right issue type? Use this one!`
      - `title: ""`
      - `labels: ""`
      - `assignees: ""`

- **gnss/types**
  - добавлены тесты для MilliHz::as_hz и Millimeter::as_m
  - добавлены тесты упорядочивания для MilliHz и Millimeter
  - добавлены явные граничные тесты для идентификаторов спутников
  - добавлены тесты отрицательных значений для Millimeter
  - улучшена общая устойчивость инвариантов на уровне типов

- **gnss/beidou**:
  - добавлена реализация `BeidouSample`:
    - структура `BeidouSample` с полями: `timestamp_ms`, `prn`, `cn0_dbhz`,
      `pseudorange_mm`, `doppler_millihz`, `carrier_phase_cycles`.
    - константы частот: `BDS_B1I_FREQ`, `BDS_B1C_FREQ`, `BDS_B2A_FREQ`.
    - методы валидации:
      - `validate_prn()`, `validate_pseudorange()`, `validate_doppler()`
      - комплексная проверка `validate()`, булевы методы `is_valid_*()`.
    - реализация трейта `GnssMeasurement`.
    - unit-тесты:
      - проверка корректности PRN, псевдодальности, допплера.
      - boundary cases для PRN, псевдодальности и допплера.
      - проверка метода `satellite_id()` и соответствия `ConstellationType::Beidou`.
      - проверка корректности частот (`BDS_B1I_FREQ`, `BDS_B1C_FREQ`, `BDS_B2A_FREQ`).

- **gnss/galileo**:
  - добавлена реализация `GalileoSample`:
    - структура `GalileoSample` с полями: `timestamp_ms`, `svn`, `cn0_dbhz`,
      `pseudorange_mm`, `doppler_millihz`, `carrier_phase_cycles`.
    - константы частот: `GAL_E1_FREQ`, `GAL_E5A_FREQ`, `GAL_E5B_FREQ`.
    - методы валидации:
      - `validate_svn()`, `validate_pseudorange()`, `validate_doppler()`
      - комплексная проверка `validate()`, булевы методы `is_valid_*()`.

    - реализация трейта `GnssMeasurement`.
    - unit-тесты:
      - проверка корректности SVN, псевдодальности, допплера.
      - boundary cases для SVN, псевдодальности и допплера.
      - проверка методов `satellite_id()` и соответствия `ConstellationType::Galileo`.
      - проверка корректности частот (`GAL_E1_FREQ`, `GAL_E5A_FREQ`, `GAL_E5B_FREQ`).

- **gnss**:
  - добавлены новые unit-тесты для всех конструкторов и методов, включая `SatelliteId`
    и `ConstellationType`.
  - добавлены edge-case тесты для граничных значений всех созвездий и их идентификаторов.

- **gnss/gps**:
  - добавлена базовая реализация `GpsSample`:
    - структура `GpsSample` с полями: `timestamp_ms`, `prn`, `cn0_dbhz`, `pseudorange_mm`,
      `doppler_millihz`, `carrier_phase_cycles`.
    - константы частот: `GPS_L1_FREQ`, `GPS_L2_FREQ`.
    - методы валидации:
      - `validate_prn()`, `validate_pseudorange()`, `validate_doppler()`
      - комплексная проверка `validate()`, булевы методы `is_valid_*()`.

    - реализация трейта `GnssMeasurement`.
    - unit-тесты для:
      - проверки корректности PRN, псевдодальности, допплера.
      - boundary cases для PRN, псевдодальности и допплера.
      - проверки корректности частот (`GPS_L1_FREQ`, `GPS_L2_FREQ`).
      - проверки методов `satellite_id()` и соответствия `ConstellationType::Gps`.

- **gnss/types**:
  - добавлены и/или задокументированы newtype и идентификаторы:
    - `Millimeter`, `MilliHz`, `Hertz`, `GpsPrn`, `GalSvn`, `BdsPrn`, `GloSlot`,
      `DbHz`.
  - методы-конструкторы (`new()`) проверяют диапазон и возвращают `GorkaError`
    при некорректном значении.
  - добавлены геттеры для безопасного извлечения внутренних значений.
  - использование integer newtype вместо float обеспечивает точность и предотвращает
    потери при арифметике.

- **gnss/types**:
  - Добавлены newtype для единиц измерения и GNSS идентификаторов:
    - `Millimeter` — расстояние в миллиметрах (`i64`), с методами `new()` и `as_i64()`.
    - `MilliHz` — частота в миллигерцах (`i32`), с методами `new()`, `as_i32()`
      и `abs()`.
    - `Hertz` — частота в герцах (`i64`).
    - `GpsPrn` — идентификатор GPS спутника (`u8`), с `MIN/MAX`, `new()` и `get()`.
    - `GalSvn` — идентификатор Galileo спутника (`u8`), с `get()`.
    - `BdsPrn` — идентификатор Beidou спутника (`u8`), с `get()`.
    - `GloSlot` — слот GLONASS (`i8`), с `MIN/MAX`, `new()` и `get()`.
    - `DbHz` — интенсивность сигнала (`u8`), с `get()`.
  - Методы-конструкторы (`new()`) проверяют диапазон и возвращают `GorkaError`
    при некорректном значении.
  - Все newtype имеют геттеры для безопасного извлечения внутреннего значения.
  - Использование integer newtype вместо `f64` / float обеспечивает точность и
    предотвращает потери при арифметике и сравнении.

- **gnss**:
  - В `constellation.rs` добавлены новые unit-тесты для максимальной надёжности:
    - Проверка всех конструкторов `SatelliteId::glonass()`, `gps()`, `galileo()`,
      `beidou()`.
    - Проверка методов `.constellation()`, `.glonass_slot()`, `.to_wire()`.
    - Проверка формата отображения `Display` для всех созвездий и спутников.
    - Проверка порядка сортировки `ConstellationType::order()`.
    - Тесты для крайних значений слотов GLONASS (-7..=6) и всех поддерживаемых PRN/SVN/BDS.
    - Дополнительные edge-case тесты для `to_wire()` и форматирования `GLO/GPS/GAL/BDS`.

  - Добавлены newtype и enum для унифицированного представления спутников:
    - `SatelliteId` — уникальный идентификатор спутника в созвездии (Glonass, GPS,
      Galileo, Beidou).
    - `ConstellationType` — тип созвездия, с методами:
      - `abbrev()` — краткий ASCII идентификатор ("GLO", "GPS", "GAL", "BDS").
      - `is_fdma()` — true для GLONASS (FDMA).
      - `order()` — порядок сортировки созвездий.
    - Методы-конструкторы: `SatelliteId::glonass()`, `gps()`, `galileo()`, `beidou()`.
    - Вспомогательные методы:
      - `SatelliteId::constellation()`
      - `SatelliteId::glonass_slot()`
      - `SatelliteId::to_wire()` — сериализация для wire формата.
    - Реализованы `Display` для удобного форматирования спутников и созвездий.
  - Юнит-тесты для всех методов, конструкторов и отображения (`Display`, `to_wire()`,
    `order`).

### Changed

- обновлён `README.md` файл удалена устаревшая информация и были внесены правки
  в примеры где использовались старые типы `i8`, `u8` они были заменены на новые
  `DbHz`, `GloSlot`

- **io**
  - в файле `mod.rs` обновлены тесты, где использовались старые
    типы `i8`, `u8` они были заменены на новые `DbHz`, `GloSlot`, которые были
    определённые newtype из `types.rs` для защиты от
    некорректных значений.

- **gnss**
  - рефакторинг `glonass.rs` для приведения к единому стилю и унификации функционала
    с другими созвездиями: `gps`, `galileo` и `beidou`.

- **gnss**
  - в `frame.rs` был произведён рефакторинг всех ф-й, где использовались старые
    типы `i8`, `u8` они были заменены на новые `DbHz`, `GloSlot`, которые были
    определённые newtype из `types.rs` для защиты от
    некорректных значений.
  - обновлены все тесты

- **tests**
  - были обновлены тесты под использование новой реализации writer:
    `codec_property.rs`, `compression_ratio.rs`, `encoder_tests.rs`
    `glonass_sample.rs`

- **bits/rider**
  - заменена старая реализация `BitWriter` на новую реализацию `RawBitWriter`
  - старая реализация была удалена из `bits/mod.rs` и `lib.rs`.

- **codec/stream**
  - произведён рефакторинг функций, использующих старую реализацию `BitWriter`,
    включая обновление тестов.
  - в `EncoderState` изменены типы некоторых полей для строгой типизации:
    - `last_slot`: **i8 → GloSlot**
    - `last_cn0`: **u8 → DbHz**
      Используются заранее определённые newtype из `types.rs` для защиты от
      некорректных значений.
  - в `StateSnapshot` изменены типы некоторых полей для строгой типизации:
    - `last_slot`: **i8 → GloSlot**
    - `last_cn0`: **u8 → DbHz**
      Используются заранее определённые newtype из `types.rs` для защиты от
      некорректных значений.
  - обновлены тесты

- **codec/encoder**
  - заменена старая реализация `BitWriter` на новую zero-copy `RawBitWriter`.
  - произведён рефакторинг функций, использующих старую реализацию `BitWriter`,
    включая обновление тестов.
  - в `EncoderState` изменены типы некоторых полей для строгой типизации:
    - `last_slot`: **i8 → GloSlot**
    - `last_cn0`: **u8 → DbHz**
      Используются заранее определённые newtype из `types.rs` для защиты от
      некорректных значений.
  - обновлены тесты

- **codec/decoder**
  - произведён рефакторинг функций, использующих старую реализацию `BitWriter`,
    включая обновление тестов.
  - в `DecoderState` изменены типы некоторых полей для строгой типизации:
    - `last_slot`: **i8 → GloSlot**
    - `last_cn0`: **u8 → DbHz**
      Используются заранее определённые newtype из `types.rs` для защиты от
      некорректных значений.
  - обновлены тесты

- **gnss/types**
  - `GalSvn::new()` теперь корректно возвращает `Err(GorkaError::InvalidSvn(_))`
    для некорректных значений.
  - удалён костыльный конструктор `from_raw()` для тестов; тесты теперь используют
    безопасный конструктор `new()`.
  - добавлен unit-тест `test_gal_svn_invalid` для проверки невозможности создания
    некорректного SVN.

- **gnss**:
  - в `types.rs` расширены newtype для единиц измерений и GNSS идентификаторов:
    - `Millimeter` — расстояние в миллиметрах (i64)
    - `MilliHz` — частота в миллигц (i32)
    - `Hertz` — частота в герцах (i64)
    - `GpsPrn` — идентификатор GPS спутника (u8), с `MIN/MAX` и методом-конструктором
      `new()`
    - `GalSvn` — идентификатор Galileo спутника (u8)
    - `BdsPrn` — идентификатор Beidou спутника (u8)
    - `GloSlot` — слот GLONASS (i8), с `MIN/MAX` и методом-конструктором `new()`
    - `DbHz` — интенсивность сигнала в dBHz (u8)
  - добавлены вспомогательные методы:
    - `.new()` и `.as_i64()/as_i32()/get()` для всех newtype
    - `MilliHz::abs()` для получения абсолютного значения

## Remove

- **bits**
  - удалена старая реализация `writer.rs`

- **benches**
  - удалены лишние бенчмарки `bitio_bench.rs`

- **examples**
  - удалены примеры для старой рализации writer: `no_std_demo.rs`

- **tests**
  - удалены тесты для проверки старой реализации writer: `test_bitstream.rs`
  - удалены проперти тесты для проверки старой реализации writer: `bit_property.rs`

## [v0.4.1] — 2026-04-06

### Added

- Улучшена документация (`rustdoc`) для модуля `gnss`:
  - добавлено описание архитектуры модуля (`mod.rs`), включая разбиение на подмодули
    и дизайн-цели (`no_std`, integer-first подход)
  - задокументированы newtype-структуры `Millimeter` и `MilliHz`:
    - назначение и единицы измерения
    - диапазоны значений (`i64` / `i32`)
    - добавлены примеры использования (`# Example`)
    - добавлены предупреждения о нецелевом использовании (где применимо)

  - расширена документация `GlonassSample`:
    - описание структуры (FDMA, fixed-point модель)
    - документированы все поля с диапазонами значений
    - добавлены `# Errors` и `# Example` для публичных методов

  - улучшена документация `GnssFrame`:
    - описаны инварианты (одинаковый timestamp, уникальность slot, сортировка)
    - задокументированы ограничения вместимости (`MAX_GLONASS_SATS`)
    - добавлены `# Errors` / `# Example` для публичных API (`push`, `from_samples`,
      `validate_all` и др.)
    - кратко задокументированы приватные методы

### Changed

- Исправлены intra-doc ссылки для совместимости с `rustdoc -D warnings`:
  - устранены `broken_intra_doc_links` ошибки
  - ссылки приведены к корректному scope (включая fully-qualified пути там, где
    необходимо)

## [0.4.0] — 2026-04-06

### ⚠️ Deprecation Notice

`BitWriter` (Vec-backed) is **deprecated** starting from this release.

It remains fully functional in v0.4.0 — existing code compiles without changes,
but will produce a compiler warning (`#[deprecated]`).

**Migration path:**

```rust
// Before (v0.3, deprecated)
use gorka::BitWriter;
let mut w = BitWriter::new();
w.write_bits(0b101, 3).unwrap();
let buf = w.finish();

// After (v0.4+, recommended)
use gorka::{BitWrite, RawBitWriter};
let mut storage = [0u8; 64];
let mut w = RawBitWriter::new(&mut storage);
w.write_bits(0b101, 3).unwrap();
let n = w.bytes_written();
let buf = &storage[..n];
```

**Removal schedule:**

| Version | Status                                                    |
| ------- | --------------------------------------------------------- |
| v0.3    | `BitWriter` — рабочий, без deprecated                     |
| v0.4    | `BitWriter` — `#[deprecated]`, предупреждение компилятора |
| v0.5    | `BitWriter` — будет удалён                                |

### Added

- **`bits::BitWrite` trait** (`src/bits/mod.rs`)
  Общий интерфейс для bit-level записи. Реализован для `RawBitWriter` и `BitWriter`.
  Методы: `write_bit`, `write_bits`, `write_bits_signed`, `align_to_byte`, `bit_len`, `is_aligned`.
  Используйте в generic-контексте: `fn encode<W: BitWrite>(w: &mut W, ...)`.

- **`RawBitWriter<'a>`** (`src/bits/raw_writer.rs`)
  Zero-copy bit writer поверх `&'a mut [u8]`. Не требует `alloc`. Подходит для `no_std` и embedded.
  Использует bulk-алгоритм (GORKA-13): fast-path для `n ≤ avail`, general-path O(n/8).
  Конструкторы: `new`, `from_offset`, `from_state` (pub(crate)).
  Аксессоры: `bytes_written`, `byte_pos`, `bit_pos`.

- **Тесты** (`tests/test_raw_bitwriter.rs`, `tests/bit_raw_property.rs`)
  Unit-тесты и property-based тесты (proptest) для `RawBitWriter`:
  roundtrip, побитовая симметрия с `BitReader`, align, signed, edge cases.

- **Бенчмарки** (`benches/raw_bitio_bench.rs`)
  Throughput в MiB/s для `RawBitWriter::write_bits` (1b/3b/7b/8b/9b/16b/32b/64b),
  `write_bit`, mixed encoder profile, roundtrip с `BitReader`.

- **Пример** (`examples/no_std_demo_raw.rs`)
  Демонстрация `RawBitWriter` + `BitWrite` в `no_std`-совместимом коде.

### Changed

- **`BitWriter`** (`src/bits/writer.rs`)
  Помечен `#[deprecated(since = "0.4.0")]`. Публичный API не изменился —
  `new()`, `write_bit`, `write_bits`, `write_bits_signed`, `finish()`, `align_to_byte`, `bit_len`, `is_aligned`
  работают идентично v0.3.
  Добавлен `impl BitWrite for BitWriter` для generic-совместимости.

- **`StreamEncoder`** (`src/codec/stream.rs`)
  Внутренний приватный `struct RawBitWriter` удалён. Вместо него используется
  публичный `crate::bits::RawBitWriter`. Публичный API `StreamEncoder` не изменился:
  `push_sample`, `flush`, `sample_count`, `bytes_written`, `STREAM_ENCODER_MIN_BUF_NO_PHASE`,
  `STREAM_ENCODER_MIN_BUF_WITH_PHASE` — совместимы с v0.3.

- **`README.md`**
  Обновлена секция `## no_std Mode`: добавлен `RawBitWriter`, план deprecation,
  пример миграции. Версия в `Cargo.toml`-примере обновлена до `"0.4"`.

### Compatibility

| API                            | v0.3 | v0.4                   |
| ------------------------------ | ---- | ---------------------- |
| `GlonassEncoder::encode_chunk` | ✅   | ✅ без изменений       |
| `GlonassDecoder::decode_chunk` | ✅   | ✅ без изменений       |
| `StreamEncoder::push_sample`   | ✅   | ✅ без изменений       |
| `StreamEncoder::flush`         | ✅   | ✅ без изменений       |
| `BitWriter`                    | ✅   | ⚠️ deprecated, рабочий |
| `BitReader`                    | ✅   | ✅ без изменений       |
| Wire format (chunk bytes)      | V1   | V1 без изменений       |

---

## [0.3.0] — 2026-03-05

### Added

- **codec/decoder**
  - добавлен zero-copy API для декодирования:
    - `decode_into(data: &[u8], out: &mut [GlonassSample]) -> Result<usize, GorkaError>`
      - декодирование в буфер, предоставленный вызывающим кодом (без аллокаций)
    - `DecodeIter<'a>` — потоковый итератор по sample без буферизации всего chunk
  - поддержка `no_std` (работа без `alloc`)
  - добавлены бенчмарки `decoder_bench.rs`:
    - сравнение `decode_chunk` (alloc) vs `decode_into` (zero-copy) vs `iter_chunk`
    - сценарии: smooth / constant / multi-slot

- **.github**
  - обновил конвиг для `ci.yml`

- **codec/stream**
  - StreamEncoder — новый инкрементальный encoder без аллокаций:
    - Поддержка `&mut [u8]` для embedded/фиксированных буферов
    - Методы:
      - `new(buf: &mut [u8])`
      - `push_sample(&mut self, sample: &GlonassSample) -> Result<usize, GorkaError>`
      - `flush(&mut self, out: &mut [u8]) -> Result<usize, GorkaError>`
      - `sample_count() -> u32`
      - `bytes_written() -> usize`
    - Автоматический выбор verbatim size: `23` байта без фазы, `31` с фазой
    - Поддержка всех 14 слотов Glonass (`slot_idx`)
    - Атомарный откат при ошибке записи (rollback)
    - Delta-кодирование timestamp, slot, CN0, pseudorange, doppler и carrier phase
    - ZigZag кодирование signed значений (`encode_i64`)
    - Поддержка отсутствующей и вновь появившейся carrier phase
    - Константы минимального буфера:
      - `STREAM_ENCODER_MIN_BUF_NO_PHASE = 32`
      - `STREAM_ENCODER_MIN_BUF_WITH_PHASE = 40`
  - Примеры:
    - `examples/stream_basic.rs` — базовый демо-тест, roundtrip проверка всех
      сэмплов
    - `examples/stream_performance.rs` — тест производительности для 100_000
      сэмплов, 162602 bytes → Push 34.7ms, Decode 28.6ms

### Changed

- **github**
  - обновил логику работы MSRV job в `ci.yml`

- **benches**
  - Обновлены тесты производительности `bitio_bench.rs` для reader и writer.

- **bits (BitWriter / BitReader)**
  - проведена оптимизация побитового ввода/вывода:
    - `write_bits`:
      - fast-path для частых случаев (`n ≤ 8`, `n ≤ 16`)
      - оптимизация записи через `u64` (bulk операции, снижение числа обращений
        к памяти)
    - `read_bits`:
      - добавлен кеш на основе `u64` с предварительной загрузкой (prefetch) при
        выравнивании
      - снижено количество побитовых операций при последовательном чтении
    - `skip_bits`:
      - подтверждена реализация через арифметику (без циклов)
    - горячие пути очищены от `debug_assert!` в release-сборке

  - достигнутые результаты:
    - streaming write (8-bit aligned):
      - до **~5.3× ускорения throughput** (≈177 → 300+ MiB/s в зависимости от размера)
    - streaming read:
      - стабильная пропускная способность до **~2.5 GiB/s**
    - single операций (`write_bits`):
      - улучшение на ~1–3% для типичных размеров (8 / 32 бита)
    - worst-case (`1 bit`):
      - без значимых изменений (ожидаемо)

  - вывод:
    - ускорен основной hot-path (streaming bit-IO)
    - оптимизация достигнута без ухудшения корректности и с минимальными регрессиями

- **codec/decoder**
  - оптимизация использования памяти:
    - снижены аллокации при декодировании за счёт `decode_into`
    - улучшена производительность (≈2–9% в зависимости от данных)

- **bits/writer**
  - улучшена документация (Rustdoc) для методов `BitWriter`
  - добавлены inline оптимизации для скорости

- **bits/reader**
  - улучшена документация (Rustdoc) для методов `BitReader`
  - добавлены пояснения по побитовому чтению и обработке signed значений через ZigZag

## [0.2.0] — 2026-03-03

### Added

- **.github**
  - добавил GitHub Actions workflow: `ci.yml`
  - добавил GitHub Actions workflow: `publish.yml`

- **docs**
  - обновил `README.md` файл
  - добавил `CODE_OF_CONDUCT.md` файл
  - добавил `SECURITY.md` файл
  - улучшил документацию в `lib.rs`

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
