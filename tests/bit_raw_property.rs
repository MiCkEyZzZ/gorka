use gorka::{BitReader, BitWrite, RawBitWriter};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_roundtrip_bits(values in proptest::collection::vec((0u64.., 1u8..=64), 1..100)) {
        let mut buf = [0u8; 1024];
        let mut w = RawBitWriter::new(&mut buf);
        let mut filtered = Vec::new();

        for (value, bits) in values {
            if bits < 64 && value >= (1u64 << bits) {
                continue;
            }

            w.write_bits(value, bits).unwrap();

            filtered.push((value, bits));
        }

        let n = w.bytes_written();
        let mut r = BitReader::new(&buf[..n]);

        for (expected, bits) in filtered {
            let actual = r.read_bits(bits).unwrap();

            prop_assert_eq!(actual, expected);
        }
    }
}

proptest! {
    #[test]
    fn prop_signed_roundtrip(values in proptest::collection::vec(
        (1u8..=32, -1_000_000i64..1_000_000), 1..100))
    {
        let mut buf = [0u8; 1024];
        let mut w = RawBitWriter::new(&mut buf);
        let filtered: Vec<(i64, u8)> = values.into_iter()
            .filter(|(bits, value)| {
                let max = 1i64 << (*bits - 1);

                *value < max && *value >= -max
            })
            .map(|(bits, value)| (value, bits))
            .collect();

        for (value, bits) in &filtered {
            w.write_bits_signed(*value, *bits).unwrap();
        }

        let n = w.bytes_written();
        let mut r = BitReader::new(&buf[..n]);

        for (expected, bits) in filtered {
            let actual = r.read_bits_signed(bits).unwrap();

            prop_assert_eq!(actual, expected);
        }
    }
}

proptest! {
    #[test]
    fn prop_bits_stream(bits in proptest::collection::vec(any::<bool>(), 1..1000)) {
        let mut buf = [0u8; 1024];
        let mut w = RawBitWriter::new(&mut buf);

        for b in &bits {
            w.write_bit(*b).unwrap();
        }

        let n = w.bytes_written();
        let mut r = BitReader::new(&buf[..n]);

        for expected in bits {
            let actual = r.read_bit().unwrap();

            prop_assert_eq!(actual, expected);
        }
    }
}

proptest! {
    #[test]
    fn prop_align_behavior(bits in 1u8..=32, value in 0u64..(1u64 << 32)) {
        let bits = bits.min(64);
        let value = value % (1u64 << bits);
        let mut buf = [0u8; 1024];
        let mut w = RawBitWriter::new(&mut buf);

        w.write_bits(value, bits).unwrap();
        w.align_to_byte();
        w.write_bits(0b10101010, 8).unwrap();

        let n = w.bytes_written();
        let mut r = BitReader::new(&buf[..n]);

        let read = r.read_bits(bits).unwrap();
        prop_assert_eq!(read, value);

        r.align_to_byte();

        let next = r.read_bits(8).unwrap();
        prop_assert_eq!(next, 0b10101010);
    }
}
