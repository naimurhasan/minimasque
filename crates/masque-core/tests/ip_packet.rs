use std::net::IpAddr;

use masque_core::ip_packet::{destination, source, version};

fn ipv4(src: [u8; 4], dst: [u8; 4]) -> Vec<u8> {
    let mut packet = vec![0x45, 0, 0, 20, 0, 0, 0, 0, 64, 17, 0, 0];
    packet.extend_from_slice(&src);
    packet.extend_from_slice(&dst);
    packet
}

fn ipv6(src: IpAddr, dst: IpAddr) -> Vec<u8> {
    let octets = |addr: IpAddr| match addr {
        IpAddr::V6(v6) => v6.octets(),
        IpAddr::V4(_) => unreachable!(),
    };
    let mut packet = vec![0x60, 0, 0, 0, 0, 0, 17, 64];
    packet.extend_from_slice(&octets(src));
    packet.extend_from_slice(&octets(dst));
    packet
}

#[test]
fn reads_ipv4_addresses() {
    let packet = ipv4([10, 89, 0, 2], [1, 1, 1, 1]);
    assert_eq!(version(&packet), Some(4));
    assert_eq!(source(&packet), Some("10.89.0.2".parse().unwrap()));
    assert_eq!(destination(&packet), Some("1.1.1.1".parse().unwrap()));
}

#[test]
fn reads_ipv6_addresses() {
    let (src, dst) = (
        "fd00::2".parse().unwrap(),
        "2606:4700::1111".parse().unwrap(),
    );
    let packet = ipv6(src, dst);
    assert_eq!(
        (source(&packet), destination(&packet)),
        (Some(src), Some(dst))
    );
}

#[test]
fn rejects_short_or_unknown_packets() {
    assert_eq!(source(&[]), None);
    assert_eq!(destination(&ipv4([1, 2, 3, 4], [5, 6, 7, 8])[..19]), None);
    assert_eq!(source(&[0x50; 40]), None);
}
