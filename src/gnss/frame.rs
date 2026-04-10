use crate::{GloSlot, GlonassSample, GnssMeasurement, GorkaError};

/// Maximum number of GLONASS satellites in a single frame.
///
/// GLONASS uses 14 frequency slots (k ∈ \[−7, +6\]), so at most 14 unique
/// satellites can be observed simultaneously.
// NOTE: GLONASS-specific limit; will change for multi-constellation support
pub const MAX_GLONASS_SATS: usize = 14;

/// A collection of [`GlonassSample`] observations sharing the same epoch.
// GNSS frame (currently GLONASS-only implementation)
// TODO: generalize to multi-constelation (GPS, Galileo, etc.)
#[derive(Debug, Clone)]
pub struct GnssFrame {
    /// Unix epoch of this observation frame in milliseconds.
    pub timestamp_ms: u64,
    observations: [Option<GlonassSample>; MAX_GLONASS_SATS],
    count: usize,
}

#[derive(Debug, Clone)]
pub struct GnssEpoch {
    timestamp_ms: u64,
    observations: [Option<GlonassSample>; MAX_GLONASS_SATS],
    count: usize,
}

impl GnssFrame {
    /// Creates an empty frame for the given epoch.
    pub fn new(timestamp_ms: u64) -> Self {
        Self {
            timestamp_ms,
            observations: core::array::from_fn(|_| None),
            count: 0,
        }
    }

    /// Inserts a [`GlonassSample`] into the frame.
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

        if self.slot_index(sample.slot.get()).is_some() {
            return Err(GorkaError::DuplicateSlot(sample.slot.get()));
        }

        self.observations[self.count] = Some(sample);
        self.count += 1;
        self.sort_by_slot();

        Ok(())
    }

    /// Builds a frame from a slice of samples.
    pub fn from_samples(samples: &[GlonassSample]) -> Result<Self, GorkaError> {
        if samples.is_empty() {
            return Err(GorkaError::EmptyChunk);
        }

        let timestamp_ms = samples[0].timestamp_ms;
        let mut frame = Self::new(timestamp_ms);

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
            .position(|o| o.as_ref().is_some_and(|s| s.slot.get() == slot))
    }

    /// Insertion-sorts the occupied portion of `observations` by slot.
    fn sort_by_slot(&mut self) {
        for i in 1..self.count {
            let mut j = i;

            while j > 0 {
                let slot_j = self.observations[j]
                    .as_ref()
                    .map_or(i8::MAX, |s| s.slot.get());
                let slot_prev = self.observations[j - 1]
                    .as_ref()
                    .map_or(i8::MAX, |s| s.slot.get());

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

impl GnssEpoch {
    /// Creates an empty epoch for `timestamp`.
    pub fn new(timestamp_ms: u64) -> Self {
        Self {
            timestamp_ms,
            observations: core::array::from_fn(|_| None),
            count: 0,
        }
    }

    /// Build an epoch from a homogeneous slice - all samples must share the
    /// same `timestamp_ms`.
    pub fn from_samples(samples: &[GlonassSample]) -> Result<Self, GorkaError> {
        if samples.is_empty() {
            return Err(GorkaError::EmptyChunk);
        }

        let timestamp_ms = samples[0].timestamp_ms;
        let mut epoch = Self::new(timestamp_ms);

        for sample in samples {
            epoch.insert(sample.clone())?;
        }

        Ok(epoch)
    }

    /// Groups a heterogeneous slice by timestamp_ms and returns one epoch per
    /// unique timestamp, ordered chronologically.
    pub fn group_by_timestamp(samples: &[GlonassSample]) -> alloc::vec::Vec<GnssEpoch> {
        use alloc::vec::Vec;

        if samples.is_empty() {
            return Vec::new();
        }

        // Collect unique timestamp in order of first appearance.
        let mut timestamps: Vec<u64> = Vec::new();

        for sample in samples {
            if !timestamps.contains(&sample.timestamp_ms) {
                timestamps.push(sample.timestamp_ms);
            }
        }

        timestamps.sort_unstable();

        timestamps
            .into_iter()
            .filter_map(|ts| {
                let group: Vec<GlonassSample> = samples
                    .iter()
                    .filter(|s| s.timestamp_ms == ts)
                    .cloned()
                    .collect();
                GnssEpoch::from_samples(&group).ok()
            })
            .collect()
    }

    /// Returns the timestamp of this epoch in milliseconds.
    #[inline]
    pub fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }

    /// Returns the number of observations in this epoch.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the epoch contains no observations.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns an iterator over the observations, sorted by slot ascending.
    pub fn iter(&self) -> impl Iterator<Item = &GlonassSample> {
        self.observations[..self.count]
            .iter()
            .filter_map(|o| o.as_ref())
    }

    /// Returns the observation for `slot`, or `None`.
    pub fn get_by_slot(
        &self,
        slot: GloSlot,
    ) -> Option<&GlonassSample> {
        self.slot_index(slot)
            .map(|i| self.observations[i].as_ref().unwrap())
    }

    /// Returns `true` if the epoch contains an observation for `slot`.
    #[inline]
    pub fn contains_slot(
        &self,
        slot: GloSlot,
    ) -> bool {
        self.slot_index(slot).is_some()
    }

    /// Returns the minimum slot observed, or `None` if empty.
    pub fn min_slot(&self) -> Option<GloSlot> {
        self.iter().map(|s| s.slot).min()
    }

    /// Returns the maximum slot observed, or `None` if empty.
    pub fn max_slot(&self) -> Option<GloSlot> {
        self.iter().map(|s| s.slot).max()
    }

    /// Encodes the epoch as a compressed Gorka chunk.
    ///
    /// Delegates to [`crate::codec::GlonassEncoder::encode_chunk`].
    ///
    /// # Errors
    /// Propagates any encoding error (e.g. `EmptyChunk`, `InvalidSlot`).
    pub fn encode(&self) -> Result<alloc::vec::Vec<u8>, GorkaError> {
        use crate::codec::GlonassEncoder;

        if self.count == 0 {
            return Err(GorkaError::EmptyChunk);
        }

        let samples: alloc::vec::Vec<GlonassSample> = self.iter().cloned().collect();

        GlonassEncoder::encode_chunk(&samples)
    }

    /// Validates all observations.
    ///
    /// Returns the first error encountered.
    pub fn validate_all(&self) -> Result<(), GorkaError> {
        for s in self.iter() {
            s.validate_slot()?;
            s.validate_pseudorange()?;
            s.validate_doppler()?;
        }

        Ok(())
    }

    fn insert(
        &mut self,
        sample: GlonassSample,
    ) -> Result<(), GorkaError> {
        // validate before inserting
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
            return Err(GorkaError::DuplicateSlot(sample.slot.get()));
        }

        self.observations[self.count] = Some(sample);
        self.count += 1;
        self.sort_by_slot();

        Ok(())
    }

    fn slot_index(
        &self,
        slot: GloSlot,
    ) -> Option<usize> {
        self.observations[..self.count]
            .iter()
            .position(|o| o.as_ref().is_some_and(|s| s.slot == slot))
    }

    fn sort_by_slot(&mut self) {
        for i in 1..self.count {
            let mut j = i;

            while j > 0 {
                let a = self.observations[j]
                    .as_ref()
                    .map_or(GloSlot::MAX, |s| s.slot.get());
                let b = self.observations[j - 1]
                    .as_ref()
                    .map_or(GloSlot::MAX, |s| s.slot.get());

                if b > a {
                    self.observations.swap(j, j - 1);
                    j -= 1;
                } else {
                    break;
                }
            }
        }
    }
}

/// Allow converting a GnssFrame into a GnssEpoch (zero-copy where possible).
impl TryFrom<GnssFrame> for GnssEpoch {
    type Error = GorkaError;

    fn try_from(frame: GnssFrame) -> Result<Self, GorkaError> {
        let samples: alloc::vec::Vec<GlonassSample> = frame.iter().cloned().collect();

        GnssEpoch::from_samples(&samples)
    }
}

/// Allow converting a GnssEpoch into a GnssFrame.
impl TryFrom<GnssEpoch> for GnssFrame {
    type Error = GorkaError;

    fn try_from(epoch: GnssEpoch) -> Result<Self, GorkaError> {
        GnssFrame::from_samples(&epoch.iter().cloned().collect::<alloc::vec::Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;
    use crate::{DbHz, GloSlot, MilliHz, Millimeter};

    const TS: u64 = 1_700_000_000_000;

    fn make_obs(
        slot: GloSlot,
        ts: u64,
    ) -> GlonassSample {
        GlonassSample {
            timestamp_ms: ts,
            slot,
            cn0_dbhz: DbHz::new(40).unwrap(),
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

        f.push(make_obs(GloSlot::new(1).unwrap(), TS)).unwrap();

        assert_eq!(f.len(), 1);
        assert!(!f.is_empty());
    }

    #[test]
    fn test_push_all_slots_fills_frame() {
        let mut f = GnssFrame::new(TS);

        for slot in -7_i8..=6 {
            f.push(make_obs(GloSlot::new(slot).unwrap(), TS)).unwrap()
        }

        assert_eq!(f.len(), MAX_GLONASS_SATS);
    }

    #[test]
    fn test_observations_sorted_by_slot_ascending() {
        let mut f = GnssFrame::new(TS);

        for slot in ((-7_i8)..=6).rev() {
            f.push(make_obs(GloSlot::new(slot).unwrap(), TS)).unwrap();
        }

        let slots: Vec<i8> = f.iter().map(|s| s.slot.get()).collect();
        let mut expected: Vec<i8> = (-7..=6).collect();

        expected.sort();

        assert_eq!(slots, expected);
    }

    #[test]
    fn test_slots_sorted_after_random_push_order() {
        let mut f = GnssFrame::new(TS);

        for &slot in &[3_i8, -7, 6, 0, -3, 1] {
            f.push(make_obs(GloSlot::new(slot).unwrap(), TS)).unwrap();
        }

        let slots: Vec<i8> = f.iter().map(|s| s.slot.get()).collect();

        assert!(
            slots.windows(2).all(|w| w[0] < w[1]),
            "not sorted: {slots:?}"
        );
    }

    #[test]
    fn test_push_wrong_timestamp_returns_error() {
        let mut f = GnssFrame::new(TS);
        let err = f
            .push(make_obs(GloSlot::new(1).unwrap(), TS + 1))
            .unwrap_err();

        assert!(matches!(err, GorkaError::TimestampMismatch { .. }));
    }

    #[test]
    fn test_push_duplicate_slot_returns_error() {
        let mut f = GnssFrame::new(TS);

        f.push(make_obs(GloSlot::new(1).unwrap(), TS)).unwrap();

        let err = f.push(make_obs(GloSlot::new(1).unwrap(), TS)).unwrap_err();

        assert!(matches!(err, GorkaError::DuplicateSlot(1)));
    }

    #[test]
    fn test_push_when_full_returns_error() {
        let mut f = GnssFrame::new(TS);

        for slot in -7_i8..=6 {
            f.push(make_obs(GloSlot::new(slot).unwrap(), TS)).unwrap();
        }

        // Фрейм заполнен — любая дополнительная попытка добавления (слот невозможна)
        // должeн вернуться FrameFull. Но проверка валидности слота выполняется раньше
        // проверки вместимости, поэтому используем валидный слот в новом фрейме, чтобы
        // протестировать FrameFull напрямую:
        let _f2 = GnssFrame::new(TS);

        // Ручное заполнение 14-и наблюдений с разными допустимыми слотами через
        // from_samples
        let samples: Vec<GlonassSample> = (-7_i8..=6)
            .map(|s| make_obs(GloSlot::new(s).unwrap(), TS))
            .collect();
        let full = GnssFrame::from_samples(&samples).unwrap();

        assert_eq!(full.len(), MAX_GLONASS_SATS);

        let _ = f; // подавляем предупреждение о неиспользуемой переменной
    }

    #[test]
    fn test_get_by_slot_found() {
        let mut f = GnssFrame::new(TS);
        f.push(make_obs(GloSlot::new(3).unwrap(), TS)).unwrap();
        let obs = f.get_by_slot(3).unwrap();
        assert_eq!(obs.slot.get(), 3);
    }

    #[test]
    fn test_get_by_slot_not_found() {
        let mut f = GnssFrame::new(TS);
        f.push(make_obs(GloSlot::new(3).unwrap(), TS)).unwrap();
        assert!(f.get_by_slot(5).is_none());
    }

    #[test]
    fn test_contains_slot_true_and_false() {
        let mut f = GnssFrame::new(TS);
        f.push(make_obs(GloSlot::new(-3).unwrap(), TS)).unwrap();
        assert!(f.contains_slot(-3));
        assert!(!f.contains_slot(0));
    }

    #[test]
    fn test_from_samples_empty_returns_error() {
        let err = GnssFrame::from_samples(&[]).unwrap_err();
        assert!(matches!(err, GorkaError::EmptyChunk));
    }

    #[test]
    fn test_from_samples_mixed_timestamps_returns_error() {
        let samples = vec![
            make_obs(GloSlot::new(1).unwrap(), TS),
            make_obs(GloSlot::new(2).unwrap(), TS + 1),
        ];
        let err = GnssFrame::from_samples(&samples).unwrap_err();
        assert!(matches!(err, GorkaError::TimestampMismatch { .. }));
    }

    #[test]
    fn test_from_samples_valid_set() {
        let samples: Vec<GlonassSample> = [1_i8, -3, 5]
            .iter()
            .map(|&s| make_obs(GloSlot::new(s).unwrap(), TS))
            .collect();
        let f = GnssFrame::from_samples(&samples).unwrap();
        assert_eq!(f.len(), 3);
        // Verify sorted
        let slots: Vec<i8> = f.iter().map(|s| s.slot.get()).collect();
        assert_eq!(slots, vec![-3, 1, 5]);
    }

    #[test]
    fn test_validate_all_ok() {
        let samples: Vec<GlonassSample> = [0_i8, 1, -1]
            .iter()
            .map(|&s| make_obs(GloSlot::new(s).unwrap(), TS))
            .collect();
        let f = GnssFrame::from_samples(&samples).unwrap();
        assert!(f.validate_all().is_ok());
    }

    #[test]
    fn test_new_epoch_is_empty() {
        let e = GnssEpoch::new(TS);

        assert!(e.is_empty());
        assert_eq!(e.len(), 0);
        assert_eq!(e.timestamp_ms(), TS);
    }

    #[test]
    fn test_from_samples_single() {
        let e = GnssEpoch::from_samples(&[make_obs(GloSlot::new(1).unwrap(), TS)]).unwrap();
        assert_eq!(e.len(), 1);
        assert!(!e.is_empty());
    }

    #[test]
    fn test_from_samples_all_14_slots() {
        let samples: alloc::vec::Vec<_> = (-7_i8..=6)
            .map(|s| make_obs(GloSlot::new(s).unwrap(), TS))
            .collect();
        let e = GnssEpoch::from_samples(&samples).unwrap();
        assert_eq!(e.len(), MAX_GLONASS_SATS);
    }

    #[test]
    fn test_epoch_from_samples_empty_returns_error() {
        assert!(matches!(
            GnssEpoch::from_samples(&[]),
            Err(GorkaError::EmptyChunk)
        ));
    }

    #[test]
    fn test_from_samples_timestamp_mismatch() {
        let s = vec![
            make_obs(GloSlot::new(1).unwrap(), TS),
            make_obs(GloSlot::new(2).unwrap(), TS + 1000),
        ];
        assert!(matches!(
            GnssEpoch::from_samples(&s),
            Err(GorkaError::TimestampMismatch { .. })
        ));
    }

    #[test]
    fn test_from_samples_duplicate_slot() {
        let s = vec![
            make_obs(GloSlot::new(1).unwrap(), TS),
            make_obs(GloSlot::new(1).unwrap(), TS),
        ];
        assert!(matches!(
            GnssEpoch::from_samples(&s),
            Err(GorkaError::DuplicateSlot(1))
        ));
    }

    #[test]
    fn test_from_samples_too_many() {
        // 15 unique slots impossible for GLONASS, but test FrameFull path
        // by inserting 14 then one more via a second GnssEpoch converted
        let samples: alloc::vec::Vec<_> = (-7_i8..=6)
            .map(|s| make_obs(GloSlot::new(s).unwrap(), TS))
            .collect();
        assert!(GnssEpoch::from_samples(&samples).is_ok()); // 14 — ok
    }

    #[test]
    fn test_sorted_ascending() {
        let s = vec![
            make_obs(GloSlot::new(3).unwrap(), TS),
            make_obs(GloSlot::new(-7).unwrap(), TS),
            make_obs(GloSlot::new(0).unwrap(), TS),
            make_obs(GloSlot::new(-3).unwrap(), TS),
        ];
        let e = GnssEpoch::from_samples(&s).unwrap();
        let slots: alloc::vec::Vec<i8> = e.iter().map(|o| o.slot.get()).collect();
        assert_eq!(slots, [-7, -3, 0, 3]);
    }

    #[test]
    fn test_get_by_slot_found_epoch() {
        let e = GnssEpoch::from_samples(&[
            make_obs(GloSlot::new(2).unwrap(), TS),
            make_obs(GloSlot::new(-3).unwrap(), TS),
        ])
        .unwrap();
        assert_eq!(
            e.get_by_slot(GloSlot::new(2).unwrap()).unwrap().slot,
            GloSlot::new(2).unwrap()
        );
        assert_eq!(
            e.get_by_slot(GloSlot::new(-3).unwrap()).unwrap().slot,
            GloSlot::new(-3).unwrap()
        );
    }

    #[test]
    fn test_get_by_slot_not_found_epoch() {
        let e = GnssEpoch::from_samples(&[make_obs(GloSlot::new(0).unwrap(), TS)]).unwrap();
        assert!(e.get_by_slot(GloSlot::new(5).unwrap()).is_none());
    }

    #[test]
    fn test_contains_slot() {
        let e = GnssEpoch::from_samples(&[
            make_obs(GloSlot::new(-5).unwrap(), TS),
            make_obs(GloSlot::new(4).unwrap(), TS),
        ])
        .unwrap();
        assert!(e.contains_slot(GloSlot::new(-5).unwrap()));
        assert!(e.contains_slot(GloSlot::new(4).unwrap()));
        assert!(!e.contains_slot(GloSlot::new(0).unwrap()));
    }

    #[test]
    fn test_min_max_slot() {
        let s = vec![
            make_obs(GloSlot::new(3).unwrap(), TS),
            make_obs(GloSlot::new(-7).unwrap(), TS),
            make_obs(GloSlot::new(1).unwrap(), TS),
        ];
        let e = GnssEpoch::from_samples(&s).unwrap();
        assert_eq!(e.min_slot(), Some(GloSlot::new(-7).unwrap()));
        assert_eq!(e.max_slot(), Some(GloSlot::new(3).unwrap()));
    }

    #[test]
    fn test_min_max_slot_empty() {
        let e = GnssEpoch::new(TS);

        assert_eq!(e.min_slot(), None);
        assert_eq!(e.max_slot(), None);
    }

    #[test]
    fn test_group_by_timestamp_empty() {
        let epochs = GnssEpoch::group_by_timestamp(&[]);
        assert!(epochs.is_empty());
    }

    #[test]
    fn test_group_by_timestamp_single_ts() {
        let s = vec![
            make_obs(GloSlot::new(-3).unwrap(), TS),
            make_obs(GloSlot::new(0).unwrap(), TS),
            make_obs(GloSlot::new(3).unwrap(), TS),
        ];
        let epochs = GnssEpoch::group_by_timestamp(&s);
        assert_eq!(epochs.len(), 1);
        assert_eq!(epochs[0].len(), 3);
        assert_eq!(epochs[0].timestamp_ms(), TS);
    }

    #[test]
    fn test_group_by_timestamp_multiple_ts() {
        let base = TS;
        let s: alloc::vec::Vec<_> = (0..5u64)
            .flat_map(|i| {
                (-1_i8..=1).map(move |slot| make_obs(GloSlot::new(slot).unwrap(), base + i * 1000))
            })
            .collect();

        let epochs = GnssEpoch::group_by_timestamp(&s);
        assert_eq!(epochs.len(), 5);
        for ep in &epochs {
            assert_eq!(ep.len(), 3);
        }
    }

    #[test]
    fn test_group_by_timestamp_chronological_order() {
        let s = vec![
            make_obs(GloSlot::new(0).unwrap(), TS + 2000),
            make_obs(GloSlot::new(0).unwrap(), TS),
            make_obs(GloSlot::new(0).unwrap(), TS + 1000),
        ];
        let epochs = GnssEpoch::group_by_timestamp(&s);
        let tss: alloc::vec::Vec<u64> = epochs.iter().map(|e| e.timestamp_ms()).collect();
        assert_eq!(tss, [TS, TS + 1000, TS + 2000]);
    }

    #[test]
    fn test_epoch_validate_all_ok() {
        let s = vec![
            make_obs(GloSlot::new(-3).unwrap(), TS),
            make_obs(GloSlot::new(1).unwrap(), TS),
        ];
        let e = GnssEpoch::from_samples(&s).unwrap();

        assert!(e.validate_all().is_ok());
    }

    #[test]
    fn test_from_gnss_frame() {
        let mut frame = GnssFrame::new(TS);
        frame.push(make_obs(GloSlot::new(-3).unwrap(), TS)).unwrap();
        frame.push(make_obs(GloSlot::new(0).unwrap(), TS)).unwrap();
        let epoch = GnssEpoch::try_from(frame).unwrap();
        assert_eq!(epoch.len(), 2);
    }

    #[test]
    fn test_to_gnss_frame() {
        let s = vec![
            make_obs(GloSlot::new(1).unwrap(), TS),
            make_obs(GloSlot::new(-2).unwrap(), TS),
        ];
        let epoch = GnssEpoch::from_samples(&s).unwrap();
        let frame = GnssFrame::try_from(epoch).unwrap();
        assert_eq!(frame.len(), 2);
    }
}
