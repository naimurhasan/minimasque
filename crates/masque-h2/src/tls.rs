use std::sync::Arc;

use masque_h3::{Error, tls};
use rustls::RootCertStore;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::{TlsAcceptor, TlsConnector};

use crate::ALPN;

pub fn acceptor(
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
) -> Result<TlsAcceptor, Error> {
    let mut config = rustls::ServerConfig::builder_with_provider(tls::provider())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    config.alpn_protocols = vec![ALPN.to_vec()];
    Ok(TlsAcceptor::from(Arc::new(config)))
}

pub fn connector(roots: RootCertStore) -> Result<TlsConnector, Error> {
    let mut config = rustls::ClientConfig::builder_with_provider(tls::provider())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![ALPN.to_vec()];
    Ok(TlsConnector::from(Arc::new(config)))
}
