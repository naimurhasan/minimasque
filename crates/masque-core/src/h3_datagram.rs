use crate::{Error, varint};

pub fn encode(stream_id: u64, payload: &[u8], out: &mut Vec<u8>) -> Result<(), Error> {
    varint::encode(stream_id / 4, out)?;
    out.extend_from_slice(payload);
    Ok(())
}

pub fn decode(buf: &[u8]) -> Result<(u64, &[u8]), Error> {
    let (quarter, len) = varint::decode(buf)?;
    Ok((quarter * 4, &buf[len..]))
}
