use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use masque_core::Target;
use masque_h3::migrate::Migrator;
use masque_h3::stats::Stats;
use masque_h3::{Error, endpoint, udp};
use rustls::RootCertStore;

const QUIC_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Auto,
    H3,
    H2,
}

#[derive(Clone, Debug)]
pub struct ConnectOptions {
    pub proxy: SocketAddr,
    pub name: String,
    pub transport: Transport,
}

impl FromStr for Transport {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "h3" => Ok(Self::H3),
            "h2" => Ok(Self::H2),
            other => Err(format!(
                "unknown transport {other:?}, expected auto, h3 or h2"
            )),
        }
    }
}

pub enum Connected {
    H3(masque_h3::client::Client),
    H2(masque_h2::client::Client),
}

pub enum Tunnel {
    H3(Box<(masque_h3::client::UdpTunnel, masque_h3::client::Client)>),
    H2(Box<(masque_h2::client::Tunnel, masque_h2::client::Client)>),
}

async fn quic(options: &ConnectOptions, roots: RootCertStore) -> Result<Connected, Error> {
    let local = endpoint::client(udp::unspecified(options.proxy), roots)?;
    let connect = masque_h3::client::Client::connect(&local, options.proxy, &options.name);
    let client = tokio::time::timeout(QUIC_TIMEOUT, connect)
        .await
        .map_err(|_| "QUIC handshake timed out")??;
    Ok(Connected::H3(client))
}

async fn tcp(options: &ConnectOptions, roots: RootCertStore) -> Result<Connected, Error> {
    let connector = masque_h2::tls::connector(roots)?;
    let client =
        masque_h2::client::Client::connect(options.proxy, &options.name, &connector).await?;
    Ok(Connected::H2(client))
}

impl Connected {
    pub async fn connect(options: &ConnectOptions, roots: RootCertStore) -> Result<Self, Error> {
        match options.transport {
            Transport::H3 => quic(options, roots).await,
            Transport::H2 => tcp(options, roots).await,
            Transport::Auto => match quic(options, roots.clone()).await {
                Ok(connected) => Ok(connected),
                Err(err) => {
                    eprintln!("HTTP/3 unavailable ({err}), falling back to HTTP/2");
                    tcp(options, roots).await
                }
            },
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::H3(_) => "HTTP/3 · QUIC",
            Self::H2(_) => "HTTP/2 · TCP",
        }
    }

    pub fn stats(&self) -> Arc<Stats> {
        match self {
            Self::H3(client) => client.stats(),
            Self::H2(client) => client.stats(),
        }
    }

    pub fn migrator(&self) -> Option<Migrator> {
        match self {
            Self::H3(client) => Some(client.migrator()),
            Self::H2(_) => None,
        }
    }

    pub async fn open(self, template: &str, target: &Target) -> Result<Tunnel, Error> {
        Ok(match self {
            Self::H3(client) => {
                Tunnel::H3(Box::new((client.open(template, target).await?, client)))
            }
            Self::H2(client) => {
                Tunnel::H2(Box::new((client.open(template, target).await?, client)))
            }
        })
    }
}

impl Tunnel {
    pub fn send(&mut self, payload: &[u8]) -> Result<(), Error> {
        match self {
            Self::H3(pair) => pair.0.send(payload),
            Self::H2(pair) => pair.0.send(payload),
        }
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        match self {
            Self::H3(pair) => pair.0.recv().await,
            Self::H2(pair) => pair.0.recv().await,
        }
    }
}
