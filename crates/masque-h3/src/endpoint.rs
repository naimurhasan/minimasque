use std::net::SocketAddr;
use std::sync::Arc;

use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::RootCertStore;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::{ALPN, Error, tls};

pub fn server(
    addr: SocketAddr,
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
) -> Result<Endpoint, Error> {
    let mut crypto = rustls::ServerConfig::builder_with_provider(tls::provider())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    crypto.alpn_protocols = vec![ALPN.to_vec()];
    let config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(crypto)?));
    Ok(Endpoint::server(config, addr)?)
}

pub fn client(bind: SocketAddr, roots: RootCertStore) -> Result<Endpoint, Error> {
    let mut crypto = rustls::ClientConfig::builder_with_provider(tls::provider())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_root_certificates(roots)
        .with_no_client_auth();
    crypto.alpn_protocols = vec![ALPN.to_vec()];
    let mut endpoint = Endpoint::client(bind)?;
    endpoint.set_default_client_config(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
        crypto,
    )?)));
    Ok(endpoint)
}
