use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use clap::ValueEnum;
use masque_core::Target;
use masque_h3::stats::Stats;
use masque_h3::{Error, endpoint, udp};
use rustls::RootCertStore;

use crate::cli::ClientArgs;

const QUIC_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Transport {
    Auto,
    H3,
    H2,
}

pub enum Connected {
    H3(masque_h3::client::Client),
    H2(masque_h2::client::Client),
}

pub enum Tunnel {
    H3(Box<(masque_h3::client::UdpTunnel, masque_h3::client::Client)>),
    H2(Box<(masque_h2::client::Tunnel, masque_h2::client::Client)>),
}

async fn quic(args: &ClientArgs, roots: RootCertStore) -> Result<Connected, Error> {
    let local = endpoint::client(udp::unspecified(args.proxy), roots)?;
    let connect = masque_h3::client::Client::connect(&local, args.proxy, &args.name);
    let client = tokio::time::timeout(QUIC_TIMEOUT, connect)
        .await
        .map_err(|_| "QUIC handshake timed out")??;
    Ok(Connected::H3(client))
}

async fn tcp(args: &ClientArgs, roots: RootCertStore) -> Result<Connected, Error> {
    let connector = masque_h2::tls::connector(roots)?;
    let client = masque_h2::client::Client::connect(args.proxy, &args.name, &connector).await?;
    Ok(Connected::H2(client))
}

impl Connected {
    pub async fn connect(args: &ClientArgs, roots: RootCertStore) -> Result<Self, Error> {
        match args.transport {
            Transport::H3 => quic(args, roots).await,
            Transport::H2 => tcp(args, roots).await,
            Transport::Auto => match quic(args, roots.clone()).await {
                Ok(connected) => Ok(connected),
                Err(err) => {
                    eprintln!("HTTP/3 unavailable ({err}), falling back to HTTP/2");
                    tcp(args, roots).await
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
