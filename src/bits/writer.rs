use alloc::vec::Vec;

use crate::{encode_i64, GorkaError};

pub struct BitWriter {
    buf: Vec<u8>,
    current: u8,
    pos: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            current: 0,
            pos: 0,
        }
    }

    #[inline(always)]
    pub fn write_bit(
        &mut self,
        bit: bool,
    ) {
        if bit {
            self.current |= 1 << (7 - self.pos);
        }

        self.pos += 1;

        if self.pos == 8 {
            self.buf.push(self.current);
            self.current = 0;
            self.pos = 0;
        }
    }

    #[inline(always)]
    pub fn write_bits(
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

        for i in (0..n).rev() {
            self.write_bit((value >> i) & 1 == 1);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn write_bits_signed(
        &mut self,
        value: i64,
        n: u8,
    ) -> Result<(), GorkaError> {
        let zz = encode_i64(value);

        self.write_bits(zz, n)
    }

    pub fn finish(mut self) -> Vec<u8> {
        if self.pos > 0 {
            self.buf.push(self.current);
        }

        self.buf
    }

    pub fn bit_len(&self) -> usize {
        self.buf.len() * 8 + self.pos as usize
    }

    pub fn align_to_byte(&mut self) {
        if self.pos > 0 {
            self.buf.push(self.current);
            self.current = 0;
            self.pos = 0;
        }
    }

    pub fn is_aligned(&self) -> bool {
        self.pos == 0
    }
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty_and_aligned() {
        let w = BitWriter::new();

        assert_eq!(w.bit_len(), 0);
        assert!(w.is_aligned());
    }

    #[test]
    fn test_write_single_bit_true() {
        let mut w = BitWriter::new();

        w.write_bit(true);

        assert_eq!(w.bit_len(), 1);
        assert!(!w.is_aligned());

        let buf = w.finish();

        assert_eq!(buf, vec![0b1000_0000]);
    }

    #[test]
    fn test_write_single_bit_false() {
        let mut w = BitWriter::new();

        w.write_bit(false);

        assert_eq!(w.bit_len(), 1);

        let buf = w.finish();

        assert_eq!(buf, vec![0b0000_0000]);
    }

    #[test]
    fn test_write_eight_bits_exact_byte() {
        let mut w = BitWriter::new();

        w.write_bits(0b1011_0101, 8).unwrap();

        assert_eq!(w.bit_len(), 8);
        assert!(w.is_aligned());

        let buf = w.finish();

        assert_eq!(buf, vec![0b1011_0101]);
    }

    #[test]
    fn test_write_bits_crosses_byte_boundary() {
        let mut w = BitWriter::new();

        w.write_bits(0b101, 3).unwrap();
        w.write_bits(0b11110000, 8).unwrap();

        assert_eq!(w.bit_len(), 11);
        assert!(!w.is_aligned());

        let buf = w.finish();

        assert_eq!(buf, vec![0b1011_1110, 0b0000_0000]);
    }

    #[test]
    fn test_write_bits_multiple_calls_same_result_as_one_stream() {
        let mut w1 = BitWriter::new();

        w1.write_bits(0b101, 3).unwrap();
        w1.write_bits(0b11, 2).unwrap();
        w1.write_bits(0b0, 1).unwrap();

        let buf1 = w1.finish();

        let mut w2 = BitWriter::new();

        w2.write_bits(0b101110, 6).unwrap();

        let buf2 = w2.finish();

        assert_eq!(buf1, buf2);
        assert_eq!(buf1, vec![0b1011_1000]);
    }

    #[test]
    fn test_align_to_byte_pads_current_byte() {
        let mut w = BitWriter::new();

        w.write_bits(0b101, 3).unwrap();

        assert!(!w.is_aligned());

        w.align_to_byte();

        assert!(w.is_aligned());
        assert_eq!(w.bit_len(), 8);

        let buf = w.finish();

        assert_eq!(buf, vec![0b1010_0000]);
    }

    #[test]
    fn test_align_to_byte_on_aligned_writer_is_noop() {
        let mut w = BitWriter::new();

        w.write_bits(0b1010_1010, 8).unwrap();

        assert!(w.is_aligned());

        w.align_to_byte();

        assert!(w.is_aligned());
        assert_eq!(w.bit_len(), 8);

        let buf = w.finish();

        assert_eq!(buf, vec![0b1010_1010]);
    }

    #[test]
    fn test_finish_flushes_partial_byte() {
        let mut w = BitWriter::new();

        w.write_bits(0b111, 3).unwrap();

        let buf = w.finish();

        assert_eq!(buf, vec![0b1110_0000]);
    }

    #[test]
    fn test_finish_on_empty_writer_returns_empty_vec() {
        let w = BitWriter::new();

        let buf = w.finish();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_full_u64_write() {
        let mut w = BitWriter::new();

        w.write_bits(u64::MAX, 64).unwrap();

        assert_eq!(w.bit_len(), 64);
        assert!(w.is_aligned());

        let buf = w.finish();
        assert_eq!(buf.len(), 8);
        assert_eq!(buf, vec![0xFF; 8]);
    }

    #[test]
    fn test_signed_roundtrip_shape() {
        let mut w = BitWriter::new();

        w.write_bits_signed(0, 1).unwrap(); // zigzag -> 0
        w.write_bits_signed(-1, 2).unwrap(); // zigzag -> 1
        w.write_bits_signed(1, 2).unwrap(); // zigzag -> 2
        w.write_bits_signed(-2, 3).unwrap(); // zigzag -> 3
        w.write_bits_signed(2, 3).unwrap(); // zigzag -> 4

        let buf = w.finish();
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_many_small_writes() {
        let mut w = BitWriter::new();

        for i in 0..1000 {
            w.write_bit(i % 2 == 0);
        }

        assert_eq!(w.bit_len(), 1000);

        let buf = w.finish();
        assert_eq!(buf.len(), 125);
        assert_eq!(buf[0], 0b1010_1010);
    }

    #[test]
    fn test_write_bits_value_too_large() {
        let mut w = BitWriter::new();
        let res = w.write_bits(0b1000, 3); // 8 не помещается в 3 бита

        assert!(res.is_err());
    }

    #[test]
    fn test_write_bits_invalid_bit_count() {
        let mut w = BitWriter::new();
        let res = w.write_bits(0, 65);

        assert!(res.is_err());
    }
}
