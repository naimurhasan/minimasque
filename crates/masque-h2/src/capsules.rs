use bytes::{Bytes, BytesMut};
use masque_core::{Capsule, Datagram, Error, capsule, datagram};

pub fn encode_udp(payload: &[u8]) -> Result<Bytes, Error> {
    let mut inner = Vec::with_capacity(payload.len() + 1);
    datagram::encode(
        Datagram {
            context_id: datagram::UDP_PAYLOAD,
            payload,
        },
        &mut inner,
    )?;
    let mut out = Vec::with_capacity(inner.len() + 8);
    capsule::encode(
        Capsule {
            kind: capsule::DATAGRAM,
            payload: &inner,
        },
        &mut out,
    )?;
    Ok(out.into())
}

#[derive(Default)]
pub struct Reader {
    buf: BytesMut,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Decoded {
    pub udp: Vec<Bytes>,
    pub skipped: usize,
}

impl Reader {
    pub fn push(&mut self, chunk: &[u8]) -> Decoded {
        self.buf.extend_from_slice(chunk);
        let frozen = self.buf.split().freeze();
        let (capsules, used) = capsule::decode_all(&frozen);
        let mut decoded = Decoded::default();
        for found in capsules {
            match datagram::decode(found.payload) {
                Ok(d)
                    if found.kind == capsule::DATAGRAM && d.context_id == datagram::UDP_PAYLOAD =>
                {
                    decoded.udp.push(frozen.slice_ref(d.payload));
                }
                _ => decoded.skipped += 1,
            }
        }
        self.buf.extend_from_slice(&frozen[used..]);
        decoded
    }
}
