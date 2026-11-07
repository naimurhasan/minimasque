use std::sync::Arc;

use masque_core::connect_udp::WELL_KNOWN_PATH;
use masque_h3::stats::Stats;
use masque_h3::{Error, endpoint, server, tls};
use tokio::net::TcpListener;

use crate::cli::ServerArgs;
use crate::log;
use crate::tui::{self, App, Feed};

pub async fn run(args: ServerArgs) -> Result<(), Error> {
    let (cert, key) = tls::self_signed(args.name)?;
    std::fs::write(&args.cert_out, &cert)?;
    let quic = endpoint::server(args.listen, cert.clone(), key.clone_key())?;
    let address = quic.local_addr()?;
    let tcp = TcpListener::bind(address).await?;
    let acceptor = masque_h2::tls::acceptor(cert, key)?;
    println!(
        "proxy listening on {address} (udp: HTTP/3, tcp: HTTP/2) (cert: {})",
        args.cert_out.display()
    );
    let template: Arc<str> = Arc::from(WELL_KNOWN_PATH);
    let stats = Arc::new(Stats::default());
    let feed = Feed::new(stats.clone());
    if args.verbose && !args.tui {
        tokio::spawn(log::events(stats.clone()));
    }
    let h3 = server::serve(quic, template.clone(), stats.clone());
    let h2 = masque_h2::server::serve(tcp, acceptor, template, stats);
    tui::supervise(
        async move {
            tokio::join!(h3, h2);
            Ok(())
        },
        args.tui
            .then(|| App::new("server", address.to_string(), "HTTP/3 + HTTP/2")),
        feed,
    )
    .await
}
