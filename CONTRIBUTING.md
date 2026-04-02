# Contributing to Gorka

Спасибо за интерес к проекту! Gorka — библиотека для сжатия GNSS
телеметрии. Любые вклады приветствуются: исправления, новые функции,
документация, тесты.

---

## Содержание

- [Быстрый старт для контрибьютора](#быстрый-старт-для-контрибьютора)
- [Структура проекта](#структура-проекта)
- [Стиль кода](#стиль-кода)
- [Тесты](#тесты)
- [Как добавить поддержку нового созвездия](#как-добавить-поддержку-нового-созвездия)
- [Работа с issues](#работа-с-issues)
- [Pull Request checklist](#pull-request-checklist)

---

## Быстрый старт для контрибьютора

```zsh
# Клонировать
git clone https://github.com/MiCkEyZzZ/gorka
cd gorka

# Установить инструменты
cargo install cargo-nextest taplo-cli

# Проверить, что всё работает
just check
# или вручную:
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
```

Все команды разработки:

```zsh
just fmt-all   # форматировать Rust + TOML
just lint      # clippy
just test-next # все тесты через nextest
just bench     # бенчмарки
just dev       # fmt + check (рекомендуется перед коммитом)
```

---

## Структура проекта

```text
src/
├── bits/          — BitReader, BitWriter (bit-level IO)
├── codec/
│   ├── encoder.rs — GlonassEncoder (основной кодировщик)
│   ├── decoder.rs — GlonassDecoder (зеркало encoder)
│   ├── delta.rs   — delta / delta-of-delta вычисления
│   ├── zigzag.rs  — encode_i64 / decode_i64
│   └── format/    — FormatVersion, CHUNK_MAGIC, header layout
├── gnss/
│   ├── types.rs   — Millimeter, MilliHz (newtype wrappers)
│   ├── glonass.rs — GlonassSample, валидация, вспомогательные методы
│   ├── frame.rs   — GnssFrame (буфер эпохи, фиксированный массив)
│   └── mod.rs
├── io/mod.rs      — ChunkWriter, ChunkReader (std only)
└── error.rs       — GorkaError

tests/             — интеграционные тесты
benches/           — Criterion benchmarks
docs/              — спецификации и документация
examples/          — рабочие примеры
```

Ключевой инвариант: **encoder и decoder должны быть зеркальными**.
Любое изменение в `encode_delta` требует симметричного изменения в
`decode_delta`, и наоборот.

---

## Стиль кода

Проект следует стандартному Rust-стилю с некоторыми уточнениями:

- Форматирование через `cargo fmt` (настройки в `rustfmt.toml`)
- Clippy без предупреждений (`-D warnings`)
- Документационные комментарии на **русском** для внутренней документации,
  на **английском** для публичного API (doc-комментарии `///`)
- Никаких `unwrap()` в production-коде кроме мест с `debug_assert!`
- `#[inline(always)]` только для горячих path в bit-IO
- Комментарии к bucket-схемам обязательны: `// '10' + 7b zigzag`

---

## Тесты

Каждая новая функция должна иметь:

1. **Unit-тесты** в `#[cfg(test)]` блоке внутри модуля — базовая корректность
2. **Roundtrip тест** — encode → decode должен возвращать идентичные данные
3. **Edge-case тесты** — граничные значения (пустой chunk, максимальный слот, None-фаза)
4. **Property тест** в `tests/` через `proptest` — случайные валидные данные

Правило: **если добавляешь bucket в encoder — добавь тест для этого bucket**.

```zsh
# Запустить конкретный тест
cargo test test_roundtrip_carrier_phase_reacquired

# С выводом println!
cargo test --test compression_ratio -- --nocapture
```

---

## Как добавить поддержку нового созвездия

Это основная точка расширения Gorka. Ниже пошаговый план добавления,
например, **GPS**.

### Шаг 1: Определить тип данных

Создайте `src/gnss/gps.rs`:

```rust
use crate::{error::GorkaError, MilliHz, Millimeter};

/// One GPS L1 C/A observation.
///
/// PRN = Pseudo-Random Noise code number, identifies the satellite.
/// GPS L1 C/A: 1575.42 MHz, всем спутникам одна частота (CDMA).
#[derive(Debug, Clone, PartialEq)]
pub struct GpsSample {
    pub timestamp_ms:         u64,
    pub prn:                  u8,          // 1..=32
    pub cn0_dbhz:             u8,
    pub pseudorange_mm:       Millimeter,
    pub doppler_millihz:      MilliHz,
    pub carrier_phase_cycles: Option<i64>,
}

impl GpsSample {
    pub const PRN_MIN: u8 = 1;
    pub const PRN_MAX: u8 = 32;

    pub fn validate_prn(&self) -> Result<(), GorkaError> {
        if !(Self::PRN_MIN..=Self::PRN_MAX).contains(&self.prn) {
            return Err(GorkaError::InvalidPrn(self.prn));
        }
        Ok(())
    }
    // ...
}
```

### Шаг 2: Добавить в gnss/mod.rs

```rust
pub mod gps;
pub use gps::GpsSample;
```

### Шаг 3: Реализовать encoder

Создайте `src/codec/gps_encoder.rs`. Скопируйте структуру из `encoder.rs`
и адаптируйте:

- `slot` → `prn` (PRN 1..32, 5 бит вместо 4)
- Удалите per-slot FDMA state (GPS использует CDMA — одна частота)
- Доплер: дельта без FDMA-коррекции (все спутники на одной несущей)

```rust
pub struct GpsEncoder;

impl GpsEncoder {
    pub fn encode_chunk(samples: &[GpsSample]) -> Result<Vec<u8>, GorkaError> {
        // Аналогично GlonassEncoder, но:
        // - state.last_prn вместо last_slot
        // - last_doppler: Option<i32> (один, не массив)
        // - verbatim: 1B prn вместо 1B slot
        todo!()
    }
}
```

### Шаг 4: Реализовать decoder

`src/codec/gps_decoder.rs` — точное зеркало encoder. Добавьте тест:

```rust
#[test]
fn test_gps_roundtrip() {
    let samples: Vec<GpsSample> = (1..=10).map(|prn| GpsSample {
        prn,
        // ...
    }).collect();
    let enc = GpsEncoder::encode_chunk(&samples).unwrap();
    let dec = GpsDecoder::decode_chunk(&enc).unwrap();
    assert_eq!(samples, dec);
}
```

### Шаг 5: Обновить публичный API

В `src/lib.rs`:

```rust
pub use codec::{GpsDecoder, GpsEncoder, /* ... */};
pub use gnss::{GpsSample, /* ... */};
```

### Шаг 6: Обновить FORMAT.md

Добавьте раздел в `docs/FORMAT.md`:

```markdown
## 9. GPS chunk format (V2)

Chunk версии V2 добавляет...
```

Изменение формата требует нового `FormatVersion::V2`.

### Шаг 7: Обновить тесты и бенчмарки

- `tests/gps_roundtrip.rs` — полные roundtrip тесты
- `benches/encode_bench.rs` — добавьте `bench_encode_gps_smooth`
- `tests/compression_ratio.rs` — добавьте `compression_ratio_gps_smooth`

### Шаг 8: Пример

`examples/multi_gnss.rs` — покажите совместное использование GLONASS и GPS.

### Ключевые правила при добавлении созвездий

- **Один тип — один encoder/decoder**. Не делайте общий "GNSS encoder".
- **Версионирование**: изменение wire format = новый `FormatVersion`.
- **Fixed-point везде**: никаких `f32`/`f64` в codec-пути.
- **Тест на симметрию**: любые `encode_X` / `decode_X` покрыты roundtrip.
- **Per-сигнал state**: если сигнал использует FDMA (ГЛОНАСС) → массив
  состояний; если CDMA (GPS, Galileo) → одно состояние.

---

## Работа с issues

- **Bug**: воспроизведите минимальный пример, укажите ожидаемое и реальное поведение
- **Feature**: откройте issue с меткой `enhancement` перед реализацией
- **Breaking change**: обсудите в issue, требует major версии

---

## Pull Request checklist

- [ ] `just fmt-all` — код отформатирован
- [ ] `just lint` — clippy без предупреждений
- [ ] `just test-next` — все тесты зелёные
- [ ] Новые публичные API имеют doc-комментарии (`///`)
- [ ] `CHANGELOG.md` обновлён
- [ ] Если изменён wire format — обновлён `docs/FORMAT.md` и версия
