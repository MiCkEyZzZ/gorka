//! Bit-level reader for compact binary codecs.
//!
//! Reads bits in MSB-first order and supports skipping, alignment, and signed
//! integer decoding. Safe for `no_std` environments and fixed-size buffers.

use crate::{decode_i64, GorkaError};

/// Bit-level reader over a byte slice.
///
/// `BitReader` reads bits in **MSB-first** order: the first bit of a byte is
/// the most significant.
///
/// This type is designed for reading compact binary streams where fields may
/// not align to byte boundaries.
///
/// # Guarantees
///
/// - Reading past the end of the buffer returns `GorkaError::UnexpectedEof`.
/// - Partial bytes are handled correctly; `align_to_byte()` skips to the next
///   byte boundary.
/// - All operations are safe and checked.
///
/// # Examples
///
/// ```ignore
/// let data = [0b1011_1000];
/// let mut r = BitReader::new(&data);
///
/// let a = r.read_bits(3).unwrap();
/// let b = r.read_bits(3).unwrap();
///
/// assert_eq!((a << 3) | b, 0b1011_1000 >> 2);
/// ```
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    /// Creates a new bit reader over `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    /// Reads a single bit in MSB-first order.
    ///
    /// # Errors
    ///
    /// Returns `GorkaError::UnexpectedEof` if the end of the buffer is reached.
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

    /// Reads the next `n` bits and returns them as an unsigned integer.
    ///
    /// # Errors
    ///
    /// - `GorkaError::InvalidBitCount(n)` if `n > 64`
    /// - `GorkaError::UnexpectedEof` if there are not enough bits remaining
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
        if n == 0 {
            return Ok(0);
        }

        // Fast path: all n bits are inside the current byte
        //
        // avail = unread bits remaining in the current byte (1..=8).
        let avail = 8 - self.bit_pos;
        if n <= avail {
            // Shift right to bring the n bits to the LSB.
            // Example: bit_pos=3, n=3, avail=5
            //   byte = 0bABCDEFGH, want bits D,E,F → shift right by (5-3)=2
            //   → 0b00ABCDEF, then mask with 0b00000111 = 7
            let shift = avail - n;
            let byte = self.data[self.byte_pos];
            // Mask: (1u16 << n) - 1 avoids u8 overflow when n=8
            let mask = ((1u16 << n) - 1) as u8;
            let out = ((byte >> shift) & mask) as u64;

            self.bit_pos += n;
            if self.bit_pos == 8 {
                self.byte_pos += 1;
                self.bit_pos = 0;
            }
            return Ok(out);
        }

        // General path: bits span multiple bytes
        //
        // 1. Consume the tail of the current byte.
        // 2. Read whole bytes from the middle.
        // 3. Consume the head of the next byte.

        let mut out = 0u64;
        let mut rem = n;

        // Step 1: consume tail of current byte (bit_pos may be 0 if we're
        // already aligned, in which case avail=8 ≥ n — but we handled that
        // in the fast path, so here bit_pos > 0 and avail < 8).
        if self.bit_pos > 0 {
            let take = avail; // bits available in current byte
            let byte = self.data[self.byte_pos];
            // Lower `take` bits of the byte, read left-to-right.
            let mask = ((1u16 << take) - 1) as u8;
            out = (byte & mask) as u64;
            rem -= take;
            self.byte_pos += 1;
            self.bit_pos = 0;
        }

        // Step 2: read whole bytes.
        while rem >= 8 {
            out = (out << 8) | self.data[self.byte_pos] as u64;
            self.byte_pos += 1;
            rem -= 8;
        }

        // Step 3: read remaining bits from the head of the next byte.
        if rem > 0 {
            let byte = self.data[self.byte_pos];
            // MSB `rem` bits of the byte.
            let bits = (byte >> (8 - rem)) as u64;
            out = (out << rem) | bits;
            self.bit_pos = rem;
        }

        Ok(out)
    }

    /// Reads `n` bits as a signed integer using ZigZag decoding.
    ///
    /// # Errors
    ///
    /// Same as [`read_bits`](Self::read_bits)
    #[inline(always)]
    pub fn read_bits_signed(
        &mut self,
        n: u8,
    ) -> Result<i64, GorkaError> {
        Ok(decode_i64(self.read_bits(n)?))
    }

    /// Returns the number of bits read so far.
    pub fn bits_read(&self) -> usize {
        self.byte_pos * 8 + self.bit_pos as usize
    }

    /// Returns the number of bits remaining in the buffer.
    pub fn bits_remaining(&self) -> usize {
        self.data.len() * 8 - self.bits_read()
    }

    /// Skips bits to the next byte boundary if not already aligned.
    pub fn align_to_byte(&mut self) {
        if self.bit_pos > 0 {
            self.byte_pos += 1;
            self.bit_pos = 0;
        }
    }

    /// Skips `n` bits in the stream.
    ///
    /// # Errors
    ///
    /// Returns `GorkaError::UnexpectedEof` if there are not enough bits left.
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

    /// Returns `true` if the reader is aligned to a byte boundary.
    pub fn is_aligned(&self) -> bool {
        self.bit_pos == 0
    }
}

#[allow(deprecated)]
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

    #[test]
    fn test_read_zero_bits() {
        let mut r = BitReader::new(&[0xFF]);

        assert_eq!(r.read_bits(0).unwrap(), 0);
        assert_eq!(r.bits_read(), 0);
    }

    #[test]
    fn test_fast_path_within_one_byte() {
        // Read 3 non-overlapping groups from the same byte
        let data = [0b1011_1001]; // правильно 8 бит, 4+4
        let mut r = BitReader::new(&data);

        assert_eq!(r.read_bits(3).unwrap(), 0b101);
        assert_eq!(r.read_bits(3).unwrap(), 0b110);
        assert_eq!(r.read_bits(2).unwrap(), 0b01);
    }

    #[test]
    fn test_fast_path_full_byte() {
        let data = [0xAB, 0xCD];
        let mut r = BitReader::new(&data);

        assert_eq!(r.read_bits(8).unwrap(), 0xAB);
        assert_eq!(r.read_bits(8).unwrap(), 0xCD);
    }

    #[test]
    fn test_general_path_many_bytes() {
        // Write 32-bit value split across byte boundaries, then read back
        let data = [0b101_00000, 0xDE, 0xAD, 0xBE, 0b11100000];
        let mut r = BitReader::new(&data);

        r.read_bits(3).unwrap(); // consume 3 bits, bit_pos=3
                                 // Now read 32 bits spanning bytes 0..4
                                 //
        let v = r.read_bits(32).unwrap();
        // Verify against bit-by-bit reference
        let mut r2 = BitReader::new(&data);

        r2.read_bits(3).unwrap();

        let mut expected = 0u64;

        for _ in 0..32 {
            expected = (expected << 1) | r2.read_bit().unwrap() as u64;
        }

        assert_eq!(v, expected);
    }

    #[test]
    fn test_bulk_read_matches_bitwise_all_widths() {
        use crate::BitWriter;

        // Encode a sequence of values with varying widths, then decode
        // using bulk reader and verify against bit-by-bit reference.
        let test_cases: &[(u64, u8)] = &[
            (0b1, 1),
            (0b101, 3),
            (0b10101, 5),
            (0xFF, 8),
            (0x1234, 13),
            (0xABCDE, 20),
            (0xDEADBEEF, 32),
            (u64::MAX >> 1, 63),
        ];

        let mut w = BitWriter::new();

        for &(val, n) in test_cases {
            w.write_bits(val, n).unwrap();
        }

        let buf = w.finish();

        // Bulk reader
        let mut r_bulk = BitReader::new(&buf);
        // Bitwise reference reader
        let mut r_ref = BitReader::new(&buf);

        for &(_, n) in test_cases {
            let bulk = r_bulk.read_bits(n).unwrap();
            let mut expected = 0u64;

            for _ in 0..n {
                expected = (expected << 1) | r_ref.read_bit().unwrap() as u64;
            }

            assert_eq!(bulk, expected, "n={n}");
        }
    }

    #[test]
    fn test_read_64_bits() {
        use crate::BitWriter;
        let mut w = BitWriter::new();

        w.write_bits(u64::MAX, 64).unwrap();

        let buf = w.finish();
        let mut r = BitReader::new(&buf);

        assert_eq!(r.read_bits(64).unwrap(), u64::MAX);
    }

    #[test]
    fn test_unaligned_read_64_bits() {
        use crate::BitWriter;
        let mut w = BitWriter::new();

        w.write_bits(0b101, 3).unwrap(); // 3 bits prefix
        w.write_bits(0xDEADBEEFCAFEBABE, 64).unwrap();

        let buf = w.finish();

        let mut r = BitReader::new(&buf);

        r.read_bits(3).unwrap(); // skip prefix

        assert_eq!(r.read_bits(64).unwrap(), 0xDEADBEEFCAFEBABE);
    }

    #[test]
    fn test_roundtrip_signed() {
        use crate::BitWriter;

        let values: &[(i64, u8)] = &[
            (0, 1),
            (-1, 2),
            (1, 2),
            (-64, 8),
            (63, 7),
            (-1_000_000, 32),
            (1_000_000, 32),
        ];
        let mut w = BitWriter::new();

        for &(v, n) in values {
            w.write_bits_signed(v, n).unwrap();
        }

        let buf = w.finish();
        let mut r = BitReader::new(&buf);

        for &(v, n) in values {
            assert_eq!(r.read_bits_signed(n).unwrap(), v, "v={v} n={n}");
        }
    }
}
