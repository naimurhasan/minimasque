use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

fn v4(packet: &[u8], at: usize) -> Option<IpAddr> {
    let octets: [u8; 4] = packet.get(at..at + 4)?.try_into().ok()?;
    Some(IpAddr::V4(Ipv4Addr::from(octets)))
}

fn v6(packet: &[u8], at: usize) -> Option<IpAddr> {
    let octets: [u8; 16] = packet.get(at..at + 16)?.try_into().ok()?;
    Some(IpAddr::V6(Ipv6Addr::from(octets)))
}

pub fn version(packet: &[u8]) -> Option<u8> {
    packet.first().map(|byte| byte >> 4)
}

pub fn source(packet: &[u8]) -> Option<IpAddr> {
    match version(packet)? {
        4 => v4(packet, 12),
        6 => v6(packet, 8),
        _ => None,
    }
}

pub fn destination(packet: &[u8]) -> Option<IpAddr> {
    match version(packet)? {
        4 => v4(packet, 16),
        6 => v6(packet, 24),
        _ => None,
    }
}
