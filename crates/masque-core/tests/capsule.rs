use masque_core::{Capsule, Error, capsule};

#[test]
fn encodes_datagram_capsule() {
    let mut out = Vec::new();
    capsule::encode(
        Capsule {
            kind: capsule::DATAGRAM,
            payload: &[1, 2, 3],
        },
        &mut out,
    )
    .unwrap();
    assert_eq!(out, [0x00, 0x03, 1, 2, 3]);
}

#[test]
fn decodes_consecutive_capsules() {
    let buf = [0x00, 0x01, 0xaa, 0x40, 0x99, 0x00];
    let (first, used) = capsule::decode(&buf).unwrap();
    assert_eq!(
        first,
        Capsule {
            kind: capsule::DATAGRAM,
            payload: &[0xaa]
        }
    );
    let (second, rest) = capsule::decode(&buf[used..]).unwrap();
    assert_eq!(
        second,
        Capsule {
            kind: 0x99,
            payload: &[]
        }
    );
    assert_eq!(used + rest, buf.len());
}

#[test]
fn round_trips_large_payload() {
    let payload = vec![7; 70_000];
    let mut out = Vec::new();
    capsule::encode(
        Capsule {
            kind: 0x1234,
            payload: &payload,
        },
        &mut out,
    )
    .unwrap();
    assert_eq!(
        capsule::decode(&out),
        Ok((
            Capsule {
                kind: 0x1234,
                payload: &payload
            },
            out.len()
        ))
    );
}

#[test]
fn reports_incomplete_capsule() {
    for buf in [&[][..], &[0x00], &[0x00, 0x03, 1, 2]] {
        assert_eq!(capsule::decode(buf), Err(Error::UnexpectedEnd));
    }
}
