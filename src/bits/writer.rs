//! Bit-level writer for compact binary codecs.
//!
//! Writes bits in MSB-first order and supports alignment and ZigZag-encoded
//! signed integers.
//!
//! This implementation uses a dynamically growing buffer (`Vec<u8>`), making it
//! convenient for general-purpose encoding. For `no_std` or fixed-buffer use
//! cases, see `RawBitWriter`.
//!
//! # Guarantees
//!
//! - Bits are written in **MSB-first** order (same as \[`BitReader`\])
//! - Partial bytes are buffered and flushed on `finish()` or `align_to_byte()`.
//! - Invalid bit widths and oversized values are checked.
//! - All operations are safe.
//!
//! # Examples
//!
//! ```ignore
//! let mut w = BitWriter::new();
//!
//! w.write_bits(0b101, 3).unwrap();
//! w.write_bits(0b11, 2).unwrap();
//!
//! let buf = w.finish();
//!
//! assert_eq!(buf, vec![0b1011_1000]);
//! ```

use alloc::vec::Vec;

use crate::{encode_i64, GorkaError};

/// Bit-level writer backed by a dynamically growing buffer.
///
/// `BitWriter` writes bits in **MSB-first** order: the first bit written
/// becomes the most significant bit of the byte.
///
/// # Deprecation (v0.4.0)
///
/// Перейдите на [`RawBitWriter`][crate::RawBitWriter] для zero-alloc записи
/// или используйте trait [`BitWrite`][crate::bits::BitWrite] как общий
/// интерфейс.
///
/// ## Путь миграции
///
/// ```ignore
/// // v0.3 (deprecated)
/// use gorka::BitWriter;
/// let mut w = BitWriter::new();
/// w.write_bits(0b101, 3).unwrap();
/// let buf = w.finish();
///
/// // v0.4+ (рекомендуется)
/// use gorka::RawBitWriter;
/// let mut storage = [0u8; 64];
/// let mut w = RawBitWriter::new(&mut storage);
/// w.write_bits(0b101, 3).unwrap();
/// let n = w.bytes_written();
/// ```
///
/// This type is suitable for building compact binary streams where fields may
/// not align to byte boundaries.
#[deprecated(
    since = "0.4.0",
    note = "Use `RawBitWriter<'a>` (zero-alloc) or `impl BitWriter` instead. \
    `BitWrite` will be removed in v0.5.0."
)]
pub struct BitWriter {
    buf: Vec<u8>,
    current: u8,
    pos: u8,
}

#[allow(deprecated)]
impl BitWriter {
    /// Creates a new empty bit writer.s
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            current: 0,
            pos: 0,
        }
    }

    /// Writes a single bit in MSB-first order.
    ///
    /// This operation never fails.
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

    /// Writes the lowest `n` bits of `value`.
    ///
    /// Bits are written from most significant to least significant.
    ///
    /// # Errors
    ///
    /// - `GorkaError::InvalidBitCount(n)` if `n > 64`
    /// - `GorkaError::ValueTooLarge` if `value` does not fit in `n` bits
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
        if n == 0 {
            return Ok(());
        }

        // Fast path: all n bits fit in the current (partial) byte
        //
        // avail = bits remaining in the current byte (1..=8).
        // Condition: n ≤ avail — no byte boundary crossing needed.
        let avail = 8 - self.pos;
        if n <= avail {
            // Shift value to align its MSB with the next free bit position.
            // Example: pos=3 (3 bits used), n=3, avail=5
            //   value = 0b101 → shift left by (avail-n)=2 → 0b10100
            //   OR into current: 0bXXX_10100 → MSB side
            self.current |= (value as u8) << (avail - n);
            self.pos += n;
            if self.pos == 8 {
                self.buf.push(self.current);
                self.current = 0;
                self.pos = 0;
            }
            return Ok(());
        }

        // General path: bits span multiple bytes
        //
        // 1. Fill the current partial byte with the top bits of `value`.
        // 2. Write full bytes from the middle of `value`.
        // 3. Store leftover bits in the new `current`.

        let mut rem = n; // bits still to write
        let mut val = value;

        // Step 1: fill current byte (pos > 0 guaranteed because avail < n ≤ 64,
        // and avail = 8 - pos, so pos > 8 - n ≥ 0; since n ≥ 1, avail ≤ 7 → pos ≥ 1).
        if self.pos > 0 {
            let take = avail; // bits to consume from val's MSB
                              // Extract the top `take` bits of val.
            let top_bits = (val >> (rem - take)) as u8;
            self.current |= top_bits;
            self.buf.push(self.current);
            self.current = 0;
            self.pos = 0;
            rem -= take;
            // Zero out the bits we just consumed.
            if rem < 64 {
                val &= (1u64 << rem) - 1;
            }
        }

        // Step 2: write whole bytes.
        while rem >= 8 {
            rem -= 8;
            self.buf.push((val >> rem) as u8);
        }

        // Step 3: store leftover bits in current.
        if rem > 0 {
            // Align the rem-bit value to the MSB of the byte.
            self.current = (val as u8) << (8 - rem);
            self.pos = rem;
        }

        Ok(())
    }

    /// Writes a signed integer using ZigZag encoding.
    ///
    /// This is equivalent to:
    /// `write_bits(encode_i64(value), n)`
    ///
    /// # Errors
    ///
    /// Same as [`write_bits`](Self::write_bits)s
    #[inline(always)]
    pub fn write_bits_signed(
        &mut self,
        value: i64,
        n: u8,
    ) -> Result<(), GorkaError> {
        self.write_bits(encode_i64(value), n)
    }

    /// Finalizes the writer and returns the underlying buffer.
    ///
    /// If there is a partially written byte, it is padded with zeros.
    pub fn finish(mut self) -> Vec<u8> {
        if self.pos > 0 {
            self.buf.push(self.current);
        }

        self.buf
    }

    /// Returns the total number of bits written.
    pub fn bit_len(&self) -> usize {
        self.buf.len() * 8 + self.pos as usize
    }

    /// Aligns the writer to the next byte boundary.
    ///
    /// If already aligned, this is a no-op.
    pub fn align_to_byte(&mut self) {
        if self.pos > 0 {
            self.buf.push(self.current);
            self.current = 0;
            self.pos = 0;
        }
    }

    /// Returns `true` if the writer is currently byte-aligned.
    pub fn is_aligned(&self) -> bool {
        self.pos == 0
    }
}

#[allow(deprecated)]
impl crate::bits::BitWrite for BitWriter {
    #[inline(always)]
    fn write_bit(
        &mut self,
        bit: bool,
    ) -> Result<(), GorkaError> {
        BitWriter::write_bit(self, bit);

        Ok(())
    }

    #[inline(always)]
    fn write_bits(
        &mut self,
        value: u64,
        n: u8,
    ) -> Result<(), GorkaError> {
        BitWriter::write_bits(self, value, n)
    }

    #[inline(always)]
    fn write_bits_signed(
        &mut self,
        value: i64,
        n: u8,
    ) -> Result<(), GorkaError> {
        BitWriter::write_bits_signed(self, value, n)
    }

    fn align_to_byte(&mut self) {
        BitWriter::align_to_byte(self);
    }

    fn bit_len(&self) -> usize {
        BitWriter::bit_len(self)
    }
}

#[allow(deprecated)]
impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(deprecated)]
#[cfg(test)]
mod tests {
    use alloc::vec;

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

    #[test]
    fn test_write_zero_bits_is_noop() {
        let mut w = BitWriter::new();

        w.write_bits(0, 0).unwrap();

        assert_eq!(w.bit_len(), 0);
        assert!(w.finish().is_empty());
    }

    #[test]
    fn test_fast_path_fits_in_current_byte() {
        // 3 bits used, then 3 more — should stay in same byte
        let mut w = BitWriter::new();

        w.write_bits(0b101, 3).unwrap(); // pos=3
        w.write_bits(0b110, 3).unwrap(); // fits in same byte: pos=6

        assert_eq!(w.bit_len(), 6);
        assert!(!w.is_aligned());
        assert_eq!(w.finish(), vec![0b1011_1000]);
    }

    #[test]
    fn test_fast_path_exactly_fills_byte() {
        let mut w = BitWriter::new();

        w.write_bits(0b101, 3).unwrap(); // pos=3
        w.write_bits(0b10101, 5).unwrap(); // fills exactly to byte boundary

        assert_eq!(w.bit_len(), 8);
        assert!(w.is_aligned());
        assert_eq!(w.finish(), vec![0b10110101]);
    }

    #[test]
    fn test_general_path_many_whole_bytes() {
        // Write 32 bits spanning 4+ bytes
        let mut w = BitWriter::new();

        w.write_bits(3, 2).unwrap(); // pos=2
        w.write_bits(0xDEAD_BEEF_u64, 32).unwrap(); // crosses many bytes

        let buf = w.finish();
        // Reconstruct manually to verify
        let mut w2 = BitWriter::new();

        w2.write_bits(3, 2).unwrap();

        for i in (0..32).rev() {
            w2.write_bit((0xDEAD_BEEF_u64 >> i) & 1 == 1);
        }

        assert_eq!(buf, w2.finish());
    }

    #[test]
    fn test_roundtrip_with_bit_reader() {
        use crate::BitReader;
        let mut w = BitWriter::new();

        w.write_bits(0b10110, 5).unwrap();
        w.write_bits(0b11001, 5).unwrap();
        w.write_bits(0b00111, 5).unwrap();

        let buf = w.finish();

        let mut r = BitReader::new(&buf);

        assert_eq!(r.read_bits(5).unwrap(), 0b10110);
        assert_eq!(r.read_bits(5).unwrap(), 0b11001);
        assert_eq!(r.read_bits(5).unwrap(), 0b00111);
    }

    #[test]
    fn test_bulk_write_matches_bitwise() {
        // Compare bulk write_bits with equivalent write_bit loop
        let values: &[(u64, u8)] = &[
            (0b1, 1),
            (0b101, 3),
            (0b1010_1010, 8),
            (0b11111111111, 11),
            (0xABCDEF, 24),
            (u64::MAX, 64),
        ];

        for &(val, n) in values {
            let mut bulk = BitWriter::new();

            bulk.write_bits(val, n).unwrap();

            let bulk_buf = bulk.finish();

            let mut bitwise = BitWriter::new();

            for i in (0..n).rev() {
                bitwise.write_bit((val >> i) & 1 == 1);
            }

            let bitwise_buf = bitwise.finish();

            assert_eq!(bulk_buf, bitwise_buf, "mismatch for val={val:#b} n={n}");
        }
    }

    #[test]
    fn test_signed_roundtrip() {
        use crate::BitReader;

        let mut w = BitWriter::new();

        w.write_bits_signed(-42, 16).unwrap();
        w.write_bits_signed(1_200, 16).unwrap();

        let buf = w.finish();
        let mut r = BitReader::new(&buf);

        assert_eq!(r.read_bits_signed(16).unwrap(), -42);
        assert_eq!(r.read_bits_signed(16).unwrap(), 1_200);
    }

    #[test]
    fn test_partial_then_full_then_partial() {
        // Stress test: partial byte → full bytes → partial byte
        let mut w = BitWriter::new();

        w.write_bits(0b111, 3).unwrap(); // partial: pos=3
        w.write_bits(0xDEADBEEF, 32).unwrap(); // crosses many bytes
        w.write_bits(0b10101, 5).unwrap(); // remainder

        let buf = w.finish();

        let mut w2 = BitWriter::new();

        for i in (0..3u8).rev() {
            w2.write_bit((0b111_u64 >> i) & 1 == 1);
        }

        for i in (0..32u8).rev() {
            w2.write_bit((0xDEADBEEF_u64 >> i) & 1 == 1);
        }

        for i in (0..5u8).rev() {
            w2.write_bit((0b10101_u64 >> i) & 1 == 1);
        }

        assert_eq!(buf, w2.finish());
    }
}
