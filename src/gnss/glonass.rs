use crate::error::GorkaError;

/// One GLONASS telemetry observation.
///
/// `slot` is the FDMA frequency slot k ∈ [-7, +6].
/// Carrier frequency = 1602 + k * 0.5625 Mhz.
#[derive(Debug, Clone, PartialEq)]
pub struct GlonassSample {
    /// Unix timestamp in milliseconds
    pub timestamp_ms: u64,
    /// GLONASS frequency slot: k ∈ [-7, +6]
    pub slot: i8,
    /// Carrier-to-noise density [dBHz], typical range 30-50
    pub cn0_dbhz: u8,
    /// Pseudorange [m], typical range 20_000_000-26_000_000
    pub pseudorange_m: u32,
    /// Dopler shift [Hz], typical range ±4000 (slot-dependent)
    pub doppler_hz: i16,
}

impl GlonassSample {
    pub fn validate_slot(&self) -> Result<(), GorkaError> {
        if !(-7..=6).contains(&self.slot) {
            return Err(GorkaError::InvalidSlot(self.slot));
        }

        Ok(())
    }
}
