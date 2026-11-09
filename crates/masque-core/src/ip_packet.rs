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

const TCP: u8 = 6;
const UDP: u8 = 17;

fn sum(bytes: &[u8]) -> u32 {
    bytes
        .chunks(2)
        .map(|pair| u32::from(u16::from_be_bytes([pair[0], *pair.get(1).unwrap_or(&0)])))
        .sum()
}

fn fold(mut total: u32) -> u16 {
    while total > 0xffff {
        total = (total & 0xffff) + (total >> 16);
    }
    !(total as u16)
}

fn transport(packet: &[u8]) -> Option<(usize, u8, u32)> {
    match version(packet)? {
        4 => {
            let header = usize::from(packet.first()? & 0x0f) * 4;
            let total = usize::from(u16::from_be_bytes([*packet.get(2)?, *packet.get(3)?]));
            let proto = *packet.get(9)?;
            let len = total.checked_sub(header)?;
            let pseudo = sum(packet.get(12..20)?) + u32::from(proto) + len as u32;
            Some((header, proto, pseudo))
        }
        6 => {
            let len = usize::from(u16::from_be_bytes([*packet.get(4)?, *packet.get(5)?]));
            let proto = *packet.get(6)?;
            let pseudo = sum(packet.get(8..40)?) + u32::from(proto) + len as u32;
            Some((40, proto, pseudo))
        }
        _ => None,
    }
}

pub fn fill_checksum(packet: &mut [u8]) -> bool {
    let Some((start, proto, pseudo)) = transport(packet) else {
        return false;
    };
    let offset = match proto {
        TCP => 16,
        UDP => 6,
        _ => return false,
    };
    let Some(segment) = packet.get_mut(start..) else {
        return false;
    };
    if segment.len() < offset + 2 {
        return false;
    }
    segment[offset..offset + 2].fill(0);
    let checksum = match fold(pseudo + sum(segment)) {
        0 if proto == UDP => 0xffff,
        value => value,
    };
    segment[offset..offset + 2].copy_from_slice(&checksum.to_be_bytes());
    true
}
