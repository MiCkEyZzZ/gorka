use gorka::{GlonassSample, GnssFrame, GorkaError, MilliHz, Millimeter, MAX_GLONASS_SATS};

const TS: u64 = 1_700_000_000_000;

fn sample(slot: i8) -> GlonassSample {
    GlonassSample {
        timestamp_ms: 1_700_000_000_000,
        slot,
        cn0_dbhz: 42,
        pseudorange_mm: Millimeter(21_500_000_000),
        doppler_millihz: MilliHz(1_200_500),
        carrier_phase_cycles: None,
    }
}

#[test]
fn pseudorange_is_i64() {
    // Compile-time: if this builds, the field is i64
    let s = sample(0);
    let _: i64 = s.pseudorange_mm.0;
}

#[test]
fn doppler_is_i32() {
    let s = sample(0);
    let _: MilliHz = s.doppler_millihz;
}

#[test]
fn carrier_phase_is_option_i64() {
    let s = sample(0);
    let _: Option<i64> = s.carrier_phase_cycles;
}

#[test]
fn pseudorange_1mm_precision() {
    // 21 500 000 000 mm vs 21 500 000 001 mm — 1 mm difference preserved
    let a = GlonassSample {
        pseudorange_mm: Millimeter(21_500_000_000),
        ..sample(0)
    };
    let b = GlonassSample {
        pseudorange_mm: Millimeter(21_500_000_001),
        ..sample(0)
    };
    assert_eq!(b.pseudorange_mm.0 - a.pseudorange_mm.0, 1);
}

#[test]
fn doppler_1mhz_precision() {
    // 1200.500 Hz and 1200.501 Hz differ by exactly 1 mHz
    let a = GlonassSample {
        doppler_millihz: MilliHz(1_200_500),
        ..sample(0)
    };
    let b = GlonassSample {
        doppler_millihz: MilliHz(1_200_501),
        ..sample(0)
    };
    assert_eq!(b.doppler_millihz.0 - a.doppler_millihz.0, 1);
}

#[test]
fn negative_doppler_stored_correctly() {
    let s = GlonassSample {
        doppler_millihz: MilliHz(-3_750_250),
        ..sample(0)
    };
    assert_eq!(s.doppler_millihz, MilliHz(-3_750_250));
}

#[test]
fn carrier_phase_none_by_default_in_helper() {
    assert_eq!(sample(0).carrier_phase_cycles, None);
}

#[test]
fn carrier_phase_large_positive() {
    let s = GlonassSample {
        carrier_phase_cycles: Some(i64::MAX),
        ..sample(0)
    };
    assert_eq!(s.carrier_phase_cycles, Some(i64::MAX));
}

#[test]
fn carrier_phase_large_negative() {
    let s = GlonassSample {
        carrier_phase_cycles: Some(i64::MIN),
        ..sample(0)
    };
    assert_eq!(s.carrier_phase_cycles, Some(i64::MIN));
}

#[test]
fn all_valid_slots_pass() {
    for k in -7_i8..=6 {
        assert!(sample(k).validate_slot().is_ok(), "slot {k} should pass");
    }
}

#[test]
fn slot_minus_eight_fails() {
    assert!(matches!(
        sample(-8).validate_slot(),
        Err(GorkaError::InvalidSlot(-8))
    ));
}

#[test]
fn slot_plus_seven_fails() {
    assert!(matches!(
        sample(7).validate_slot(),
        Err(GorkaError::InvalidSlot(7))
    ));
}

#[test]
fn pseudorange_boundary_min_passes() {
    let s = GlonassSample {
        pseudorange_mm: GlonassSample::PSEUDORANGE_MIN_MM,
        ..sample(0)
    };
    assert!(s.validate_pseudorange().is_ok());
}

#[test]
fn pseudorange_boundary_max_passes() {
    let s = GlonassSample {
        pseudorange_mm: GlonassSample::PSEUDORANGE_MAX_MM,
        ..sample(0)
    };
    assert!(s.validate_pseudorange().is_ok());
}

#[test]
fn pseudorange_zero_fails() {
    let s = GlonassSample {
        pseudorange_mm: Millimeter(0),
        ..sample(0)
    };
    assert!(matches!(
        s.validate_pseudorange(),
        Err(GorkaError::InvalidPseudorange(0))
    ));
}

#[test]
fn pseudorange_negative_fails() {
    let s = GlonassSample {
        pseudorange_mm: Millimeter(-1_000),
        ..sample(0)
    };
    assert!(matches!(
        s.validate_pseudorange(),
        Err(GorkaError::InvalidPseudorange(-1_000))
    ));
}

#[test]
fn doppler_boundary_positive_passes() {
    let s = GlonassSample {
        doppler_millihz: GlonassSample::DOPPLER_MAX_MILLIHZ,
        ..sample(0)
    };
    assert!(s.validate_doppler().is_ok());
}

#[test]
fn doppler_boundary_negative_passes() {
    let s = GlonassSample {
        doppler_millihz: MilliHz(-GlonassSample::DOPPLER_MAX_MILLIHZ.0),
        ..sample(0)
    };
    assert!(s.validate_doppler().is_ok());
}

#[test]
fn doppler_overflow_positive_fails() {
    let s = GlonassSample {
        doppler_millihz: MilliHz(5_000_001),
        ..sample(0)
    };
    assert!(matches!(
        s.validate_doppler(),
        Err(GorkaError::InvalidDoppler(5_000_001))
    ));
}

#[test]
fn doppler_overflow_negative_fails() {
    let s = GlonassSample {
        doppler_millihz: MilliHz(-5_000_001),
        ..sample(0)
    };
    assert!(matches!(
        s.validate_doppler(),
        Err(GorkaError::InvalidDoppler(-5_000_001))
    ));
}

#[test]
fn carrier_freq_k0_is_1602mhz() {
    // k=0 → 1 602 000 000 mHz = 1602.000 MHz
    let s = GlonassSample {
        slot: 0,
        ..sample(0)
    };
    assert_eq!(s.carrier_freq_millihz().unwrap(), 1_602_000_000);
}

#[test]
fn carrier_freq_k1_is_16025625mhz() {
    // k=+1 → 1602 + 0.5625 = 1602.5625 MHz = 1_602_562_500 mHz
    let s = GlonassSample {
        slot: 1,
        ..sample(1)
    };
    assert_eq!(s.carrier_freq_millihz().unwrap(), 1_602_562_500);
}

#[test]
fn carrier_freq_k_minus7() {
    // k=-7 → 1602 - 7×0.5625 = 1598.0625 MHz = 1_598_062_500 mHz
    let s = GlonassSample {
        slot: -7,
        ..sample(-7)
    };
    assert_eq!(s.carrier_freq_millihz().unwrap(), 1_598_062_500);
}

#[test]
fn carrier_freq_k6() {
    // k=+6 → 1602 + 6×0.5625 = 1605.375 MHz = 1_605_375_000 mHz
    let s = GlonassSample {
        slot: 6,
        ..sample(6)
    };
    assert_eq!(s.carrier_freq_millihz().unwrap(), 1_605_375_000);
}

#[test]
fn carrier_freq_invalid_slot_errors() {
    let s = GlonassSample {
        slot: 99,
        ..sample(0)
    };
    assert!(s.carrier_freq_millihz().is_err());
}

#[test]
fn frame_push_and_len() {
    let mut f = GnssFrame::new(TS);
    assert_eq!(f.len(), 0);
    f.push(sample(0)).unwrap();
    assert_eq!(f.len(), 1);
    f.push(sample(1)).unwrap();
    assert_eq!(f.len(), 2);
}

#[test]
fn frame_push_all_14_slots() {
    let mut f = GnssFrame::new(TS);
    for k in -7_i8..=6 {
        f.push(sample(k)).unwrap();
    }
    assert_eq!(f.len(), MAX_GLONASS_SATS);
}

#[test]
fn frame_iter_is_sorted_by_slot() {
    let mut f = GnssFrame::new(TS);
    for &k in &[6_i8, -7, 3, 0, -1] {
        f.push(sample(k)).unwrap();
    }
    let slots: Vec<i8> = f.iter().map(|s| s.slot).collect();
    assert!(slots.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn frame_get_by_slot_hit_and_miss() {
    let mut f = GnssFrame::new(TS);
    f.push(sample(-3)).unwrap();
    assert!(f.get_by_slot(-3).is_some());
    assert!(f.get_by_slot(5).is_none());
}

#[test]
fn frame_duplicate_slot_error() {
    let mut f = GnssFrame::new(TS);
    f.push(sample(2)).unwrap();
    let err = f.push(sample(2)).unwrap_err();
    assert!(matches!(err, GorkaError::DuplicateSlot(2)));
}

#[test]
fn frame_timestamp_mismatch_error() {
    let mut f = GnssFrame::new(TS);
    let wrong = GlonassSample {
        timestamp_ms: TS + 10,
        ..sample(1)
    };
    let err = f.push(wrong).unwrap_err();
    assert!(matches!(
        err,
        GorkaError::TimestampMismatch {
            frame: _,
            sample: _
        }
    ));
}

#[test]
fn frame_invalid_slot_error() {
    let mut f = GnssFrame::new(TS);
    let bad = GlonassSample {
        slot: -9,
        ..sample(0)
    };
    let err = f.push(bad).unwrap_err();
    assert!(matches!(err, GorkaError::InvalidSlot(-9)));
}

#[test]
fn frame_from_samples_empty_errors() {
    let err = GnssFrame::from_samples(&[]).unwrap_err();
    assert!(matches!(err, GorkaError::EmptyChunk));
}

#[test]
fn frame_from_samples_valid() {
    let samples: Vec<GlonassSample> = [-7_i8, 0, 6].iter().map(|&k| sample(k)).collect();
    let f = GnssFrame::from_samples(&samples).unwrap();
    assert_eq!(f.len(), 3);
    // Slots sorted
    let slots: Vec<i8> = f.iter().map(|s| s.slot).collect();
    assert_eq!(slots, vec![-7, 0, 6]);
}

#[test]
fn error_display_invalid_pseudorange() {
    let e = GorkaError::InvalidPseudorange(-42);
    let s = e.to_string();
    assert!(s.contains("-42"), "got: {s}");
}

#[test]
fn error_display_invalid_doppler() {
    let e = GorkaError::InvalidDoppler(9_999_999);
    let s = e.to_string();
    assert!(s.contains("9999999"), "got: {s}");
}

#[test]
fn error_display_timestamp_mismatch() {
    let e = GorkaError::TimestampMismatch {
        frame: 100,
        sample: 200,
    };
    let s = e.to_string();
    assert!(s.contains("200") && s.contains("100"), "got: {s}");
}

#[test]
fn error_display_frame_full() {
    let s = GorkaError::FrameFull.to_string();
    assert!(s.contains("14"), "got: {s}");
}
