use masque_core::{Capsule, Datagram, Error, capsule, datagram};

#[test]
fn encodes_udp_payload() {
    let mut out = Vec::new();
    datagram::encode(
        Datagram {
            context_id: datagram::UDP_PAYLOAD,
            payload: b"dns",
        },
        &mut out,
    )
    .unwrap();
    assert_eq!(out, b"\x00dns");
}

#[test]
fn decodes_extension_context() {
    assert_eq!(
        datagram::decode(&[0x40, 0x02, 0xff]),
        Ok(Datagram {
            context_id: 2,
            payload: &[0xff]
        })
    );
}

#[test]
fn allows_empty_payload() {
    assert_eq!(
        datagram::decode(&[0x00]),
        Ok(Datagram {
            context_id: datagram::UDP_PAYLOAD,
            payload: &[]
        })
    );
}

#[test]
fn rejects_missing_context_id() {
    assert_eq!(datagram::decode(&[]), Err(Error::UnexpectedEnd));
}

#[test]
fn nests_inside_datagram_capsule() {
    let mut inner = Vec::new();
    datagram::encode(
        Datagram {
            context_id: datagram::UDP_PAYLOAD,
            payload: b"hi",
        },
        &mut inner,
    )
    .unwrap();
    let mut wire = Vec::new();
    capsule::encode(
        Capsule {
            kind: capsule::DATAGRAM,
            payload: &inner,
        },
        &mut wire,
    )
    .unwrap();
    let (outer, _) = capsule::decode(&wire).unwrap();
    assert_eq!(
        datagram::decode(outer.payload),
        Ok(Datagram {
            context_id: datagram::UDP_PAYLOAD,
            payload: b"hi"
        })
    );
}
