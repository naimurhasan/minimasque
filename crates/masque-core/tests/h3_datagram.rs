use masque_core::{Error, h3_datagram};

#[test]
fn encodes_quarter_stream_id() {
    let mut out = Vec::new();
    h3_datagram::encode(8, b"\x00hi", &mut out).unwrap();
    assert_eq!(out, b"\x02\x00hi");
}

#[test]
fn round_trips_client_bidi_stream_ids() {
    for stream_id in [0, 4, 400, 1 << 40] {
        let mut out = Vec::new();
        h3_datagram::encode(stream_id, b"x", &mut out).unwrap();
        assert_eq!(h3_datagram::decode(&out), Ok((stream_id, &b"x"[..])));
    }
}

#[test]
fn rejects_empty_datagram() {
    assert_eq!(h3_datagram::decode(&[]), Err(Error::UnexpectedEnd));
}
