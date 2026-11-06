use crate::{Error, varint};

pub const UDP_PAYLOAD: u64 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Datagram<'a> {
    pub context_id: u64,
    pub payload: &'a [u8],
}

pub fn encode(datagram: Datagram<'_>, out: &mut Vec<u8>) -> Result<(), Error> {
    varint::encode(datagram.context_id, out)?;
    out.extend_from_slice(datagram.payload);
    Ok(())
}

pub fn decode(buf: &[u8]) -> Result<Datagram<'_>, Error> {
    let (context_id, len) = varint::decode(buf)?;
    Ok(Datagram {
        context_id,
        payload: &buf[len..],
    })
}
