use crate::Error;

pub const MAX: u64 = (1 << 62) - 1;

pub fn encoded_len(value: u64) -> Result<usize, Error> {
    match value {
        0..=0x3f => Ok(1),
        0x40..=0x3fff => Ok(2),
        0x4000..=0x3fff_ffff => Ok(4),
        0x4000_0000..=MAX => Ok(8),
        _ => Err(Error::VarIntOverflow),
    }
}

pub fn encode(value: u64, out: &mut Vec<u8>) -> Result<(), Error> {
    let len = encoded_len(value)?;
    let tag = u64::from(len.trailing_zeros()) << (len * 8 - 2);
    out.extend_from_slice(&(value | tag).to_be_bytes()[8 - len..]);
    Ok(())
}

pub fn decode(buf: &[u8]) -> Result<(u64, usize), Error> {
    let first = *buf.first().ok_or(Error::UnexpectedEnd)?;
    let len = 1usize << (first >> 6);
    let bytes = buf.get(1..len).ok_or(Error::UnexpectedEnd)?;
    let value = bytes
        .iter()
        .fold(u64::from(first & 0x3f), |acc, &b| (acc << 8) | u64::from(b));
    Ok((value, len))
}
