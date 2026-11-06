use std::net::SocketAddr;
use std::path::PathBuf;

use masque_core::{Target, connect_udp};
use masque_h3::client::Client;
use masque_h3::{Error, endpoint, udp};
use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;
use tokio::net::UdpSocket;

use crate::forward;

pub async fn run(
    proxy: SocketAddr,
    name: String,
    ca: PathBuf,
    local: SocketAddr,
    target: Target,
) -> Result<(), Error> {
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(std::fs::read(ca)?))?;
    let quic = endpoint::client(udp::unspecified(proxy), roots)?;
    let client = Client::connect(&quic, proxy, &name).await?;
    let url = connect_udp::well_known_url(&format!("{name}:{}", proxy.port()));
    let tunnel = client.open(&url, &target).await?;
    let socket = UdpSocket::bind(local).await?;
    println!(
        "forwarding udp {} -> {}:{} via {proxy}",
        socket.local_addr()?,
        target.host,
        target.port
    );
    forward::run(socket, tunnel).await
}
