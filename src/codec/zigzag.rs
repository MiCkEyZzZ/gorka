#[inline]
pub fn encode_i64(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

#[inline]
pub fn decode_i64(zz: u64) -> i64 {
    let n = (zz >> 1) as i64;
    if (zz & 1) == 0 {
        n
    } else {
        !n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_roundtrip() {
        let values = [0, -1, 1, -2, 2, i64::MAX, i64::MIN, 123456, -654321];

        for &v in &values {
            let encoded = encode_i64(v);
            let decoded = decode_i64(encoded);

            assert_eq!(decoded, v, "failed for value {v}");
        }
    }

    #[test]
    fn test_zigzag_encoding_values() {
        assert_eq!(encode_i64(0), 0);
        assert_eq!(encode_i64(-1), 1);
        assert_eq!(encode_i64(1), 2);
        assert_eq!(encode_i64(-2), 3);
        assert_eq!(encode_i64(2), 4);
    }
}
