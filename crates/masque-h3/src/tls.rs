use std::sync::Arc;

use rustls::crypto::{CryptoProvider, ring};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use crate::Error;

pub fn provider() -> Arc<CryptoProvider> {
    Arc::new(ring::default_provider())
}

pub fn self_signed(
    names: Vec<String>,
) -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>), Error> {
    let certified = rcgen::generate_simple_self_signed(names)?;
    let key = PrivatePkcs8KeyDer::from(certified.signing_key.serialize_der());
    Ok((certified.cert.der().clone(), key.into()))
}
