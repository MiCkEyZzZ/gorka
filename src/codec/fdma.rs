//! GLONASS FDMA Doppler baseline tracking.
//!
//! GLONASS uses **Frequency Division Multiple Access**: each satellite
//! transmits on a unique carrier determined by its slot `k`:
//!
//! ```text
//! f_L1(k) = 1 602 + k × 0.5625  MHz   (k ∈ [-7, +6])
//! ```
//!
//! Because every slot has a *different* carrier, the Doppler frequency observed
//! on the ground contains a per-slot **frequency offset**.  When observations
//! alternate across slots — e.g. slot -3, +1, -3, +1, … — a naïve inter-sample
//! delta would include the large inter-slot frequency difference every time.
//!
//! # Solution — per-slot EMA baseline
//!
//! [`FdmaState`] maintains a separate Doppler baseline per slot.  The baseline
//! is updated via a low-pass **Exponential Moving Average** (EMA) with
//! α = 1/128, implemented as a bitshift to stay embedded-friendly:
//!
//! ```text
//! baseline_new = baseline_old + (observed - baseline_old) >> 7
//! ```
//!
//! The value encoded in the bitstream is the **residual**:
//!
//! ```text
//! residual = observed − baseline_old
//! ```
//!
//! This keeps residuals small even for interleaved multi-slot streams.
//!
//! # CDMA comparison
//!
//! GPS, Galileo and BeiDou all share a single carrier per band — no
//! per-satellite frequency offset.  See [`crate::codec::cdma`] for the simpler
//! CDMA path.

use crate::{
    encode_i64, BitReader, BitWrite, DopplerCodec, GloSlot, GorkaError, MilliHz, RawBitWriter,
};

/// Number of GLONASS frequency slots (k ∈ [-7, +6]).
pub const N_SLOT: usize = 14;

/// EMA shift: α = 1 / (1 << EMA_SHIFT) = 1/128.
///
/// Large value -> slower convergence, smaller residuals one converged.
/// 128 epochs at 1 Hz ≈ 2 minutes to reach ~63% of the true value (one
/// time-constant of the first-order IIR filter).
pub const EMA_SHIFT: u32 = 7;

/// Bits used for small residual encoding (zigzag residual path)
const FDMA_RESIDUAL_BITS: u8 = 14;

/// Bits used for full verbatim fallback encoding
const FDMA_VERBATIM_BITS: u8 = 32;

/// GLONASS FDMA per-slot Doppler baseline state.
///
/// Stores one EMA baseline per slot in millihertz.  `None` means the slot
/// has not yet been observed in this chunk.
///
/// The struct is intentionally small (14 × 4 = 56 bytes + 14 option tags)
/// and `Copy`-able so it can be embedded in encoder/decoder state without
/// heap allocation.
#[derive(Debug, Clone, Copy)]
pub struct FdmaState {
    /// Per-slot EMA baseline in millihertz.  `None` = not yet seen.
    baseline: [Option<MilliHz>; N_SLOT],
}

pub struct FdmaCodec;

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
    ) -> Option<MilliHz> {
        self.baseline[Self::idx(slot)]
    }

    /// Updates the EMA baseline with a new observation and returns the
    /// **residual** (observation - baseline_old) that should be encoded.
    ///
    /// If this is the first observation for the slot, the baseline is
    /// seeded with `observed` and the returned residual is `0` — the
    /// caller writes a verbatim value instead.
    pub fn update(
        &mut self,
        slot: GloSlot,
        observed: MilliHz,
    ) -> MilliHz {
        let idx = Self::idx(slot);

        match self.baseline[idx] {
            None => {
                // First observation: seed baseline, residual = 0.
                // Caller writes verbatim.
                self.baseline[idx] = Some(observed);
                MilliHz(0)
            }
            Some(prev) => {
                // EMA: new = prev + (observed - prev) >> EMA_SHIFT
                let diff = (observed.0 as i64 - prev.0 as i64) >> EMA_SHIFT;
                let new_baseline = MilliHz((prev.0 as i64 + diff) as i32);

                self.baseline[idx] = Some(new_baseline);

                // Residual relative to previous baseline (what we encode).
                MilliHz(observed.0.wrapping_sub(prev.0))
            }
        }
    }

    /// Reconstructs `observed` from a decoded residual and advances the EMA.
    ///
    /// Must be called in the same order as [`Self::update`] was called on the
    /// encoder side.
    pub fn reconstruct(
        &mut self,
        slot: GloSlot,
        residual: MilliHz,
    ) -> Result<MilliHz, GorkaError> {
        let idx = Self::idx(slot);
        let prev = self.baseline[idx].ok_or(GorkaError::UnexpectedEof)?;
        let observed = MilliHz(prev.0.wrapping_add(residual.0));

        // Advance EMA identically to the encoder.
        let diff = ((observed.0 as i64 - prev.0 as i64) >> EMA_SHIFT) as i32;

        self.baseline[idx] = Some(MilliHz(prev.0.wrapping_add(diff)));

        Ok(observed)
    }

    /// Convert a GLONASS slots to an array index (slot + 7 ∈ 0..14).
    #[inline]
    fn idx(slot: GloSlot) -> usize {
        debug_assert!((-7..=6).contains(&slot.get()));

        // GloSlot::MAX = 6 и прибавляе 1, что бы соблюсти условие
        (slot.get() + GloSlot::MAX + 1) as usize
    }
}

impl DopplerCodec for FdmaCodec {
    type State = FdmaState;
    type Id = GloSlot;

    fn encode(
        writer: &mut RawBitWriter,
        state: &mut Self::State,
        value: MilliHz,
        id: Self::Id,
    ) -> Result<(), GorkaError> {
        let idx = FdmaState::idx(id);

        if state.baseline[idx].is_none() {
            writer.write_bit(false)?;
            writer.write_bits(value.0 as u64 & 0xFFFF_FFFF, 32)?;

            state.baseline[idx] = Some(value);

            return Ok(());
        }

        // Compute residual (delta from previous baseline) and advance EMA.
        let prev = state.baseline[idx].unwrap();
        let residual = MilliHz(value.0.wrapping_sub(prev.0));
        let encoded = encode_i64(residual.0 as i64);

        if residual.0 == 0 {
            writer.write_bits(0b10, 2)?; // '10'

            state.update(id, value);
        } else if encoded < (1u64 << FDMA_RESIDUAL_BITS) {
            writer.write_bits(0b110, 3)?; // '110' + 14b
            writer.write_bits_signed(residual.0 as i64, FDMA_RESIDUAL_BITS)?;

            state.update(id, value);
        } else {
            writer.write_bits(0b111, 3)?; // '111' + 32b
            writer.write_bits(value.0 as u64 & 0xFFFF_FFFF, FDMA_VERBATIM_BITS)?;

            // Re-seed baseline on large jump to avoid compounding error.
            state.baseline[idx] = Some(value);
        }

        Ok(())
    }

    fn decode(
        reader: &mut BitReader,
        state: &mut Self::State,
        id: Self::Id,
    ) -> Result<MilliHz, GorkaError> {
        let idx = FdmaState::idx(id);

        if state.baseline[idx].is_none() {
            // First observation: verbatim, flag bit is '0'
            let flag = reader.read_bit()?;

            debug_assert!(!flag);

            let raw = reader.read_bits(32)? as u32 as i32;
            let obs = MilliHz(raw);

            state.baseline[idx] = Some(obs);

            return Ok(obs);
        }

        let b0 = reader.read_bit()?;
        let b1 = reader.read_bit()?;

        match (b0, b1) {
            // '10' — residual == 0, baseline doesn't change
            (true, false) => {
                let prev = state.baseline[idx].unwrap();

                state.update(id, prev);

                // EMA advance with zero residual = no change.
                Ok(prev)
            }
            // '11x'
            (true, true) => {
                let b2 = reader.read_bit()?;

                if !b2 {
                    // '110' + 14b zigzag residual
                    let residual = MilliHz(reader.read_bits_signed(FDMA_RESIDUAL_BITS)? as i32);
                    let obs = state.reconstruct(id, residual)?;

                    Ok(obs)
                } else {
                    // '111' + 32b verbatim (large jump, re-seed)
                    let raw = reader.read_bits(FDMA_VERBATIM_BITS)? as u32 as i32;
                    let obs = MilliHz(raw);

                    state.baseline[idx] = Some(obs);

                    Ok(obs)
                }
            }
            _ => Err(GorkaError::UnexpectedEof),
        }
    }
}

impl Default for FdmaState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn slot(k: i8) -> GloSlot {
        GloSlot::new(k).unwrap()
    }

    fn fdma_roundtrip(values: &[(GloSlot, i32)]) -> alloc::vec::Vec<i32> {
        let mut buf = vec![0u8; 4096];
        let mut enc_state = FdmaState::new();
        let mut writer = RawBitWriter::new(&mut buf);

        for &(s, v) in values {
            FdmaCodec::encode(&mut writer, &mut enc_state, MilliHz(v), s).unwrap();
        }

        let n = writer.bytes_written();
        let mut reader = BitReader::new(&buf[..n]);
        let mut dec_state = FdmaState::new();
        let mut out = vec![];

        for &(s, _) in values {
            let v = FdmaCodec::decode(&mut reader, &mut dec_state, s).unwrap();

            out.push(v.0);
        }

        out
    }

    #[test]
    fn test_fdma_state_new_all_none() {
        let s = FdmaState::default();

        for k in -7_i8..=6 {
            assert_eq!(s.baseline(slot(k)), None, "slot {k}");
        }
    }

    #[test]
    fn test_fdma_state_reset() {
        let mut s = FdmaState::default();

        s.update(slot(0), MilliHz(1_200_000));
        s.reset();

        assert_eq!(s.baseline(slot(0)), None);
    }

    #[test]
    fn test_fdma_first_observation_seeds_baseline() {
        let mut s = FdmaState::default();
        let residual = s.update(slot(1), MilliHz(1_200_500));

        assert_eq!(residual, MilliHz(0));
        assert_eq!(s.baseline(slot(1)), Some(MilliHz(1_200_500)));
    }

    #[test]
    fn test_fdma_ema_one_step() {
        // seed at 0, observe 128 → diff = 128 >> 7 = 1 → baseline = 1
        let mut s = FdmaState::default();

        s.update(slot(0), MilliHz(0));
        s.update(slot(0), MilliHz(128));

        assert_eq!(s.baseline(slot(0)), Some(MilliHz(1)));
    }

    #[test]
    fn test_fdma_ema_converges() {
        let mut s = FdmaState::default();
        let target = MilliHz(1_200_000i32);

        s.update(slot(0), MilliHz(0));

        for _ in 0..1024 {
            s.update(slot(0), target);
        }

        let b = s.baseline(slot(0)).unwrap();
        let err = (b.0 - target.0).abs();

        assert!(
            err < target.0 / 100,
            "baseline {} did not converge to {} (err={})",
            b.0,
            target.0,
            err
        );
    }

    #[test]
    fn test_fdma_independent_slots() {
        let mut s = FdmaState::default();

        s.update(slot(-7), MilliHz(1_000_000));
        s.update(slot(6), MilliHz(2_000_000));

        assert_eq!(s.baseline(slot(-7)), Some(MilliHz(1_000_000)));
        assert_eq!(s.baseline(slot(6)), Some(MilliHz(2_000_000)));
    }

    #[test]
    fn test_baseline_convergence_rate() {
        // After 128 steps, baseline should exceed 63% of target
        let mut s = FdmaState::default();
        let target = MilliHz(1_200_000i32);

        s.update(slot(0), MilliHz(0));

        for _ in 0..128 {
            s.update(slot(0), target);
        }

        let baseline = s.baseline(slot(0)).unwrap();

        let fraction = baseline.0 as f64 / target.0 as f64;

        assert!(
            fraction > 0.60,
            "baseline fraction after 128 steps: {fraction:.3}"
        );
    }

    #[test]
    fn test_fdma_roundtrip_single() {
        assert_eq!(fdma_roundtrip(&[(slot(1), 1_200_500)]), [1_200_500]);
    }

    #[test]
    fn test_fdma_roundtrip_constant() {
        let vals: alloc::vec::Vec<_> = (0..32).map(|_| (slot(0), 1_200_000i32)).collect();
        let d = fdma_roundtrip(&vals);

        assert!(d.iter().all(|&v| v == 1_200_000));
    }

    #[test]
    fn test_fdma_roundtrip_smooth() {
        let vals: alloc::vec::Vec<_> = (0..64).map(|i| (slot(1), 1_200_000 + i * 10)).collect();
        let exp: alloc::vec::Vec<_> = vals.iter().map(|&(_, v)| v).collect();

        assert_eq!(fdma_roundtrip(&vals), exp);
    }

    #[test]
    fn test_fdma_roundtrip_interleaved() {
        let vals: alloc::vec::Vec<_> = (0..40)
            .map(|i| {
                let s = if i % 2 == 0 { slot(-3) } else { slot(3) };
                let base = if i % 2 == 0 { 1_000_000i32 } else { 1_600_000 };
                (s, base + i * 5)
            })
            .collect();
        let exp: alloc::vec::Vec<_> = vals.iter().map(|&(_, v)| v).collect();

        assert_eq!(fdma_roundtrip(&vals), exp);
    }

    #[test]
    fn test_fdma_roundtrip_all_slots() {
        let vals: alloc::vec::Vec<_> = (-7_i8..=6)
            .flat_map(|k| (0..8).map(move |i| (slot(k), 1_200_000 + k as i32 * 562 + i * 20)))
            .collect();
        let exp: alloc::vec::Vec<_> = vals.iter().map(|&(_, v)| v).collect();

        assert_eq!(fdma_roundtrip(&vals), exp);
    }

    #[test]
    fn test_fdma_roundtrip_large_jump() {
        let vals = [
            (slot(0), 1_200_000i32),
            (slot(0), 1_200_000),
            (slot(0), 3_000_000),
        ];

        assert_eq!(fdma_roundtrip(&vals), [1_200_000, 1_200_000, 3_000_000]);
    }
}
