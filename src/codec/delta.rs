#[inline]
pub fn delta_i64(
    current: i64,
    previous: i64,
) -> i64 {
    current - previous
}

#[inline]
pub fn delta_of_delta_i64(
    current: i64,
    previous: i64,
    prev_delta: i64,
) -> i64 {
    let delta = current - previous;

    delta - prev_delta
}

#[inline]
pub fn delta_u64(
    current: u64,
    previous: u64,
) -> u64 {
    current.wrapping_sub(previous)
}

#[inline]
pub fn delta_of_delta_u64(
    current: u64,
    previous: u64,
    prev_delta: u64,
) -> i64 {
    let delta = current.wrapping_sub(previous);

    delta as i64 - prev_delta as i64
}

#[inline]
pub fn reconstruct_from_delta(
    previous: i64,
    delta: i64,
) -> i64 {
    previous + delta
}

#[inline]
pub fn reconstruct_from_dod(
    previous: i64,
    prev_delta: i64,
    dod: i64,
) -> (i64, i64) {
    let delta = prev_delta + dod;
    let current = previous + delta;

    (current, delta)
}

#[inline]
pub fn reconstruct_from_dod_u64(
    previous: u64,
    prev_delta: u64,
    dod: i64,
) -> (u64, u64) {
    let delta = (prev_delta as i64 + dod) as u64;
    let current = previous.wrapping_add(delta);

    (current, delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_i64_basic() {
        assert_eq!(delta_i64(10, 5), 5);
        assert_eq!(delta_i64(5, 10), -5);
        assert_eq!(delta_i64(0, 0), 0);
    }

    #[test]
    fn test_delta_of_delta_i64_basic() {
        assert_eq!(delta_of_delta_i64(10, 5, 4), 1); // delta = 5, dod = 5 - 4 = 1
        assert_eq!(delta_of_delta_i64(5, 10, -3), -2); // delta = -5, dod = -5 -
                                                       // (-3) = -2
    }

    #[test]
    fn test_delta_u64_basic() {
        assert_eq!(delta_u64(10, 5), 5);
        assert_eq!(delta_u64(0, 10), u64::MAX - 9); // wrapping_sub
        assert_eq!(delta_u64(100, 100), 0);
    }

    #[test]
    fn test_delta_of_delta_u64_basic() {
        assert_eq!(delta_of_delta_u64(10, 5, 4), 1);
        assert_eq!(delta_of_delta_u64(5, 10, 0), (u64::MAX - 4) as i64); // wrapping sub and i64 conversion
    }

    #[test]
    fn test_reconstruct_from_delta() {
        assert_eq!(reconstruct_from_delta(5, 10), 15);
        assert_eq!(reconstruct_from_delta(-5, 3), -2);
        assert_eq!(reconstruct_from_delta(0, 0), 0);
    }

    #[test]
    fn test_reconstruct_from_dod() {
        let (current, delta) = reconstruct_from_dod(5, 3, 2);
        assert_eq!(delta, 5); // prev_delta + dod = 3 + 2
        assert_eq!(current, 10); // previous + delta = 5 + 5

        let (c2, d2) = reconstruct_from_dod(-2, -1, -3);
        assert_eq!(d2, -4);
        assert_eq!(c2, -6);
    }

    #[test]
    fn test_reconstruct_from_dod_u64() {
        let (current, delta) = reconstruct_from_dod_u64(10, 3, 5);
        assert_eq!(delta, 8);
        assert_eq!(current, 18);

        // тест wrapping
        let max = u64::MAX;
        let (c2, d2) = reconstruct_from_dod_u64(max - 2, 3, 5);
        assert_eq!(d2, 8);
        assert_eq!(c2, 5); // wrapping_add
    }

    #[test]
    fn test_roundtrip_i64() {
        let previous = -100i64;
        let prev_delta = 50i64;
        let current = 75i64;
        let dod = delta_of_delta_i64(current, previous, prev_delta);
        let (reconstructed, new_delta) = reconstruct_from_dod(previous, prev_delta, dod);

        assert_eq!(reconstructed, current);
        assert_eq!(new_delta, current - previous);
    }

    #[test]
    fn test_roundtrip_u64() {
        let previous: u64 = 100;
        let prev_delta: u64 = 25;
        let current: u64 = 150;
        let dod = delta_of_delta_u64(current, previous, prev_delta);
        let (reconstructed, new_delta) = reconstruct_from_dod_u64(previous, prev_delta, dod);

        assert_eq!(reconstructed, current);
        assert_eq!(new_delta, current - previous);
    }
}
