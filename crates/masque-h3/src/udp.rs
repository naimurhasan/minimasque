use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use masque_core::Target;
use tokio::net::UdpSocket;

use crate::Error;

pub fn unspecified(remote: SocketAddr) -> SocketAddr {
    match remote {
        SocketAddr::V4(_) => (Ipv4Addr::UNSPECIFIED, 0).into(),
        SocketAddr::V6(_) => (Ipv6Addr::UNSPECIFIED, 0).into(),
    }
}

pub async fn connect(target: &Target) -> Result<UdpSocket, Error> {
    let remote = tokio::net::lookup_host((target.host.as_str(), target.port))
        .await?
        .next()
        .ok_or("target did not resolve")?;
    let socket = UdpSocket::bind(unspecified(remote)).await?;
    socket.connect(remote).await?;
    Ok(socket)
}
