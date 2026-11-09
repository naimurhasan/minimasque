use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use crate::{Error, varint};

pub const ADDRESS_ASSIGN: u64 = 0x01;
pub const ADDRESS_REQUEST: u64 = 0x02;
pub const ROUTE_ADVERTISEMENT: u64 = 0x03;

const V4: u8 = 4;
const V6: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prefix {
    pub addr: IpAddr,
    pub len: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address {
    pub request_id: u64,
    pub prefix: Prefix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: IpAddr,
    pub end: IpAddr,
    pub protocol: u8,
}

impl FromStr for Prefix {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Error> {
        let (addr, len) = input.split_once('/').ok_or(Error::InvalidTarget)?;
        let addr: IpAddr = addr.parse().map_err(|_| Error::InvalidTarget)?;
        let len: u8 = len.parse().map_err(|_| Error::InvalidTarget)?;
        if len > max_len(addr) {
            return Err(Error::InvalidTarget);
        }
        Ok(Self { addr, len })
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.addr, self.len)
    }
}

fn max_len(addr: IpAddr) -> u8 {
    match addr {
        IpAddr::V4(_) => 32,
        IpAddr::V6(_) => 128,
    }
}

fn put_ip(addr: IpAddr, out: &mut Vec<u8>) {
    match addr {
        IpAddr::V4(v4) => out.extend_from_slice(&v4.octets()),
        IpAddr::V6(v6) => out.extend_from_slice(&v6.octets()),
    }
}

fn version(addr: IpAddr) -> u8 {
    match addr {
        IpAddr::V4(_) => V4,
        IpAddr::V6(_) => V6,
    }
}

fn take<'a>(buf: &mut &'a [u8], len: usize) -> Result<&'a [u8], Error> {
    let (head, rest) = buf.split_at_checked(len).ok_or(Error::UnexpectedEnd)?;
    *buf = rest;
    Ok(head)
}

fn take_ip(buf: &mut &[u8], version: u8) -> Result<IpAddr, Error> {
    match version {
        V4 => Ok(IpAddr::V4(Ipv4Addr::from(
            <[u8; 4]>::try_from(take(buf, 4)?).map_err(|_| Error::UnexpectedEnd)?,
        ))),
        V6 => Ok(IpAddr::V6(Ipv6Addr::from(
            <[u8; 16]>::try_from(take(buf, 16)?).map_err(|_| Error::UnexpectedEnd)?,
        ))),
        _ => Err(Error::InvalidCapsule),
    }
}

pub fn encode_addresses(addresses: &[Address], out: &mut Vec<u8>) -> Result<(), Error> {
    for address in addresses {
        let Prefix { addr, len } = address.prefix;
        if len > max_len(addr) {
            return Err(Error::InvalidCapsule);
        }
        varint::encode(address.request_id, out)?;
        out.push(version(addr));
        put_ip(addr, out);
        out.push(len);
    }
    Ok(())
}

pub fn decode_addresses(mut buf: &[u8]) -> Result<Vec<Address>, Error> {
    let mut addresses = Vec::new();
    while !buf.is_empty() {
        let (request_id, used) = varint::decode(buf)?;
        buf = &buf[used..];
        let version = take(&mut buf, 1)?[0];
        let addr = take_ip(&mut buf, version)?;
        let len = take(&mut buf, 1)?[0];
        if len > max_len(addr) {
            return Err(Error::InvalidCapsule);
        }
        addresses.push(Address {
            request_id,
            prefix: Prefix { addr, len },
        });
    }
    Ok(addresses)
}

fn valid_range(range: &Range) -> bool {
    version(range.start) == version(range.end) && range.start <= range.end
}

fn ordered(prev: &Range, next: &Range) -> bool {
    let key = |range: &Range| (version(range.start), range.protocol);
    match key(prev).cmp(&key(next)) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Equal => prev.end < next.start,
        std::cmp::Ordering::Greater => false,
    }
}

fn valid_routes(ranges: &[Range]) -> bool {
    ranges.iter().all(valid_range) && ranges.windows(2).all(|pair| ordered(&pair[0], &pair[1]))
}

pub fn encode_routes(ranges: &[Range], out: &mut Vec<u8>) -> Result<(), Error> {
    if !valid_routes(ranges) {
        return Err(Error::InvalidCapsule);
    }
    for range in ranges {
        out.push(version(range.start));
        put_ip(range.start, out);
        put_ip(range.end, out);
        out.push(range.protocol);
    }
    Ok(())
}

pub fn decode_routes(mut buf: &[u8]) -> Result<Vec<Range>, Error> {
    let mut ranges = Vec::new();
    while !buf.is_empty() {
        let version = take(&mut buf, 1)?[0];
        let range = Range {
            start: take_ip(&mut buf, version)?,
            end: take_ip(&mut buf, version)?,
            protocol: take(&mut buf, 1)?[0],
        };
        ranges.push(range);
    }
    match valid_routes(&ranges) {
        true => Ok(ranges),
        false => Err(Error::InvalidCapsule),
    }
}
