use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use quinn::{ClientConfig, Endpoint, ServerConfig, TransportConfig};
use rustls::RootCertStore;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::{ALPN, Error, tls};

pub const KEEP_ALIVE: Duration = Duration::from_secs(10);

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
    let mut transport = TransportConfig::default();
    transport.keep_alive_interval(Some(KEEP_ALIVE));
    let mut config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));
    config.transport_config(Arc::new(transport));
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}
