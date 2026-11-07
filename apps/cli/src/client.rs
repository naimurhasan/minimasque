use masque_h3::client::Client;
use masque_h3::{Error, endpoint, udp};
use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;
use tokio::net::UdpSocket;

use crate::cli::ClientArgs;
use crate::forward;
use crate::tui::{self, App, Feed};

pub async fn run(args: ClientArgs) -> Result<(), Error> {
    let mut roots = RootCertStore::empty();
    let ca = std::fs::read(&args.ca)
        .map_err(|err| format!("cannot read --ca {}: {err}", args.ca.display()))?;
    roots.add(CertificateDer::from(ca))?;
    let quic = endpoint::client(udp::unspecified(args.proxy), roots)?;
    let client = Client::connect(&quic, args.proxy, &args.name).await?;
    let feed = Feed::new(client.stats());
    let tunnel = client.open(&args.template(), &args.target).await?;
    let socket = UdpSocket::bind(args.local).await?;
    let route = format!(
        "udp {} -> {}:{} via {}",
        socket.local_addr()?,
        args.target.host,
        args.target.port,
        args.proxy
    );
    println!("forwarding {route}");
    tui::supervise(
        async move {
            let _client = client;
            forward::run(socket, tunnel).await
        },
        args.tui.then(|| App::new("client", route)),
        feed,
    )
    .await
}
