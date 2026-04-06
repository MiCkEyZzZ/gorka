use gorka::{BitReader, BitWrite, RawBitWriter};

#[test]
fn test_roundtrip_simple_raw() {
    let mut buf = [0u8; 32];
    let mut w = RawBitWriter::new(&mut buf);

    w.write_bits(0b101, 3).unwrap();
    w.write_bits(0b11110000, 8).unwrap();
    w.write_bits(0b11, 2).unwrap();

    let n = w.bytes_written();

    let mut r = BitReader::new(&buf[..n]);

    let a = r.read_bits(3).unwrap();
    let b = r.read_bits(8).unwrap();
    let c = r.read_bits(2).unwrap();

    assert_eq!(a, 0b101);
    assert_eq!(b, 0b11110000);
    assert_eq!(c, 0b11);
}

#[test]
fn test_roundtrip_bit_by_bit_raw() {
    let mut buf = [0u8; 32];
    let mut w = RawBitWriter::new(&mut buf);

    for i in 0..100 {
        w.write_bit(i % 2 == 0).unwrap();
    }

    let n = w.bytes_written();
    let mut r = BitReader::new(&buf[..n]);

    for i in 0..100 {
        let bit = r.read_bit().unwrap();

        assert_eq!(bit, i % 2 == 0);
    }
}

#[test]
fn test_roundtrip_cross_byte_boundaries_raw() {
    let mut buf = [0u8; 32];
    let mut w = RawBitWriter::new(&mut buf);

    w.write_bits(0b1, 1).unwrap();
    w.write_bits(0b10, 2).unwrap();
    w.write_bits(0b10101010, 8).unwrap();
    w.write_bits(0b111, 3).unwrap();

    let n = w.bytes_written();
    let mut r = BitReader::new(&buf[..n]);

    assert_eq!(r.read_bits(1).unwrap(), 0b1);
    assert_eq!(r.read_bits(2).unwrap(), 0b10);
    assert_eq!(r.read_bits(8).unwrap(), 0b10101010);
    assert_eq!(r.read_bits(3).unwrap(), 0b111);
}

#[test]
fn test_signed_roundtrip() {
    let values = [0, -1, 1, -2, 2, -100, 100];
    let mut buf = [0u8; 64];
    let mut w = RawBitWriter::new(&mut buf);

    for &v in &values {
        w.write_bits_signed(v, 16).unwrap();
    }

    let n = w.bytes_written();
    let mut r = BitReader::new(&buf[..n]);

    for &expected in &values {
        let actual = r.read_bits_signed(16).unwrap();

        assert_eq!(actual, expected);
    }
}

#[test]
fn test_align_roundtrip() {
    let mut buf = [0u8; 32];
    let mut w = RawBitWriter::new(&mut buf);

    w.write_bits(0b101, 3).unwrap();
    w.align_to_byte();
    w.write_bits(0b11110000, 8).unwrap();

    let n = w.bytes_written();
    let mut r = BitReader::new(&buf[..n]);

    assert_eq!(r.read_bits(3).unwrap(), 0b101);

    r.align_to_byte();

    assert_eq!(r.read_bits(8).unwrap(), 0b11110000);
}

#[test]
fn test_eof_after_exact_read() {
    let mut buf = [0u8; 32];
    let mut w = RawBitWriter::new(&mut buf);

    w.write_bits(0b10101010, 8).unwrap();

    let n = w.bytes_written();
    let mut r = BitReader::new(&buf[..n]);

    r.read_bits(8).unwrap();

    let res = r.read_bit();

    assert!(res.is_err());
}

#[test]
fn test_partial_byte_roundtrip() {
    let mut buf = [0u8; 32];
    let mut w = RawBitWriter::new(&mut buf);

    w.write_bits(0b10101, 5).unwrap();

    let n = w.bytes_written();
    let mut r = BitReader::new(&buf[..n]);
    let v = r.read_bits(5).unwrap();

    assert_eq!(v, 0b10101);
}
