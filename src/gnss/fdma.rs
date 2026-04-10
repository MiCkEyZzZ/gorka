//! GLONASS FDMA Doppler baseline tracking.
//!
//! GLONASS uses **Frequency Division Multiple Access**: each satellite
//! transmits on a unique carrier determined by its slot `k`:
//!
//! ```text
//! f_L1(k) = 1 602 + k × 0.5625  MHz   (k ∈ [-7, +6])
//! ```

use crate::{encode_i64, BitReader, BitWrite, GloSlot, GorkaError, MilliHz, RawBitWriter};

/// Number of GLONASS frequency slots (k ∈ [-7, +6]).
pub const N_SLOT: usize = 14;

/// EMA shift: α = 1 / (1 << EMA_SHIFT) = 1/128.
pub const EMA_SHIFT: u32 = 7;

/// GLONASS FDMA per-slot Doppler baseline state.
#[derive(Debug, Clone, Copy)]
pub struct FdmaState {
    baseline: [Option<i32>; N_SLOT],
}

impl FdmaState {
    /// Creates a new state with all baselines uninitialised.
    pub const fn new() -> Self {
        Self {
            baseline: [None; N_SLOT],
        }
    }

    /// Resets all baselines (e.g. at the start of a new chunk).
    pub fn reset(&mut self) {
        self.baseline = [None; N_SLOT];
    }

    /// Returns the current baseline for `slot`, or `None` if not yet seen.
    #[inline]
    pub fn baseline(
        &self,
        slot: GloSlot,
    ) -> Option<i32> {
        self.baseline[Self::idx(slot)]
    }

    /// Updates the EMA baseline with a new observation and returns the
    /// **residual** (observation - baseline) that should be encoded.
    pub fn update(
        &mut self,
        slot: GloSlot,
        observed: i32,
    ) -> i32 {
        let idx = Self::idx(slot);

        match self.baseline[idx] {
            None => {
                // First observation: seed baseline, residual = 0 (caller writes verbatim)
                self.baseline[idx] = Some(observed);
                0
            }
            Some(prev) => {
                // EMA: new = prev + (observed - prev) >> EMA_SHIFT
                let diff = (observed as i64 - prev as i64) >> EMA_SHIFT;
                let new_baseline = prev.wrapping_add(diff as i32);

                self.baseline[idx] = Some(new_baseline);

                // Residual relative to previous baseline (what we encode).
                observed.wrapping_sub(prev)
            }
        }
    }

    /// Reconstructs `observed` from a decode residual and advances the EMA.
    pub fn reconstruct(
        &mut self,
        slot: GloSlot,
        residual: i32,
    ) -> Result<i32, GorkaError> {
        let idx = Self::idx(slot);
        let prev = self.baseline[idx].ok_or(GorkaError::UnexpectedEof)?;
        let observed = prev.wrapping_add(residual);

        // Advance EMA the same way the encoder does.
        let diff = (observed as i64 - prev as i64) >> EMA_SHIFT;

        self.baseline[idx] = Some(prev.wrapping_add(diff as i32));

        Ok(observed)
    }

    #[inline]
    fn idx(slot: GloSlot) -> usize {
        debug_assert!((-7..=6).contains(&slot.get()));

        // GloSlot::MAX = 6 и прибавляе 1, что бы соблюсти условие
        (slot.get() + GloSlot::MAX + 1) as usize
    }
}

impl Default for FdmaState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn encoder_doppler_fdma(
    writer: &mut RawBitWriter,
    state: &mut FdmaState,
    slot: GloSlot,
    observed: i32,
) -> Result<(), GorkaError> {
    let idx = (slot.get() + GloSlot::MAX + 1) as usize;

    if state.baseline[idx].is_none() {
        // First observation: write verbatim, seed baseline.
        writer.write_bit(false)?;
        writer.write_bits(observed as u64 & 0xFFFF_FFFF, 32)?;

        state.baseline[idx] = Some(observed);

        return Ok(());
    }

    // Compute residual (delta from previous baseline) and advance EMA.
    let residual = state.update(slot, observed);
    let zz = encode_i64(residual as i64);

    if residual == 0 {
        writer.write_bits(0b10, 2)?; // '10'
    } else if zz < (1u64 << 14) {
        writer.write_bits(0b110, 3)?; // '110' + 14b
        writer.write_bits_signed(residual as i64, 14)?;
    } else {
        writer.write_bits(0b111, 3)?; // '111' + 32b
        writer.write_bits(observed as u64 & 0xFFFF_FFFF, 32)?;

        // Re-seed baseline on large jump.
        state.baseline[idx] = Some(observed);
    }

    Ok(())
}

pub fn decode_doppler_fdma(
    reader: &mut BitReader,
    state: &mut FdmaState,
    slot: GloSlot,
) -> Result<MilliHz, GorkaError> {
    let idx = (slot.get() + GloSlot::MAX + 1) as usize;

    if state.baseline[idx].is_none() {
        // First observation: verbatim, flag bit is '0'
        let _flag = reader.read_bit()?; // always false
        let raw = reader.read_bits(32)? as u32 as i32;

        state.baseline[idx] = Some(raw);

        return Ok(MilliHz(raw));
    }

    let b0 = reader.read_bit()?;
    let b1 = reader.read_bit()?;

    match (b0, b1) {
        // '10' - residual == 0
        (true, false) => {
            let prev = state.baseline[idx].unwrap();

            // EMA advance with zero residual = no change.
            Ok(MilliHz(prev))
        }
        // '11'
        (true, true) => {
            let b2 = reader.read_bit()?;

            if !b2 {
                // '110' + 14b zigzag residual
                let residual = reader.read_bits_signed(14)? as i32;
                let observed = state.reconstruct(slot, residual)?;

                Ok(MilliHz(observed))
            } else {
                // '111' + 32b verbatim (large jump, re-seed)
                let raw = reader.read_bits(32)? as u32 as i32;

                state.baseline[idx] = Some(raw);

                Ok(MilliHz(raw))
            }
        }
        _ => Err(GorkaError::UnexpectedEof),
    }
}
