use crate::{Error, varint};

pub const DATAGRAM: u64 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capsule<'a> {
    pub kind: u64,
    pub payload: &'a [u8],
}

pub fn encode(capsule: Capsule<'_>, out: &mut Vec<u8>) -> Result<(), Error> {
    varint::encode(capsule.kind, out)?;
    varint::encode(capsule.payload.len() as u64, out)?;
    out.extend_from_slice(capsule.payload);
    Ok(())
}

pub fn decode(buf: &[u8]) -> Result<(Capsule<'_>, usize), Error> {
    let (kind, kind_len) = varint::decode(buf)?;
    let (len, len_len) = varint::decode(&buf[kind_len..])?;
    let start = kind_len + len_len;
    let end = usize::try_from(len)
        .ok()
        .and_then(|len| start.checked_add(len))
        .ok_or(Error::UnexpectedEnd)?;
    let payload = buf.get(start..end).ok_or(Error::UnexpectedEnd)?;
    Ok((Capsule { kind, payload }, end))
}

pub fn decode_all(buf: &[u8]) -> (Vec<Capsule<'_>>, usize) {
    let mut capsules = Vec::new();
    let mut used = 0;
    while let Ok((capsule, len)) = decode(&buf[used..]) {
        capsules.push(capsule);
        used += len;
    }
    (capsules, used)
}
