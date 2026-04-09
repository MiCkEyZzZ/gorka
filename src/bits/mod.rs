//! Bit-level I/O primitives.
//!
//! This module provides low-level utilities for reading and writing
//! bitstreams in **MSB-first** (most significant bit first) order.
//!
//! ## Overview
//!
//! - [`BitReader`] — bit-level reader over `&[u8]`
//! - [`RawBitWriter`] — zero-copy writer over `&mut [u8]` (no allocation)
//! - [`BitWrite`] — generic trait for writing bits

pub mod raw_writer;
pub mod reader;

pub use raw_writer::*;
pub use reader::*;

use crate::GorkaError;

/// A trait for writing bits in **MSB-first** order.
///
/// This trait abstracts over different bit-level writers,
/// allowing generic encoding logic.
pub trait BitWrite {
    /// Writes a single bit.
    fn write_bit(
        &mut self,
        bit: bool,
    ) -> Result<(), GorkaError>;

    /// Writes the lowest `n` bits of `value`.
    ///
    /// Bits are written in MSB-first order.
    fn write_bits(
        &mut self,
        value: u64,
        n: u8,
    ) -> Result<(), GorkaError>;

    /// Writes a signed integer using `n` bits.
    ///
    /// Typically uses ZigZag encoding internally.
    fn write_bits_signed(
        &mut self,
        value: i64,
        n: u8,
    ) -> Result<(), GorkaError>;

    /// Pads with zero bits until byte-aligned.
    fn align_to_byte(&mut self);

    /// Returns total number of bits written.
    fn bit_len(&self) -> usize;

    /// Returns `true` if writer is byte-aligned.
    fn is_aligned(&self) -> bool {
        self.bit_len() % 8 == 0
    }
}
