use crate::{decode_i64, GorkaError};

pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    #[inline(always)]
    pub fn read_bit(&mut self) -> Result<bool, GorkaError> {
        if self.byte_pos >= self.data.len() {
            return Err(GorkaError::UnexpectedEof);
        }

        let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1 == 1;

        self.bit_pos += 1;

        if self.bit_pos == 8 {
            self.byte_pos += 1;
            self.bit_pos = 0;
        }

        Ok(bit)
    }

    #[inline(always)]
    pub fn read_bits(
        &mut self,
        n: u8,
    ) -> Result<u64, GorkaError> {
        if n > 64 {
            return Err(GorkaError::InvalidBitCount(n));
        }

        if self.bits_remaining() < n as usize {
            return Err(GorkaError::UnexpectedEof);
        }

        let mut out = 0u64;

        for _ in 0..n {
            out = (out << 1) | self.read_bit()? as u64;
        }

        Ok(out)
    }

    #[inline(always)]
    pub fn read_bits_signed(
        &mut self,
        n: u8,
    ) -> Result<i64, GorkaError> {
        let zz = self.read_bits(n)?;

        Ok(decode_i64(zz))
    }

    pub fn bits_read(&self) -> usize {
        self.byte_pos * 8 + self.bit_pos as usize
    }

    pub fn bits_remaining(&self) -> usize {
        self.data.len() * 8 - self.bits_read()
    }

    pub fn align_to_byte(&mut self) {
        if self.bit_pos > 0 {
            self.byte_pos += 1;
            self.bit_pos = 0;
        }
    }

    pub fn skip_bits(
        &mut self,
        n: u8,
    ) -> Result<(), GorkaError> {
        if self.bits_remaining() < n as usize {
            return Err(GorkaError::UnexpectedEof);
        }

        let total = self.bit_pos as usize + n as usize;

        self.byte_pos += total / 8;
        self.bit_pos = (total % 8) as u8;

        Ok(())
    }

    pub fn is_aligned(&self) -> bool {
        self.bit_pos == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reader_is_empty() {
        let data = [];
        let r = BitReader::new(&data);

        assert_eq!(r.bits_read(), 0);
        assert_eq!(r.bits_remaining(), 0);
        assert!(r.is_aligned());
    }

    #[test]
    fn test_read_single_bit() {
        let data = [0b1000_0000];
        let mut r = BitReader::new(&data);
        let bit = r.read_bit().unwrap();

        assert!(bit);
        assert_eq!(r.bits_read(), 1);
    }

    #[test]
    fn test_read_bits_full_byte() {
        let data = [0b1011_0101];
        let mut r = BitReader::new(&data);
        let v = r.read_bits(8).unwrap();

        assert_eq!(v, 0b1011_0101);
        assert!(r.is_aligned());
    }

    #[test]
    fn test_read_bits_cross_byte_boundary() {
        let data = [0b1011_1110, 0b0000_0000];
        let mut r = BitReader::new(&data);
        let v = r.read_bits(3).unwrap();

        assert_eq!(v, 0b101);

        let v2 = r.read_bits(8).unwrap();

        assert_eq!(v2, 0b11110000);
    }

    #[test]
    fn test_read_bits_multiple_calls_equivalent() {
        let data = [0b1011_1000];
        let mut r1 = BitReader::new(&data);

        let a = r1.read_bits(3).unwrap();
        let b = r1.read_bits(2).unwrap();
        let c = r1.read_bits(1).unwrap();

        let mut r2 = BitReader::new(&data);
        let combined = r2.read_bits(6).unwrap();

        assert_eq!((a << 3) | (b << 1) | c, combined);
    }

    #[test]
    fn test_unexpected_eof_on_read_bit() {
        let data = [];
        let mut r = BitReader::new(&data);

        let res = r.read_bit();

        assert!(matches!(res, Err(GorkaError::UnexpectedEof)));
    }

    #[test]
    fn test_unexpected_eof_on_read_bits() {
        let data = [0b1010_0000];
        let mut r = BitReader::new(&data);

        let res = r.read_bits(16);

        assert!(matches!(res, Err(GorkaError::UnexpectedEof)));
    }

    #[test]
    fn test_invalid_bit_count() {
        let data = [0];
        let mut r = BitReader::new(&data);

        let res = r.read_bits(65);

        assert!(matches!(res, Err(GorkaError::InvalidBitCount(65))));
    }

    #[test]
    fn test_align_to_byte() {
        let data = [0b1010_0000, 0b1111_0000];
        let mut r = BitReader::new(&data);

        r.read_bits(3).unwrap();
        assert!(!r.is_aligned());

        r.align_to_byte();

        assert!(r.is_aligned());
        assert_eq!(r.bits_read(), 8);

        let v = r.read_bits(8).unwrap();
        assert_eq!(v, 0b1111_0000);
    }

    #[test]
    fn test_align_on_aligned_is_noop() {
        let data = [0b1010_1010];
        let mut r = BitReader::new(&data);

        assert!(r.is_aligned());

        r.align_to_byte();

        assert!(r.is_aligned());
        assert_eq!(r.bits_read(), 0);
    }

    #[test]
    fn test_skip_bits() {
        let data = [0b1011_0101];
        let mut r = BitReader::new(&data);

        r.skip_bits(3).unwrap();

        let v = r.read_bits(3).unwrap();

        assert_eq!(v, 0b101);
    }

    #[test]
    fn test_skip_bits_across_bytes() {
        let data = [0b1111_0000, 0b1010_1010];
        let mut r = BitReader::new(&data);

        r.skip_bits(8).unwrap();

        let v = r.read_bits(8).unwrap();

        assert_eq!(v, 0b1010_1010);
    }

    #[test]
    fn test_skip_bits_eof() {
        let data = [0];
        let mut r = BitReader::new(&data);

        let res = r.skip_bits(16);

        assert!(matches!(res, Err(GorkaError::UnexpectedEof)));
    }

    #[test]
    fn test_bits_remaining_tracking() {
        let data = [0b1111_0000];
        let mut r = BitReader::new(&data);

        assert_eq!(r.bits_remaining(), 8);

        r.read_bits(3).unwrap();

        assert_eq!(r.bits_remaining(), 5);
    }

    #[test]
    fn test_signed_read() {
        let data = [0b0110_0000]; // zigzag: 3 (-> -2)
        let mut r = BitReader::new(&data);

        let v = r.read_bits_signed(3).unwrap();

        assert_eq!(v, -2);
    }
}
