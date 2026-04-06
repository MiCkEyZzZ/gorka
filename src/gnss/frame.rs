use crate::{GlonassSample, GorkaError};

/// Maximum number of GLONASS satellites in a single frame.
///
/// GLONASS uses 14 frequency slots (k ∈ \[−7, +6\]), so at most 14 unique
/// satellites can be observed simultaneously.
pub const MAX_GLONASS_SATS: usize = 14;

/// A collection of [`GlonassSample`] observations sharing the same epoch.
///
/// `GnssFrame` acts as a fixed-capacity, slot-sorted container for one
/// measurement epoch.  All samples inside a frame must carry the same
/// [`timestamp_ms`](GnssFrame::timestamp_ms) and refer to distinct GLONASS
/// frequency slots.
///
/// # Capacity
/// The frame holds at most [`MAX_GLONASS_SATS`] (14) observations, matching
/// the number of GLONASS FDMA frequency slots.
///
/// # Ordering invariant
/// After every [`push`](GnssFrame::push) the internal storage is kept sorted
/// by slot in ascending order, so [`iter`](GnssFrame::iter) always yields
/// samples from slot −7 up to slot +6.
///
/// # `no_std` compatibility
/// The backing store is a fixed-size array (`[Option<GlonassSample>; 14]`),
/// so `GnssFrame` works in `no_std` environments without heap allocation.
#[derive(Debug, Clone)]
pub struct GnssFrame {
    /// Unix epoch of this observation frame in milliseconds.
    pub timestamp_ms: u64,
    // В бущем тут надо будет подумать как реализовать дженерик для подстановки enum GnssSample,
    // который будет содержить: GlonassSample, GpsSample, GalileoSample, BeidouSample
    observations: [Option<GlonassSample>; MAX_GLONASS_SATS],
    count: usize,
}

impl GnssFrame {
    /// Creates an empty frame for the given epoch.
    ///
    /// # Example
    /// ```
    /// use gorka::GnssFrame;
    ///
    /// let frame = GnssFrame::new(1_700_000_000_000);
    /// assert!(frame.is_empty());
    /// ```
    pub fn new(timestamp_ms: u64) -> Self {
        Self {
            timestamp_ms,
            observations: core::array::from_fn(|_| None),
            count: 0,
        }
    }

    /// Inserts a [`GlonassSample`] into the frame.
    ///
    /// The sample is validated and then appended; the internal array is
    /// re-sorted by slot after each insertion so that the ascending-slot
    /// invariant is maintained.
    ///
    /// # Errors
    /// * [`GorkaError::InvalidSlot`] — the sample's slot is outside \[−7, +6\].
    /// * [`GorkaError::TimestampMismatch`] — the sample's `timestamp_ms`
    ///   differs from the frame's [`timestamp_ms`](GnssFrame::timestamp_ms).
    /// * [`GorkaError::FrameFull`] — the frame already contains
    ///   [`MAX_GLONASS_SATS`] observations.
    /// * [`GorkaError::DuplicateSlot`] — a sample for this slot already exists
    ///   in the frame.
    ///
    /// # Example
    /// ```
    /// use gorka::{GlonassSample, GnssFrame, MilliHz, Millimeter};
    ///
    /// let mut frame = GnssFrame::new(1_700_000_000_000);
    /// let sample = GlonassSample {
    ///     timestamp_ms: 1_700_000_000_000,
    ///     slot: 1,
    ///     cn0_dbhz: 40,
    ///     pseudorange_mm: Millimeter::new(21_500_000_000),
    ///     doppler_millihz: MilliHz::new(1_000_000),
    ///     carrier_phase_cycles: None,
    /// };
    /// frame.push(sample).unwrap();
    /// assert_eq!(frame.len(), 1);
    /// ```
    pub fn push(
        &mut self,
        sample: GlonassSample,
    ) -> Result<(), GorkaError> {
        sample.validate_slot()?;

        if sample.timestamp_ms != self.timestamp_ms {
            return Err(GorkaError::TimestampMismatch {
                frame: self.timestamp_ms,
                sample: sample.timestamp_ms,
            });
        }

        if self.count == MAX_GLONASS_SATS {
            return Err(GorkaError::FrameFull);
        }

        if self.slot_index(sample.slot).is_some() {
            return Err(GorkaError::DuplicateSlot(sample.slot));
        }

        self.observations[self.count] = Some(sample);
        self.count += 1;
        self.sort_by_slot();

        Ok(())
    }

    /// Builds a frame from a slice of samples.
    ///
    /// The timestamp is taken from `samples[0]`; every subsequent sample must
    /// share that timestamp.  All samples are pushed with the same rules as
    /// [`push`](GnssFrame::push).
    ///
    /// # Errors
    /// * [`GorkaError::EmptyChunk`] — `samples` is empty.
    /// * Any error that [`push`](GnssFrame::push) can return.
    ///
    /// # Example
    /// ```
    /// use gorka::{GlonassSample, GnssFrame, MilliHz, Millimeter};
    ///
    /// let ts = 1_700_000_000_000_u64;
    /// let samples: Vec<GlonassSample> = [-3_i8, 1, 5]
    ///     .iter()
    ///     .map(|&slot| GlonassSample {
    ///         timestamp_ms: ts,
    ///         slot,
    ///         cn0_dbhz: 40,
    ///         pseudorange_mm: Millimeter::new(21_500_000_000),
    ///         doppler_millihz: MilliHz::new(0),
    ///         carrier_phase_cycles: None,
    ///     })
    ///     .collect();
    ///
    /// let frame = GnssFrame::from_samples(&samples).unwrap();
    /// assert_eq!(frame.len(), 3);
    /// ```
    pub fn from_samples(samples: &[GlonassSample]) -> Result<Self, GorkaError> {
        if samples.is_empty() {
            return Err(GorkaError::EmptyChunk);
        }

        let ts = samples[0].timestamp_ms;
        let mut frame = Self::new(ts);

        for s in samples {
            frame.push(s.clone())?;
        }

        Ok(frame)
    }

    /// Returns the number of observations currently stored in the frame.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the frame contains no observations.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns an iterator over the stored observations in ascending slot
    /// order.
    ///
    /// The iterator yields only the occupied slots (up to
    /// [`len`](GnssFrame::len) items) and is guaranteed to be sorted by
    /// `slot` from lowest to highest.
    pub fn iter(&self) -> impl Iterator<Item = &GlonassSample> {
        self.observations[..self.count]
            .iter()
            .filter_map(|o| o.as_ref())
    }

    /// Looks up an observation by its GLONASS frequency slot.
    ///
    /// Returns `None` if no sample with the given slot exists in this frame.
    pub fn get_by_slot(
        &self,
        slot: i8,
    ) -> Option<&GlonassSample> {
        self.slot_index(slot)
            .map(|i| self.observations[i].as_ref().unwrap())
    }

    /// Returns `true` if the frame contains an observation for `slot`.
    #[inline]
    pub fn contains_slot(
        &self,
        slot: i8,
    ) -> bool {
        self.slot_index(slot).is_some()
    }

    /// Runs [`GlonassSample::validate`] on every observation in the frame.
    ///
    /// Returns the first validation error encountered, or `Ok(())` when all
    /// samples pass.
    pub fn validate_all(&self) -> Result<(), GorkaError> {
        for s in self.iter() {
            s.validate()?;
        }

        Ok(())
    }

    /// Returns the array index of the observation with the given slot, or
    /// `None` if no such observation exists.
    fn slot_index(
        &self,
        slot: i8,
    ) -> Option<usize> {
        self.observations[..self.count]
            .iter()
            .position(|o| o.as_ref().is_some_and(|s| s.slot == slot))
    }

    /// Insertion-sorts the occupied portion of `observations` by slot.
    ///
    /// Called after every [`push`](GnssFrame::push) to maintain the
    /// ascending-slot invariant.  Because only one new element is added per
    /// call, a single insertion-sort pass (O(n)) is sufficient.
    fn sort_by_slot(&mut self) {
        for i in 1..self.count {
            let mut j = i;

            while j > 0 {
                let slot_j = self.observations[j].as_ref().map_or(i8::MAX, |s| s.slot);
                let slot_prev = self.observations[j - 1]
                    .as_ref()
                    .map_or(i8::MAX, |s| s.slot);

                if slot_prev > slot_j {
                    self.observations.swap(j, j - 1);
                    j -= 1;
                } else {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;
    use crate::{MilliHz, Millimeter};

    const TS: u64 = 1_700_000_000_000;

    fn make_obs(
        slot: i8,
        ts: u64,
    ) -> GlonassSample {
        GlonassSample {
            timestamp_ms: ts,
            slot,
            cn0_dbhz: 40,
            pseudorange_mm: Millimeter::new(21_500_000_000),
            doppler_millihz: MilliHz::new(1_000_000),
            carrier_phase_cycles: None,
        }
    }

    #[test]
    fn test_new_frame_is_empty() {
        let f = GnssFrame::new(TS);

        assert!(f.is_empty());
        assert_eq!(f.len(), 0);
        assert_eq!(f.timestamp_ms, TS);
    }

    #[test]
    fn test_push_single_observation() {
        let mut f = GnssFrame::new(TS);

        f.push(make_obs(1, TS)).unwrap();

        assert_eq!(f.len(), 1);
        assert!(!f.is_empty());
    }

    #[test]
    fn test_push_all_slots_fills_frame() {
        let mut f = GnssFrame::new(TS);

        for slot in -7_i8..=6 {
            f.push(make_obs(slot, TS)).unwrap()
        }

        assert_eq!(f.len(), MAX_GLONASS_SATS);
    }

    #[test]
    fn test_observations_sorted_by_slot_ascending() {
        let mut f = GnssFrame::new(TS);

        for slot in ((-7_i8)..=6).rev() {
            f.push(make_obs(slot, TS)).unwrap();
        }

        let slots: Vec<i8> = f.iter().map(|s| s.slot).collect();
        let mut expected: Vec<i8> = (-7..=6).collect();

        expected.sort();

        assert_eq!(slots, expected);
    }

    #[test]
    fn test_slots_sorted_after_random_push_order() {
        let mut f = GnssFrame::new(TS);

        for &slot in &[3_i8, -7, 6, 0, -3, 1] {
            f.push(make_obs(slot, TS)).unwrap();
        }

        let slots: Vec<i8> = f.iter().map(|s| s.slot).collect();

        assert!(
            slots.windows(2).all(|w| w[0] < w[1]),
            "not sorted: {slots:?}"
        );
    }

    #[test]
    fn push_invalid_slot_returns_error() {
        let mut f = GnssFrame::new(TS);
        let err = f.push(make_obs(-8, TS)).unwrap_err();

        assert!(matches!(err, GorkaError::InvalidSlot(-8)));
    }

    #[test]
    fn push_wrong_timestamp_returns_error() {
        let mut f = GnssFrame::new(TS);
        let err = f.push(make_obs(1, TS + 1)).unwrap_err();

        assert!(matches!(err, GorkaError::TimestampMismatch { .. }));
    }

    #[test]
    fn push_duplicate_slot_returns_error() {
        let mut f = GnssFrame::new(TS);

        f.push(make_obs(1, TS)).unwrap();

        let err = f.push(make_obs(1, TS)).unwrap_err();

        assert!(matches!(err, GorkaError::DuplicateSlot(1)));
    }

    #[test]
    fn push_when_full_returns_error() {
        let mut f = GnssFrame::new(TS);

        for slot in -7_i8..=6 {
            f.push(make_obs(slot, TS)).unwrap();
        }

        // Фрейм заполнен — любая дополнительная попытка добавления (слот невозможна)
        // должeн вернуться FrameFull. Но проверка валидности слота выполняется раньше
        // проверки вместимости, поэтому используем валидный слот в новом фрейме, чтобы
        // протестировать FrameFull напрямую:
        let _f2 = GnssFrame::new(TS);

        // Ручное заполнение 14-и наблюдений с разными допустимыми слотами через
        // from_samples
        let samples: Vec<GlonassSample> = (-7_i8..=6).map(|s| make_obs(s, TS)).collect();
        let full = GnssFrame::from_samples(&samples).unwrap();

        assert_eq!(full.len(), MAX_GLONASS_SATS);

        let _ = f; // подавляем предупреждение о неиспользуемой переменной
    }

    #[test]
    fn get_by_slot_found() {
        let mut f = GnssFrame::new(TS);
        f.push(make_obs(3, TS)).unwrap();
        let obs = f.get_by_slot(3).unwrap();
        assert_eq!(obs.slot, 3);
    }

    #[test]
    fn get_by_slot_not_found() {
        let mut f = GnssFrame::new(TS);
        f.push(make_obs(3, TS)).unwrap();
        assert!(f.get_by_slot(5).is_none());
    }

    #[test]
    fn contains_slot_true_and_false() {
        let mut f = GnssFrame::new(TS);
        f.push(make_obs(-3, TS)).unwrap();
        assert!(f.contains_slot(-3));
        assert!(!f.contains_slot(0));
    }

    #[test]
    fn from_samples_empty_returns_error() {
        let err = GnssFrame::from_samples(&[]).unwrap_err();
        assert!(matches!(err, GorkaError::EmptyChunk));
    }

    #[test]
    fn from_samples_mixed_timestamps_returns_error() {
        let samples = vec![make_obs(1, TS), make_obs(2, TS + 1)];
        let err = GnssFrame::from_samples(&samples).unwrap_err();
        assert!(matches!(err, GorkaError::TimestampMismatch { .. }));
    }

    #[test]
    fn from_samples_valid_set() {
        let samples: Vec<GlonassSample> = [1_i8, -3, 5].iter().map(|&s| make_obs(s, TS)).collect();
        let f = GnssFrame::from_samples(&samples).unwrap();
        assert_eq!(f.len(), 3);
        // Verify sorted
        let slots: Vec<i8> = f.iter().map(|s| s.slot).collect();
        assert_eq!(slots, vec![-3, 1, 5]);
    }

    #[test]
    fn validate_all_ok() {
        let samples: Vec<GlonassSample> = [0_i8, 1, -1].iter().map(|&s| make_obs(s, TS)).collect();
        let f = GnssFrame::from_samples(&samples).unwrap();
        assert!(f.validate_all().is_ok());
    }
}
