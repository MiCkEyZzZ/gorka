# Gorka Binary Format — Specification v1

> **Status:** stable for v0.1.0
> **Endianness:** little-endian для всех многобайтовых полей в заголовках и
> verbatim-секции; MSB-first для bit-stream дельт.

## Содержание

1. [Обзор структуры chunk](#1-обзор-структуры-chunk)
2. [Chunk header — 9 байт](#2-chunk-header--9-байт)
3. [Verbatim первый сэмпл](#3-verbatim-первый-сэмпл)
4. [Bit-stream: дельта-кодирование сэмплов 1..N](#4-bit-stream-дельта-кодирование-сэмплов-1n)
   - 4.1 [Timestamp — delta-of-delta](#41-timestamp--delta-of-delta)
   - 4.2 [Slot — FDMA-слот](#42-slot--fdma-слот)
   - 4.3 [C/N₀](#cn0)
   - 4.4 [Pseudorange — delta-of-delta в миллиметрах](#44-pseudorange--delta-of-delta-в-миллиметрах)
   - 4.5 [Doppler — per-slot delta с FDMA-коррекцией](#45-doppler--per-slot-delta-с-fdma-коррекцией)
   - 4.6 [Carrier phase — опциональная delta-of-delta](#46-carrier-phase--опциональная-delta-of-delta)
5. [Таблица scale-факторов](#5-таблица-scale-факторов)
6. [Пример hex-дампа](#6-пример-hex-дампа)
7. [Версионирование и совместимость](#7-версионирование-и-совместимость)
8. [Известные ограничения v1](#8-известные-ограничения-v1)

## 1. Обзор структуры chunk

Chunk - минимальная единица хранения и передачи. Содержит одну непрерывную временную
серию наблюдений (один или несколько `GlonassSample`).

```text
┌─────────────────────────────────────────────────────────┐
│  CHUNK HEADER  9 bytes                                  │
│  [magic 4B LE][version 1B][sample_count 4B LE]          │
├─────────────────────────────────────────────────────────┤
│  VERBATIM FIRST SAMPLE  23 or 31 bytes                  │
│  Все поля без сжатия, little-endian                     │
├─────────────────────────────────────────────────────────┤
│  BIT-STREAM  variable length                            │
│  Gorilla-encoded deltas для сэмплов [1..N]              │
│  MSB-first bit ordering                                 │
│  Последний байт дополнен нулями до границы байта        │
└─────────────────────────────────────────────────────────┘
```

При `sample_count == 1` bit_stream отсутствует.

## 2. Chunk header — 9 байт

```text
| Смещение | Размер | Тип    | Значение                        |
|----------|--------|--------|---------------------------------|
| 0        | 4 B    | u32 LE | Magic `0x474F524B` (`"GORK"`)   |
| 4        | 1 B    | u8     | Версия формата (v1 = `0x01`)    |
| 5        | 4 B    | u32 LE | Количество сэмплов в chunk      |
```

### Magic

```text
0x47 0x4F 0x52 0x4B  →  'G' 'O' 'R' 'K'
```

Читается в little-endian: `u32::from_le_bytes([0x4B, 0x52, 0x4F, 0x47]) == 0x474F524B`.

Decoder обязан вернуть `GorkaError::InvalidMagic` если magic не совпадает.
Decoder обязан вернуть `GorkaError::InvalidVersion` если version неизвестна.

## 3. Verbatim первый сэмпл

Первый сэмпл всегда записывается без сжатия. Это точка отсчёта для всех последующих
дельт.

### Без carrier phase (`carrier_phase_cycles = None`) — 23 байта

```text
| Смещение | Размер | Тип    | Поле                          |
|----------|--------|--------|-------------------------------|
| 0        | 8 B    | u64 LE | `timestamp_ms`                |
| 8        | 1 B    | i8     | `slot` (хранится как u8)      |
| 9        | 1 B    | u8     | `cn0_dbhz`                    |
| 10       | 8 B    | i64 LE | `pseudorange_mm`              |
| 18       | 4 B    | i32 LE | `doppler_millihz`             |
| 22       | 1 B    | u8     | Phase flag: `0x00` = None     |
```

**Итого: 9 (header) + 23 = 32 байта** для chunk из одного сэмпла без фазы.

### С carrier phase (`carrier_phase_cycles = Some(v)`) — 31 байт

```text
| Смещение | Размер | Тип    | Поле                          |
|----------|--------|--------|-------------------------------|
| 0        | 8 B    | u64 LE | `timestamp_ms`                |
| 8        | 1 B    | i8     | `slot`                        |
| 9        | 1 B    | u8     | `cn0_dbhz`                    |
| 10       | 8 B    | i64 LE | `pseudorange_mm`              |
| 18       | 4 B    | i32 LE | `doppler_millihz`             |
| 22       | 1 B    | u8     | Phase flag: `0x01` = Some     |
| 23       | 8 B    | i64 LE | `carrier_phase_cycles` value  |
```

**Итого: 9 (header) + 31 = 40 байт** для chunk из одного сэмпла с фазой.

## 4. Bit-stream: дельта-кодирование сэмплов 1..N

Для каждого сэмпла `i` в диапазоне `[1, count)` encoder последовательно записывает
поля в порядке:

```text
timestamp → slot → cn0 → pseudorange → doppler → carrier_phase
```

Decoder читает их в том же порядке. Состояние (предыдущие значения, doppler-baseline)
хранится неявно в `EncoderState` / `DecoderState`, идентично инициализированных
из verbatim первого сэмпла.

### Соглашения bit-stream

- **Bit ordering:** MSB-first (старший бит байта — первый по времени).
- **Zigzag encoding:** знаковые значения перед записью преобразуются:

```text
  encode(v) = (v << 1) ^ (v >> 63)   // 0 → 0, -1 → 1, 1 → 2, -2 → 3 ...
  decode(z) = (z >> 1) ^ -(z & 1)
```

- Все bucket-схемы ниже показывают биты в порядке их записи (MSB-first).

### 4.1 Timestamp — delta-of-delta

Компрессия регулярных 1 ms интервалов до 1 бита через delta-of-delta (DoD).

**Переменные состояния:** `last_ts: u64`, `last_delta_ts: u64`

```text
delta   = ts.wrapping_sub(last_ts)
dod     = delta as i64 - last_delta_ts as i64
zz      = zigzag_encode(dod)
```

| Prefix | Условие  | Payload | Итого бит | Пример dod |
| ------ | -------- | ------- | --------- | ---------- |
| `0`    | dod == 0 | —       | 1         | 0          |
| `10`   | zz < 2⁷  | 7b zz   | 9         | ±63        |
| `110`  | zz < 2⁹  | 9b zz   | 12        | ±255       |
| `111`  | иначе    | 64b abs | 67        | любой gap  |

Для `111`-bucket в payload записывается абсолютное значение `ts` (64 бита), не DoD.

После decode: `last_delta_ts ← delta`, `last_ts ← ts`.

### 4.2 Slot — FDMA-слот

Слот меняется редко (обычно chunk содержит один спутник).

**Состояние:** `last_slot: i8`

```text
Если slot == last_slot:
    write_bit(false)          // '0'  — 1 бит

Иначе:
    write_bit(true)           // '1'
    write_bits(idx, 4)        // 4 бита: idx = slot + 7  ∈ [0, 13]
```

| Prefix | Payload | Итого бит | Значение             |
| ------ | ------- | --------- | -------------------- |
| `0`    | —       | 1         | слот не изменился    |
| `1`    | 4b idx  | 5         | новый слот `idx - 7` |

Допустимые слоты k ∈ [−7, +6], idx = k + 7 ∈ [0, 13].

### 4.3 C/N₀ {#cn0}

Медленно изменяющийся показатель качества сигнала. Дельта кодируется как i16 через
zigzag.

**Состояние:** `last_cn0: u8`

```text
delta = cn0 as i16 - last_cn0 as i16   // range [-255, 255]
zz    = zigzag_encode(delta as i64)     // max: zigzag(255) = 510 < 2⁹
```

| Prefix | Условие    | Payload | Итого бит |
| ------ | ---------- | ------- | --------- |
| `0`    | delta == 0 | —       | 1         |
| `1`    | иначе      | 9b zz   | 10        |

9 бит достаточно: `zigzag(255) = 510 < 512 = 2⁹`.

### 4.4 Pseudorange — delta-of-delta в миллиметрах

Псевдодальность монотонно растёт при движении спутника → DoD близка к нулю.

**Состояние:** `last_pr_mm: Millimeter`, `last_pr_delta: Millimeter`

```text
delta  = pr_mm - last_pr_mm        // i64, мм
dod    = delta - last_pr_delta     // i64, мм
zz     = zigzag_encode(dod)
```

| Prefix | Условие  | Payload | Итого бит | Покрытие dod |
| ------ | -------- | ------- | --------- | ------------ |
| `0`    | dod == 0 | —       | 1         | 0            |
| `10`   | zz < 2¹⁰ | 10b zz  | 12        | ±511 мм      |
| `110`  | zz < 2²⁰ | 20b zz  | 23        | ±524 287 мм  |
| `111`  | иначе    | 64b abs | 67        | любой скачок |

Для `111`-bucket payload = абсолютное значение `pr_mm.0 as u64` (64 бита).
После decode delta обновляется: `last_pr_delta ← delta`, `last_pr_mm ← pr`.

### 4.5 Doppler — per-slot delta с FDMA-коррекцией

Каждый из 14 слотов ГЛОНАСС имеет собственную несущую частоту, поэтому Doppler
для разных слотов нельзя дельта-кодировать совместно. Encoder хранит
`last_doppler: [Option<i32>; 14]` с отдельным состоянием для каждого слота.

**Первое появление слота** (state `last_doppler[idx] == None`):

```text
write_bit(false)            // флаг: verbatim
write_bits(doppler as u32, 32)   // 32 бита LE (знаковый i32 as u32)
```

Итого: **33 бита**.

**Последующие появления** (state `last_doppler[idx] == Some(prev)`):

```text
delta = doppler as i64 - prev as i64
zz    = zigzag_encode(delta)
```

| Prefix | Условие    | Payload | Итого бит | Покрытие delta       |
| ------ | ---------- | ------- | --------- | -------------------- |
| `10`   | delta == 0 | —       | 2         | 0 мГц                |
| `110`  | zz < 2¹⁴   | 14b zz  | 17        | ±8 191 мГц (~8.2 Гц) |
| `111`  | иначе      | 32b abs | 35        | любой сдвиг          |

Для `111`-bucket payload = `doppler as u32` (32 бита, знак через two's complement).

> **Примечание:** флаг `0` для первого появления и первый бит `1` для последующих
> — намеренная асимметрия. Decoder определяет тип по состоянию `last_doppler[idx]`,
> а не по значению флага.

### 4.6 Carrier phase — опциональная delta-of-delta

Фаза несущей может отсутствовать (потеря захвата), появляться заново и накапливаться.
Переходы кодируются 2-битным префиксом.

**Состояние:** `last_phase: Option<i64>`, `last_phase_delta: Option<i64>`

#### 2-битный префикс

| Prefix | Переход     | Payload                 |
| ------ | ----------- | ----------------------- |
| `00`   | None → None | —                       |
| `01`   | Some → None | —                       |
| `10`   | None → Some | 64b verbatim i64        |
| `11`   | Some → Some | DoD inner branch (ниже) |

#### Inner branch для `11` (Some → Some)

```text
delta    = curr - prev
prev_d   = last_phase_delta.unwrap_or(0)
dod      = delta - prev_d
zz       = zigzag_encode(dod)
```

| Следующие биты | Условие  | Payload | Итого бит (с `11`) | Покрытие dod        |
| -------------- | -------- | ------- | ------------------ | ------------------- |
| `0`            | dod == 0 | —       | 3                  | 0                   |
| `10`           | zz < 2³² | 32b zz  | 36                 | ±2 147 483 647 ед.  |
| `11`           | иначе    | 64b abs | 68                 | сброс DoD, verbatim |

При `11`-inner payload = абсолютное `curr as u64`; `last_phase_delta` сбрасывается
в `None`.

#### Правило last_phase_delta при Some → None

При переходе `Some → None` (`01`) значение `last_phase_delta` **не сбрасывается**.
Это важно для корректного восстановления DoD если фаза будет заново захвачена:
encoder и decoder должны сохранять симметрию состояния.

## 5. Таблица scale-факторов

Все числовые поля хранятся в целочисленном fixed-point формате. Float (f32/f64)
не используется.

| Поле                   | Тип хранения | Scale      | Единица  | Диапазон (плановый)                 |
| ---------------------- | ------------ | ---------- | -------- | ----------------------------------- |
| `timestamp_ms`         | `u64`        | 1 мс       | мс       | Unix timestamp ~1.7 × 10¹²          |
| `slot`                 | `i8`         | —          | —        | −7 .. +6 (14 слотов ГЛОНАСС)        |
| `cn0_dbhz`             | `u8`         | 1 дБГц     | дБГц     | 0 .. 255 (типично 20 .. 55)         |
| `pseudorange_mm`       | `i64`        | 1 мм       | мм       | 19 100 000 000 .. 25 600 000 000 мм |
| `doppler_millihz`      | `i32`        | 0.001 Гц   | мГц      | ±5 000 000 мГц (±5000 Гц)           |
| `carrier_phase_cycles` | `i64`        | 2⁻³² цикла | субциклы | накопленная фаза, опционально       |

### Обоснование выбора типов

**`pseudorange_mm: i64`** — `u32` (старый тип) ограничивал точность до 1 м и не
поддерживал отрицательные значения. `i64` покрывает диапазон ±9.2 × 10¹² мм >>
25 600 км максимальной дальности ГЛОНАСС.

**`doppler_millihz: i32`** — `i16` ограничивал диапазон ±32.7 Гц, недостаточно.
`i32` покрывает ±2.1 × 10⁹ мГц при точности 1 мГц.

**Отсутствие float** — целочисленный fixed-point даёт: детерминированное поведение
на всех платформах (включая embedded без FPU), предсказуемые дельты для
DoD-компрессии, совместимость с `no_std`.

## 6. Пример hex-дампа

### Chunk: 1 сэмпл без carrier phase

```rust
GlonassSample {
    timestamp_ms:          1_700_000_000_000,  // 0x0001_8BF8_E08C_0000
    slot:                  1,
    cn0_dbhz:              42,                  // 0x2A
    pseudorange_mm:        21_500_000_000,      // 0x0000_0005_02F9_0000
    doppler_millihz:       1_200_500,           // 0x001251F4
    carrier_phase_cycles:  None,
}
```

**Hex dump (32 байта):**

```text
Offset  Bytes                           Описание
------  ------------------------------  -----------------------------------
00      4B 52 4F 47                     Magic "GORK" (LE: 0x474F524B)
04      01                              Version V1
05      01 00 00 00                     sample_count = 1 (LE)
--- verbatim first sample (23 bytes) ---
09      00 80 F4 70 17 01 00 00         timestamp_ms = 1_700_000_000_000 (LE u64)
11      01                              slot = 1 (as u8)
12      2A                              cn0_dbhz = 42
13      00 00 F9 02 05 00 00 00         pseudorange_mm = 21_500_000_000 (LE i64)
1B      F4 51 12 00                     doppler_millihz = 1_200_500 (LE i32)
1F      00                              phase_flag = 0 (None)
```

**Итого: 32 байта** (`0x20`).

### Chunk: 3 сэмпла, постоянный сигнал (демонстрация bit-stream)

Три сэмпла с одинаковыми полями, timestamp растёт по 1 мс.

После verbatim сэмпла [0], сэмплы [1] и [2] кодируются в bit-stream.

**Сэмпл [1]:**

```text
timestamp:  dod = (1 - 0) - 0 = 1, zz = 2 < 128 → '10' + 7b zigzag(1) = 0b10_0000010
slot:       same → '0'
cn0:        delta = 0 → '0'
pseudorange: dod = 0 → '0'
doppler:    seen-before, delta = 0 → '10'
phase:      None → None → '00'
```

Итого: `10 0000010 0 0 0 10 00` = 15 бит → 2 байта с padding.

**Сэмпл [2]:**

```text
timestamp:  dod = (2-1) - (1-0) = 1 - 1 = 0 → '0'
slot:       same → '0'
cn0:        delta = 0 → '0'
pseudorange: dod = 0 → '0'
doppler:    delta = 0 → '10'
phase:      None → None → '00'
```

Итого: `0 0 0 0 10 00` = 8 бит → 1 байт.

## 7. Версионирование и совместимость

### Текущая версия

| Версия | Байт | Статус     | Описание                    |
| ------ | ---- | ---------- | --------------------------- |
| V1     | 0x01 | актуальная | Начальное кодирование Gorka |

### Правила backward/forward compatibility

**Backward compatibility (старый decoder читает новый chunk):**

- Decoder, реализующий V_reader, может читать chunk версии V_dump если
  `V_reader >= V_dump`.
- Decoder обязан отклонить chunk с неизвестной версией (`GorkaError::InvalidVersion`).

**Forward compatibility (новый decoder читает старый chunk):**

- Новый decoder может читать V1 chunk без ограничений.
- Добавление новых полей в V_next требует увеличения версии.

### Что можно изменить без смены версии

- Внутренние оптимизации encoder (другие bucket-границы) — **нельзя**: меняет
  wire format.
- Добавление новых методов API — **можно**.
- Изменение scale-факторов — **нельзя**: меняет семантику значений.

### Что требует V2

- Новое поле в `GlonassSample` (например, elevation angle).
- Изменение bucket-схемы для любого поля.
- Изменение порядка полей в verbatim или bit-stream.
- Изменение endianness.

### Правило миграции

При появлении V2 encoder должен уметь записывать V1 для обратной совместимости
(feature-флаг или явный параметр). Decoder V2 обязан читать V1 chunks.

## 8. Известные ограничения v1

| Ограничение                         | Описание                                                                 | Планируемое решение                                                                    |
| ----------------------------------- | ------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| Только GLONASS                      | `slot` специфичен для FDMA; нет поддержки PRN GPS/Galileo                | Issue #GORKA-11: multi-GNSS                                                            |
| Один слот первичен                  | `last_doppler` per-slot, но первый сэмпл задаёт baseline одного слота    | При многоспутниковом chunk каждый слот корректно инициализируется при первом появлении |
| Нет entropy coding                  | Bit-stream без Huffman/ANS → не оптимален для очень шумных данных        | Issue (future): entropy stage                                                          |
| Нет streaming API                   | `encode_chunk` требует `Vec<GlonassSample>` целиком                      | Issue #GORKA-9: `StreamEncoder`                                                        |
| `no_std` без `alloc`                | Сейчас encoder возвращает `Vec<u8>`                                      | Issue #GORKA-9: fixed-buffer API                                                       |
| `carrier_phase_delta` при Some→None | `last_phase_delta` не сбрасывается намеренно — требует симметрии decoder | Задокументировано, тест `test_roundtrip_carrier_phase_reacquired` покрывает            |

## Appendix A: Размер chunk в худшем случае

Для N сэмплов с максимальным jitter (все поля попадают в verbatim bucket):

```text
header:        9 B
first sample: 31 B (с фазой)
N-1 сэмплов:  (3 + 67 + 5 + 10 + 35 + 68) бит × (N-1) = 188 бит × (N-1)
```

Для N = 512: `9 + 31 + ceil(188 × 511 / 8) ≈ 12 044 B` ≈ 23.5 байт/сэмпл.
Для сравнения: raw = 31 × 512 = 15 872 B. Ratio > 1 даже в worst case.

## Appendix B: Размер chunk в лучшем случае

Постоянный сигнал (1 ms шаг, все поля константны):

```text
header:        9 B
first sample: 23 B (без фазы)
N-1 сэмплов:  (9 + 1 + 1 + 1 + 2 + 2) бит × (N-1) = 16 бит × (N-1)
```

Для N = 512: `9 + 23 + ceil(16 × 511 / 8) = 32 + 1022 = 1054 B`.
Raw = 23 × 512 = 11 776 B. **Ratio ≈ 11.2×** — соответствует наблюдаемым 21.65×
на практике (DoD timestamp часто 1 бит, а не 9).
