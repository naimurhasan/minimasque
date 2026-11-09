use std::net::IpAddr;

use masque_core::Error;
use masque_core::ip_capsule::{
    Address, Prefix, Range, decode_addresses, decode_routes, encode_addresses, encode_routes,
};

fn ip(text: &str) -> IpAddr {
    text.parse().unwrap()
}

fn address(request_id: u64, addr: &str, len: u8) -> Address {
    Address {
        request_id,
        prefix: Prefix {
            addr: ip(addr),
            len,
        },
    }
}

fn range(start: &str, end: &str, protocol: u8) -> Range {
    Range {
        start: ip(start),
        end: ip(end),
        protocol,
    }
}

#[test]
fn encodes_ipv4_assignment_bytes() {
    let mut out = Vec::new();
    encode_addresses(&[address(0, "192.0.2.10", 32)], &mut out).unwrap();
    assert_eq!(out, [0, 4, 192, 0, 2, 10, 32]);
}

#[test]
fn round_trips_mixed_addresses() {
    let addresses = [address(1, "10.0.0.2", 32), address(2, "2001:db8::", 64)];
    let mut out = Vec::new();
    encode_addresses(&addresses, &mut out).unwrap();
    assert_eq!(out.len(), 7 + 19);
    assert_eq!(decode_addresses(&out), Ok(addresses.to_vec()));
}

#[test]
fn empty_payload_means_no_addresses() {
    assert_eq!(decode_addresses(&[]), Ok(vec![]));
    assert_eq!(decode_routes(&[]), Ok(vec![]));
}

#[test]
fn rejects_bad_addresses() {
    assert_eq!(
        encode_addresses(&[address(0, "10.0.0.1", 33)], &mut Vec::new()),
        Err(Error::InvalidCapsule)
    );
    assert_eq!(
        decode_addresses(&[0, 5, 1, 2, 3, 4, 32]),
        Err(Error::InvalidCapsule)
    );
    assert_eq!(
        decode_addresses(&[0, 4, 10, 0, 0]),
        Err(Error::UnexpectedEnd)
    );
    assert_eq!(
        decode_addresses(&[0, 4, 10, 0, 0, 1, 40]),
        Err(Error::InvalidCapsule)
    );
}

#[test]
fn encodes_full_ipv4_route_bytes() {
    let mut out = Vec::new();
    encode_routes(&[range("0.0.0.0", "255.255.255.255", 0)], &mut out).unwrap();
    assert_eq!(out, [4, 0, 0, 0, 0, 255, 255, 255, 255, 0]);
}

#[test]
fn round_trips_ordered_routes() {
    let routes = [
        range("10.0.0.0", "10.255.255.255", 6),
        range("10.0.0.0", "10.0.0.255", 17),
        range("192.168.0.0", "192.168.255.255", 17),
        range("::", "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff", 0),
    ];
    let mut out = Vec::new();
    encode_routes(&routes, &mut out).unwrap();
    assert_eq!(decode_routes(&out), Ok(routes.to_vec()));
}

#[test]
fn rejects_unordered_or_invalid_routes() {
    for routes in [
        vec![range("10.0.0.9", "10.0.0.1", 0)],
        vec![range("10.0.0.0", "2001:db8::", 0)],
        vec![
            range("10.0.0.0", "10.0.0.255", 0),
            range("10.0.0.255", "10.0.1.0", 0),
        ],
        vec![
            range("10.0.0.0", "10.0.0.1", 17),
            range("10.0.0.5", "10.0.0.6", 6),
        ],
        vec![range("::", "::1", 0), range("10.0.0.0", "10.0.0.1", 0)],
    ] {
        assert_eq!(
            encode_routes(&routes, &mut Vec::new()),
            Err(Error::InvalidCapsule),
            "{routes:?}"
        );
    }
    let overlapping = [
        4, 10, 0, 0, 0, 10, 0, 0, 9, 0, 4, 10, 0, 0, 5, 10, 0, 0, 20, 0,
    ];
    assert_eq!(decode_routes(&overlapping), Err(Error::InvalidCapsule));
}
