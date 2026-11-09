use bytes::{Buf, Bytes};
use masque_core::{Capsule, capsule};

use crate::Error;

#[derive(Default)]
pub struct Reader(Vec<u8>);

impl Reader {
    pub fn push(&mut self, mut chunk: impl Buf) -> Vec<(u64, Vec<u8>)> {
        while chunk.has_remaining() {
            let part = chunk.chunk();
            self.0.extend_from_slice(part);
            let len = part.len();
            chunk.advance(len);
        }
        let (capsules, used) = capsule::decode_all(&self.0);
        let owned = capsules
            .into_iter()
            .map(|capsule| (capsule.kind, capsule.payload.to_vec()))
            .collect();
        self.0.drain(..used);
        owned
    }
}

pub fn encode(kind: u64, payload: &[u8]) -> Result<Bytes, Error> {
    let mut out = Vec::with_capacity(payload.len() + 8);
    capsule::encode(Capsule { kind, payload }, &mut out)?;
    Ok(out.into())
}
