use masque_core::{Error, varint};

const RFC9000_VECTORS: [(&[u8], u64); 5] = [
    (
        &[0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c],
        151_288_809_941_952_652,
    ),
    (&[0x9d, 0x7f, 0x3e, 0x7d], 494_878_333),
    (&[0x7b, 0xbd], 15_293),
    (&[0x25], 37),
    (&[0x40, 0x25], 37),
];

#[test]
fn decodes_rfc9000_vectors() {
    for (bytes, value) in RFC9000_VECTORS {
        assert_eq!(varint::decode(bytes), Ok((value, bytes.len())));
    }
}

#[test]
fn encodes_minimal_length() {
    for (bytes, value) in RFC9000_VECTORS
        .into_iter()
        .filter(|(b, _)| b != &[0x40, 0x25])
    {
        let mut out = Vec::new();
        varint::encode(value, &mut out).unwrap();
        assert_eq!(out, bytes);
    }
}

#[test]
fn round_trips_boundaries() {
    for value in [
        0,
        63,
        64,
        16_383,
        16_384,
        1_073_741_823,
        1_073_741_824,
        varint::MAX,
    ] {
        let mut out = Vec::new();
        varint::encode(value, &mut out).unwrap();
        assert_eq!(varint::decode(&out), Ok((value, out.len())));
    }
}

#[test]
fn rejects_overflow() {
    assert_eq!(
        varint::encode(varint::MAX + 1, &mut Vec::new()),
        Err(Error::VarIntOverflow)
    );
}

#[test]
fn rejects_truncated() {
    assert_eq!(varint::decode(&[]), Err(Error::UnexpectedEnd));
    assert_eq!(varint::decode(&[0x9d, 0x7f]), Err(Error::UnexpectedEnd));
}
