use crate::{encode_i64, BitWrite, GorkaError};

pub struct RawBitWriter<'a> {
    pub(crate) buf: &'a mut [u8],
    pub(crate) byte_pos: usize,
    pub(crate) bit_pos: u8, // current bit position in the current byte (0..=7)
}

impl<'a> RawBitWriter<'a> {
    /// Creates a new writer over the given buffer.
    pub fn new(buf: &'a mut [u8]) -> Self {
        buf.fill(0);

        Self {
            buf,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    /// Creates a writer starting at a given byte offset.
    pub fn from_offset(
        buf: &'a mut [u8],
        start: usize,
    ) -> Self {
        if start < buf.len() {
            buf[start..].fill(0);
        }

        Self {
            buf,
            byte_pos: start,
            bit_pos: 0,
        }
    }

    /// Returns the number of bytes written (including a partial byte if any).
    pub fn bytes_written(&self) -> usize {
        if self.bit_pos > 0 {
            self.byte_pos + 1
        } else {
            self.byte_pos
        }
    }

    /// Returns the current byte position.
    pub fn byte_pos(&self) -> usize {
        self.byte_pos
    }

    pub fn bit_pos(&self) -> u8 {
        self.bit_pos
    }

    /// Creates a writer from an existing state.
    ///
    /// Internal API: does **not** check correctness of `byte_pos` and
    /// `bit_pos`. Use for:
    /// - Resuming a previous write state
    /// - Integrating with streaming readers/writers
    #[allow(dead_code)]
    pub(crate) fn from_state(
        buf: &'a mut [u8],
        byte_pos: usize,
        bit_pos: u8,
    ) -> Self {
        Self {
            buf,
            byte_pos,
            bit_pos,
        }
    }

    /// Returns the number of bits available in the remaining buffer.
    fn bits_available(&self) -> usize {
        self.buf.len().saturating_sub(self.byte_pos) * 8 - self.bit_pos as usize
    }
}

impl<'a> BitWrite for RawBitWriter<'a> {
    #[inline(always)]
    fn write_bit(
        &mut self,
        bit: bool,
    ) -> Result<(), crate::GorkaError> {
        if self.byte_pos >= self.buf.len() {
            return Err(crate::GorkaError::BufferFull);
        }

        if bit {
            self.buf[self.byte_pos] |= 1 << (7 - self.bit_pos);
        }

        self.bit_pos += 1;

        if self.bit_pos == 8 {
            self.byte_pos += 1;
            self.bit_pos = 0;

            // Зануляем следующий байт заранее (чтобы OR работал корректно)
            if self.byte_pos < self.buf.len() {
                self.buf[self.byte_pos] = 0;
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn write_bits(
        &mut self,
        value: u64,
        n: u8,
    ) -> Result<(), GorkaError> {
        if n > 64 {
            return Err(GorkaError::InvalidBitCount(n));
        }
        if n < 64 && value >= (1u64 << n) {
            return Err(GorkaError::ValueTooLarge { value, bits: n });
        }
        if n == 0 {
            return Ok(());
        }

        if self.bits_available() < n as usize {
            return Err(GorkaError::BufferFull);
        }

        let avail = 8 - self.bit_pos;

        if n <= avail {
            let masked = if n == 64 {
                value
            } else {
                value & ((1u64 << n) - 1)
            };

            self.buf[self.byte_pos] |= (masked as u8) << (avail - n);
            self.bit_pos += n;

            if self.bit_pos == 8 {
                self.byte_pos += 1;
                self.bit_pos = 0;

                if self.byte_pos < self.buf.len() {
                    self.buf[self.byte_pos] = 0;
                }
            }

            return Ok(());
        }

        let mut rem = n;
        let mut val = value;

        // 1. заполнить текущий байт
        if self.bit_pos > 0 {
            let take = avail;

            let chunk = (val >> (rem - take)) & ((1u64 << take) - 1);
            self.buf[self.byte_pos] |= chunk as u8;

            self.byte_pos += 1;
            self.bit_pos = 0;

            if self.byte_pos < self.buf.len() {
                self.buf[self.byte_pos] = 0;
            }

            rem -= take;

            if rem < 64 {
                val &= (1u64 << rem) - 1;
            }
        }

        // 2. полные байты
        while rem >= 8 {
            rem -= 8;

            self.buf[self.byte_pos] = (val >> rem) as u8;
            self.byte_pos += 1;
        }

        // 3. остаток
        if rem > 0 {
            let chunk = val & ((1u64 << rem) - 1);
            self.buf[self.byte_pos] = (chunk as u8) << (8 - rem);
            self.bit_pos = rem;
        }

        Ok(())
    }

    #[inline(always)]
    fn write_bits_signed(
        &mut self,
        value: i64,
        n: u8,
    ) -> Result<(), crate::GorkaError> {
        self.write_bits(encode_i64(value), n)
    }

    fn align_to_byte(&mut self) {
        if self.bit_pos > 0 {
            self.byte_pos += 1;
            self.bit_pos = 0;

            if self.byte_pos < self.buf.len() {
                self.buf[self.byte_pos] = 0;
            }
        }
    }

    fn bit_len(&self) -> usize {
        self.byte_pos * 8 + self.bit_pos as usize
    }
}

#[allow(deprecated)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BitReader, GorkaError};

    #[test]
    fn test_new_zeros_buffer_and_is_aligned() {
        let mut buf = [0xFF; 8];
        let w = RawBitWriter::new(&mut buf);

        assert_eq!(w.bit_len(), 0);
        assert!(w.is_aligned());
        assert_eq!(w.bytes_written(), 0);
    }

    #[test]
    fn test_write_bit_single_true() {
        let mut buf = [0u8; 1];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bit(true).unwrap();

        assert_eq!(w.bit_len(), 1);

        assert_eq!(buf[0], 0b1000_0000);
    }

    #[test]
    fn test_write_bit_single_false() {
        let mut buf = [0xFFu8; 1];

        buf.fill(0);

        let mut w = RawBitWriter::new(&mut buf);

        w.write_bit(false).unwrap();

        assert_eq!(buf[0], 0b0000_0000);
    }

    #[test]
    fn test_write_full_byte() {
        let mut buf = [0u8; 1];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(0b1011_0101, 8).unwrap();

        assert!(w.is_aligned());

        assert_eq!(buf[0], 0b1011_0101);
    }

    #[test]
    fn test_write_bits_zero_n() {
        let mut buf = [0u8; 4];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(0, 0).unwrap();

        assert_eq!(w.bit_len(), 0);
    }

    #[test]
    fn test_write_bits_cross_byte_boundary() {
        let mut buf = [0u8; 2];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(0b101, 3).unwrap();
        w.write_bits(0b11110000, 8).unwrap();

        assert_eq!(w.bit_len(), 11);

        assert_eq!(buf[0], 0b1011_1110);
        assert_eq!(buf[1] >> 5, 0); // остаток 0b000 в старших битах
    }

    #[test]
    fn test_write_full_u64() {
        let mut buf = [0u8; 8];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(u64::MAX, 64).unwrap();

        assert_eq!(w.bit_len(), 64);
        assert!(w.is_aligned());

        assert_eq!(buf, [0xFF; 8]);
    }

    #[test]
    fn test_buffer_full_error_on_bit() {
        let mut buf = [0u8; 0];
        let mut w = RawBitWriter::new(&mut buf);

        assert!(matches!(w.write_bit(true), Err(GorkaError::BufferFull)));
    }

    #[test]
    fn test_buffer_full_error_on_bits() {
        let mut buf = [0u8; 1];
        let mut w = RawBitWriter::new(&mut buf);

        // 8 бит влезут, 9-й — нет
        w.write_bits(0xFF, 8).unwrap();

        assert!(matches!(w.write_bits(1, 1), Err(GorkaError::BufferFull)));
    }

    #[test]
    fn test_invalid_bit_count() {
        let mut buf = [0u8; 16];
        let mut w = RawBitWriter::new(&mut buf);

        assert!(matches!(
            w.write_bits(0, 65),
            Err(GorkaError::InvalidBitCount(65))
        ));
    }

    #[test]
    fn test_value_too_large() {
        let mut buf = [0u8; 4];
        let mut w = RawBitWriter::new(&mut buf);

        assert!(matches!(
            w.write_bits(0b1000, 3),
            Err(GorkaError::ValueTooLarge { .. })
        ));
    }

    #[test]
    fn test_align_to_byte() {
        let mut buf = [0u8; 2];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(0b101, 3).unwrap();

        assert!(!w.is_aligned());

        w.align_to_byte();

        assert!(w.is_aligned());
        assert_eq!(w.bit_len(), 8);
    }

    #[test]
    fn test_align_to_byte_already_aligned_is_noop() {
        let mut buf = [0u8; 2];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(0xFF, 8).unwrap();

        assert!(w.is_aligned());

        w.align_to_byte();

        assert_eq!(w.bit_len(), 8);
    }

    #[test]
    fn test_bytes_written() {
        let mut buf = [0u8; 4];
        let mut w = RawBitWriter::new(&mut buf);

        assert_eq!(w.bytes_written(), 0);

        w.write_bits(0xFF, 8).unwrap();

        assert_eq!(w.bytes_written(), 1);

        w.write_bits(0b101, 3).unwrap();

        assert_eq!(w.bytes_written(), 2); // частичный байт считается
    }

    #[test]
    fn test_from_offset() {
        let mut buf = [0xFFu8; 8];
        let mut w = RawBitWriter::from_offset(&mut buf, 2);

        w.write_bits(0b1010, 4).unwrap();

        // Байты 0,1 не тронуты (сохранены FF)
        assert_eq!(buf[0], 0xFF);
        assert_eq!(buf[1], 0xFF);
        // Байт 2 содержит 0b1010_0000
        assert_eq!(buf[2], 0b1010_0000);
    }

    #[test]
    fn test_write_bits_matches_bitwise_all_widths() {
        let cases: &[(u64, u8, &[u8])] = &[
            (0b1, 1, &[0b1000_0000]),
            (0b101, 3, &[0b1010_0000]),
            (0b10101, 5, &[0b1010_1000]),
            (0xFF, 8, &[0xFF]),
            (0x1234, 13, &[0x91, 0xA0]),
            (0xABCDE, 20, &[0xAB, 0xCD, 0xE0]),
            (0xDEAD_BEEF, 32, &[0xDE, 0xAD, 0xBE, 0xEF]),
            (
                u64::MAX >> 1,
                63,
                &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE],
            ),
            (u64::MAX, 64, &[0xFF; 8]),
        ];

        for &(val, n, expected) in cases {
            let mut raw_buf = [0u8; 16];
            let mut raw = RawBitWriter::new(&mut raw_buf);

            raw.write_bits(val, n).unwrap();

            let raw_bytes = raw.bytes_written();
            assert_eq!(&raw_buf[..raw_bytes], expected, "val={val:#x} n={n}");
        }
    }

    #[test]
    fn test_fast_path_stays_in_same_byte() {
        let mut buf = [0u8; 2];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(0b101, 3).unwrap(); // bit_pos=3
        w.write_bits(0b110, 3).unwrap(); // bit_pos=6, still in byte 0

        assert_eq!(w.bit_len(), 6);

        assert_eq!(buf[0], 0b1011_1000);
    }

    #[test]
    fn test_partial_full_partial() {
        let mut buf = [0u8; 8];
        let mut rw = RawBitWriter::new(&mut buf);

        rw.write_bits(0b111, 3).unwrap();
        rw.write_bits(0xDEAD_BEEF, 32).unwrap();
        rw.write_bits(0b10101, 5).unwrap();

        let n = rw.bytes_written();

        assert_eq!(&buf[..n], &[0xFB, 0xD5, 0xB7, 0xDD, 0xF5],);
    }

    #[test]
    fn test_roundtrip_with_reader() {
        let mut buf = [0u8; 32];
        let cases: &[(u64, u8)] = &[
            (0b10110, 5),
            (0b11001, 5),
            (0xDEAD, 16),
            (0b1, 1),
            (0xFF, 8),
            (0b0, 3),
        ];

        let mut w = RawBitWriter::new(&mut buf);

        for &(val, n) in cases {
            w.write_bits(val, n).unwrap();
        }

        let bytes_n = w.bytes_written();

        let mut r = BitReader::new(&buf[..bytes_n]);

        for &(val, n) in cases {
            assert_eq!(r.read_bits(n).unwrap(), val, "n={n}");
        }
    }

    #[test]
    fn test_signed_roundtrip() {
        let mut buf = [0u8; 32];
        let values: &[(i64, u8)] = &[
            (0, 1),
            (-1, 2),
            (1, 2),
            (-64, 8),
            (63, 7),
            (-1_000_000, 32),
            (1_000_000, 32),
        ];

        let mut w = RawBitWriter::new(&mut buf);

        for &(v, n) in values {
            w.write_bits_signed(v, n).unwrap();
        }

        let bytes_n = w.bytes_written();

        let mut r = BitReader::new(&buf[..bytes_n]);

        for &(v, n) in values {
            assert_eq!(r.read_bits_signed(n).unwrap(), v, "v={v} n={n}");
        }
    }

    #[test]
    fn test_implements_bit_write_trait() {
        fn write_via_trait(w: &mut impl BitWrite) -> Result<(), GorkaError> {
            w.write_bits(0b101, 3)?;
            w.write_bit(true)?;

            Ok(())
        }

        let mut buf = [0u8; 4];
        let mut w = RawBitWriter::new(&mut buf);

        write_via_trait(&mut w).unwrap();
        assert_eq!(w.bit_len(), 4);
    }

    #[test]
    fn test_from_state_resume() {
        let mut buf = [0u8; 2];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(0b101, 3).unwrap();

        let state = (w.byte_pos(), w.bit_pos());

        let mut w2 = RawBitWriter::from_state(&mut buf, state.0, state.1);

        w2.write_bits(0b111, 3).unwrap();

        assert_eq!(buf[0], 0b1011_1100);
    }
}
