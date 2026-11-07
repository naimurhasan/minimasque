use bytes::Bytes;
use masque_h2::capsules::{Decoded, Reader, encode_udp};

#[test]
fn encodes_udp_as_datagram_capsule() {
    assert_eq!(
        encode_udp(b"hi").unwrap(),
        Bytes::from_static(&[0x00, 0x03, 0x00, b'h', b'i'])
    );
}

#[test]
fn reassembles_capsules_split_across_chunks() {
    let mut wire = encode_udp(b"first").unwrap().to_vec();
    wire.extend_from_slice(&encode_udp(b"second").unwrap());
    let mut reader = Reader::default();
    let (a, b) = wire.split_at(9);
    let first = reader.push(a);
    let second = reader.push(b);
    assert_eq!(
        first,
        Decoded {
            udp: vec![Bytes::from_static(b"first")],
            skipped: 0
        }
    );
    assert_eq!(
        second,
        Decoded {
            udp: vec![Bytes::from_static(b"second")],
            skipped: 0
        }
    );
}

#[test]
fn skips_unknown_capsules_and_contexts() {
    let mut reader = Reader::default();
    let decoded = reader.push(&[0x40, 0x99, 0x01, 0xff, 0x00, 0x02, 0x02, 0xaa]);
    assert_eq!(
        decoded,
        Decoded {
            udp: vec![],
            skipped: 2
        }
    );
}
