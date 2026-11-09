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

fn hex(text: &str) -> Vec<u8> {
    let digits: String = text.split_whitespace().collect();
    (0..digits.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&digits[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn repairs_offloaded_tcp_checksum() {
    let mut syn_ack = hex(
        "4500 003c 4159 0000 3e06 2f07 0101 0101 0a59 0002 01bb bb50 61c0 1e88 1838 967c
         a012 ff80 f750 0000 0204 ffd7 0402 080a fad6 c80c b4e7 d71b 0103 0307",
    );
    assert!(masque_core::ip_packet::fill_checksum(&mut syn_ack));
    assert_eq!(&syn_ack[36..38], &[0x06, 0xff]);
}

#[test]
fn computes_udp_checksum_and_skips_icmp() {
    let mut udp = ipv4([10, 0, 0, 1], [10, 0, 0, 2]);
    udp[2..4].copy_from_slice(&29u16.to_be_bytes());
    udp.extend_from_slice(&[0x13, 0x88, 0x00, 0x35, 0x00, 0x09, 0xaa, 0xbb, 0x61]);
    assert!(masque_core::ip_packet::fill_checksum(&mut udp));
    assert_ne!(&udp[26..28], &[0, 0]);
    let mut verify = udp.clone();
    assert!(masque_core::ip_packet::fill_checksum(&mut verify));
    assert_eq!(verify, udp);

    let mut icmp = ipv4([10, 0, 0, 1], [10, 0, 0, 2]);
    icmp[9] = 1;
    assert!(!masque_core::ip_packet::fill_checksum(&mut icmp));
}
