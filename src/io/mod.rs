// Это потоковый ввод/вывод chunk-последовательностей. Формат фрейма будет таким
// (length-prefixed) Несколько фреймов записываются последовательно - нет
// разделителей между ними. ChunkReader итерирует по фреймам, не копируя данные
// (&[u8] ссылка).

use std::io::{self, Write};

use crate::GorkaError;

/// Размер length-prefix в байтах (u32 LE).
pub const FRAME_HEADER_LEN: usize = 4;

/// Максимальный допустимый размер payload одного фрейма (64 MiB).
// Защищаем от корруптированных данных одного фрейма (64 MiB)
pub const MAX_FRAME_PAYLOAD: usize = 64 * 1024 * 1024;

pub struct ChunkWriter<W: Write> {
    inner: W,
    chunk_written: usize,
    bytes_written: usize,
}

/// Итератор по chunk-фреёмам в бийтовом срезе.
pub struct ChunkReader<'a> {
    data: &'a [u8],
    offset: usize,
    done: bool,
}

impl<W: Write> ChunkWriter<W> {
    /// Создаёт новый `ChunkWriter` поверх `writer`.
    pub fn new(writer: W) -> Self {
        Self {
            inner: writer,
            chunk_written: 0,
            bytes_written: 0,
        }
    }

    /// Записывает один chunk как length-prefixed фрейм.
    pub fn write_chunk(
        &mut self,
        chunk: &[u8],
    ) -> io::Result<()> {
        write_framed(chunk, &mut self.inner)?;

        self.chunk_written += 1;
        self.bytes_written += FRAME_HEADER_LEN + chunk.len();

        Ok(())
    }

    /// Сбрасывает буфер underlying writer.
    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    /// Вщвзращает кол-во chunk, записанных через этот writer.
    pub fn chunks_written(&self) -> usize {
        self.chunk_written
    }

    /// Возвращает суммарное кол-во байт, записанных (заголовок + payload).
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }

    /// Возвращает underlying writer, поглащая `ChunkWriter`.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<'a> ChunkReader<'a> {
    /// Создаёт итератор по фреймам в `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            offset: 0,
            done: false,
        }
    }

    /// Вовзращает `true` если байты прочитаны без ошибок.
    pub fn is_exhausted(&self) -> bool {
        self.done || self.offset >= self.data.len()
    }

    /// Возвращает кол-во байт, уже прочитанных итератором.
    pub fn bytes_read(&self) -> usize {
        self.offset
    }
}

impl<'a> Iterator for ChunkReader<'a> {
    type Item = Result<&'a [u8], GorkaError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.offset >= self.data.len() {
            return None;
        }

        match read_framed(&self.data[self.offset..]) {
            Ok((payload, consumed)) => {
                self.offset += consumed;
                Some(Ok(payload))
            }
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}

/// Записывает один chunk как length-prefixed фрейм (64 MiB).
// Формат: `[payload_len: u32 LE][payload: &[u8]]`
pub fn write_framed<W: Write>(
    chunk: &[u8],
    out: &mut W,
) -> io::Result<()> {
    if chunk.len() > MAX_FRAME_PAYLOAD {
        // Возвращаем io::Error, так как это уровень транспортного слоя (IO).
        // Проверка размера относится к framing, а не к domain-логике.
        //
        // TODO:
        // - унифицировать ошибки (например, IoFrameError)
        // - разделить transport и codec error model
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame payload too large",
        ));
    }

    let len = chunk.len() as u32;

    out.write_all(&len.to_le_bytes())?;
    out.write_all(chunk)?;

    Ok(())
}

/// Читает один фрейм c начала среза `data`.
pub fn read_framed(data: &[u8]) -> Result<(&[u8], usize), GorkaError> {
    if data.len() < FRAME_HEADER_LEN {
        return Err(GorkaError::UnexpectedEof);
    }

    // SAFETY: длина среза проверена выше
    let payload_len = u32::from_le_bytes(
        data[..FRAME_HEADER_LEN]
            .try_into()
            .expect("slice length checked"),
    ) as usize;

    if payload_len > MAX_FRAME_PAYLOAD {
        return Err(GorkaError::ValueTooLarge {
            value: payload_len as u64,
            bits: 32,
        });
    }

    let total = FRAME_HEADER_LEN + payload_len;

    if data.len() < total {
        return Err(GorkaError::UnexpectedEof);
    }

    Ok((&data[FRAME_HEADER_LEN..total], total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DbHz, GloSlot, GlonassDecoder, GlonassEncoder, GlonassSample, MilliHz, Millimeter,
    };

    const BASE_TS: u64 = 1_700_000_000_000;

    fn make_samples(
        n: usize,
        slot: GloSlot,
    ) -> Vec<GlonassSample> {
        (0..n)
            .map(|i| GlonassSample {
                timestamp_ms: BASE_TS + i as u64,
                slot,
                cn0_dbhz: DbHz::new(42).unwrap(),
                pseudorange_mm: Millimeter::new(21_500_000_000 + i as i64 * 222),
                doppler_millihz: MilliHz::new(1_200_000 + i as i32 * 10),
                carrier_phase_cycles: None,
            })
            .collect()
    }

    #[test]
    fn test_write_read_framed_single() {
        let payload = b"gorka_chunk_payload";
        let mut buf: Vec<u8> = Vec::new();

        write_framed(payload, &mut buf).unwrap();

        // header должен быть 4 байта + payload
        assert_eq!(buf.len(), 4 + payload.len());

        // header = payload.len() as u32 LE
        let stored_len = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize;

        assert_eq!(stored_len, payload.len());

        let (read_payload, consumed) = read_framed(&buf).unwrap();

        assert_eq!(read_payload, payload);
        assert_eq!(consumed, 4 + payload.len());
    }

    #[test]
    fn test_write_read_framed_empty_payload() {
        let mut buf: Vec<u8> = Vec::new();

        write_framed(b"", &mut buf).unwrap();

        assert_eq!(buf.len(), 4);

        let (payload, consumed) = read_framed(&buf).unwrap();

        assert!(payload.is_empty());
        assert_eq!(consumed, 4);
    }

    #[test]
    fn test_read_framed_truncated_header() {
        let data = [0x05, 0x00]; // только 2 байта вместо 4

        assert!(matches!(read_framed(&data), Err(GorkaError::UnexpectedEof)));
    }

    #[test]
    fn test_read_framed_truncated_payload() {
        let mut buf: Vec<u8> = Vec::new();

        write_framed(b"hello world", &mut buf).unwrap();

        // Обрезаем payload
        buf.truncate(buf.len() - 3);

        assert!(matches!(read_framed(&buf), Err(GorkaError::UnexpectedEof)));
    }

    #[test]
    fn test_read_framed_empty_slice() {
        assert!(matches!(read_framed(&[]), Err(GorkaError::UnexpectedEof)));
    }

    #[test]
    fn test_read_framed_oversized_payload_rejected() {
        // Записываем фиктивный header с payload_len > MAX_FRAME_PAYLOAD
        let huge_len = (MAX_FRAME_PAYLOAD + 1) as u32;
        let header = huge_len.to_le_bytes();

        // Предоставляем ровно 4 байта чтобы дойти до проверки размера
        assert!(matches!(
            read_framed(&header),
            Err(GorkaError::ValueTooLarge { .. })
        ));
    }

    #[test]
    fn test_chunk_writer_single_chunk() {
        let payload = b"single_chunk";
        let mut buf: Vec<u8> = Vec::new();
        let mut w = ChunkWriter::new(&mut buf);

        w.write_chunk(payload).unwrap();
        w.flush().unwrap();

        assert_eq!(w.chunks_written(), 1);
        assert_eq!(w.bytes_written(), 4 + payload.len());
        assert_eq!(buf.len(), 4 + payload.len());
    }

    #[test]
    fn test_chunk_writer_multiple_chunks() {
        let chunks: &[&[u8]] = &[b"alpha", b"beta", b"gamma_delta_epsilon"];
        let mut buf: Vec<u8> = Vec::new();
        let mut w = ChunkWriter::new(&mut buf);

        for c in chunks {
            w.write_chunk(c).unwrap();
        }

        w.flush().unwrap();

        assert_eq!(w.chunks_written(), 3);

        let expected_bytes: usize = chunks.iter().map(|c| 4 + c.len()).sum();

        assert_eq!(w.bytes_written(), expected_bytes);
        assert_eq!(buf.len(), expected_bytes);
    }

    #[test]
    fn test_chunk_writer_into_inner() {
        let mut buf: Vec<u8> = Vec::new();
        let mut w = ChunkWriter::new(&mut buf);

        w.write_chunk(b"test").unwrap();

        // into_inner возвращает underlying writer
        let _inner: &mut Vec<u8> = w.into_inner();
    }

    #[test]
    fn test_chunk_reader_empty_slice() {
        let reader = ChunkReader::new(&[]);
        let items: Vec<_> = reader.collect();

        assert!(items.is_empty());
    }

    #[test]
    fn test_chunk_reader_single_chunk() {
        let mut buf: Vec<u8> = Vec::new();

        write_framed(b"payload_one", &mut buf).unwrap();

        let mut reader = ChunkReader::new(&buf);
        let payload = reader.next().unwrap().unwrap();

        assert_eq!(payload, b"payload_one");
        assert!(reader.next().is_none());
        assert!(reader.is_exhausted());
    }

    #[test]
    fn test_chunk_reader_multiple_chunks_in_order() {
        let payloads: &[&[u8]] = &[b"first", b"second", b"third"];
        let mut buf: Vec<u8> = Vec::new();

        for p in payloads {
            write_framed(p, &mut buf).unwrap();
        }

        let read: Vec<&[u8]> = ChunkReader::new(&buf).map(|r| r.unwrap()).collect();

        assert_eq!(read.len(), 3);

        for (got, expected) in read.iter().zip(payloads.iter()) {
            assert_eq!(*got, *expected);
        }
    }

    #[test]
    fn test_chunk_reader_stops_after_error() {
        let mut buf: Vec<u8> = Vec::new();

        write_framed(b"good_chunk", &mut buf).unwrap();

        // Добавляем повреждённый второй фрейм: 100 байт payload, но данных нет
        buf.extend_from_slice(&100u32.to_le_bytes());

        // payload не добавляем → UnexpectedEof
        let mut reader = ChunkReader::new(&buf);

        assert!(reader.next().unwrap().is_ok()); // первый chunk — ок
        assert!(reader.next().unwrap().is_err()); // второй — ошибка
        assert!(reader.next().is_none()); // останавливаемся
    }

    #[test]
    fn test_chunk_reader_bytes_read_tracking() {
        let mut buf: Vec<u8> = Vec::new();

        write_framed(b"abcde", &mut buf).unwrap(); // 4 + 5 = 9
        write_framed(b"xyz", &mut buf).unwrap(); // 4 + 3 = 7

        let mut reader = ChunkReader::new(&buf);

        assert_eq!(reader.bytes_read(), 0);

        reader.next().unwrap().unwrap();

        assert_eq!(reader.bytes_read(), 9);

        reader.next().unwrap().unwrap();

        assert_eq!(reader.bytes_read(), 16);
    }

    #[test]
    fn test_writer_reader_roundtrip_bytes() {
        let inputs: Vec<Vec<u8>> = vec![
            b"chunk_one".to_vec(),
            b"second_payload_longer".to_vec(),
            vec![], // пустой chunk
            b"last".to_vec(),
        ];

        let mut buf: Vec<u8> = Vec::new();
        let mut writer = ChunkWriter::new(&mut buf);

        for chunk in &inputs {
            writer.write_chunk(chunk).unwrap();
        }

        writer.flush().unwrap();

        let outputs: Vec<Vec<u8>> = ChunkReader::new(&buf)
            .map(|r| r.unwrap().to_vec())
            .collect();

        assert_eq!(inputs, outputs);
    }

    #[test]
    fn test_writer_reader_roundtrip_many_chunks() {
        const N: usize = 1000;

        let mut buf: Vec<u8> = Vec::new();
        let mut writer = ChunkWriter::new(&mut buf);

        for i in 0..N {
            let payload = format!("chunk_{i:04}").into_bytes();
            writer.write_chunk(&payload).unwrap();
        }

        writer.flush().unwrap();

        let count = ChunkReader::new(&buf).count();

        assert_eq!(count, N);
    }

    #[test]
    fn test_codec_integration_single_chunk() {
        let samples = make_samples(32, GloSlot::new(1).unwrap());
        let chunk = GlonassEncoder::encode_chunk(&samples).unwrap();

        let mut buf: Vec<u8> = Vec::new();
        let mut writer = ChunkWriter::new(&mut buf);

        writer.write_chunk(&chunk).unwrap();
        writer.flush().unwrap();

        let decoded_samples: Vec<Vec<GlonassSample>> = ChunkReader::new(&buf)
            .map(|r| GlonassDecoder::decode_chunk(r.unwrap()).unwrap())
            .collect();

        assert_eq!(decoded_samples.len(), 1);
        assert_eq!(decoded_samples[0], samples);
    }

    #[test]
    fn test_codec_integration_multi_chunk_stream() {
        // 4 независимых chunk из разных спутников
        let slot_samples: Vec<Vec<GlonassSample>> = (-7_i8..=-4)
            .map(|slot| make_samples(64, GloSlot::new(slot).unwrap()))
            .collect();

        let mut buf: Vec<u8> = Vec::new();
        let mut writer = ChunkWriter::new(&mut buf);

        for samples in &slot_samples {
            let chunk = GlonassEncoder::encode_chunk(samples).unwrap();
            writer.write_chunk(&chunk).unwrap();
        }

        writer.flush().unwrap();

        assert_eq!(writer.chunks_written(), 4);

        let decoded: Vec<Vec<GlonassSample>> = ChunkReader::new(&buf)
            .map(|r| GlonassDecoder::decode_chunk(r.unwrap()).unwrap())
            .collect();

        assert_eq!(decoded.len(), slot_samples.len());

        for (got, expected) in decoded.iter().zip(slot_samples.iter()) {
            assert_eq!(got, expected);
        }
    }

    #[test]
    fn test_codec_integration_large_chunks() {
        // 8192 samples — проверяем что большие chunk корректно framed
        let samples = make_samples(8192, GloSlot::new(0).unwrap());
        let chunk = GlonassEncoder::encode_chunk(&samples).unwrap();

        let mut buf: Vec<u8> = Vec::new();

        write_framed(&chunk, &mut buf).unwrap();

        let (payload, consumed) = read_framed(&buf).unwrap();

        assert_eq!(consumed, buf.len());

        let decoded = GlonassDecoder::decode_chunk(payload).unwrap();

        assert_eq!(decoded, samples);
    }

    #[test]
    fn test_codec_integration_roundtrip_preserves_order() {
        // 10 chunk по 128 сэмплов — проверяем порядок и точность
        let all_samples: Vec<Vec<GlonassSample>> = (0..10)
            .map(|i| make_samples(128, GloSlot::new((i % 14) as i8 - 7).unwrap()))
            .collect();

        let mut buf: Vec<u8> = Vec::new();
        let mut w = ChunkWriter::new(&mut buf);

        for s in &all_samples {
            w.write_chunk(&GlonassEncoder::encode_chunk(s).unwrap())
                .unwrap();
        }

        w.flush().unwrap();

        let decoded: Vec<Vec<GlonassSample>> = ChunkReader::new(&buf)
            .enumerate()
            .map(|(i, r)| {
                let dec = GlonassDecoder::decode_chunk(r.unwrap()).unwrap();
                assert_eq!(dec, all_samples[i], "chunk {i} mismatch");
                dec
            })
            .collect();

        assert_eq!(decoded.len(), 10);
    }

    #[test]
    fn test_frame_header_len_is_four() {
        assert_eq!(FRAME_HEADER_LEN, 4);
    }

    #[test]
    fn test_max_frame_payload_is_sane() {
        // 64 MiB — достаточно для любого разумного chunk
        assert_eq!(MAX_FRAME_PAYLOAD, 64 * 1024 * 1024);
    }
}
