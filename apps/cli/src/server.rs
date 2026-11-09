use std::sync::Arc;

use masque_core::connect_ip;
use masque_core::connect_udp::WELL_KNOWN_PATH;
use masque_core::ip_capsule::{Prefix, Range};
use masque_h3::gateway::{self, Gateway};
use masque_h3::stats::Stats;
use masque_h3::{Error, endpoint, server, tls};
use masque_tun::{bridge, device};
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
    let gateway = args.ip_pool.map(ip_gateway).transpose()?;
    let h3 = server::serve_with(quic, template.clone(), gateway, stats.clone());
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

fn all_ipv4() -> Range {
    Range {
        start: std::net::IpAddr::from([0, 0, 0, 0]),
        end: std::net::IpAddr::from([255, 255, 255, 255]),
        protocol: 0,
    }
}

fn ip_gateway(pool: Prefix) -> Result<Arc<Gateway>, Error> {
    let std::net::IpAddr::V4(network) = pool.addr else {
        return Err("--ip-pool must be IPv4".into());
    };
    let own = gateway::host(network, pool.len, 1).ok_or("--ip-pool is too small")?;
    let tun = device::open(Prefix {
        addr: own.into(),
        len: pool.len,
    })?;
    println!(
        "connect-ip pool {pool} on {} (proxy side {own})",
        device::name(&tun)?
    );
    let (gateway, outbound) = Gateway::new(
        Arc::from(connect_ip::WELL_KNOWN_PATH),
        (network, pool.len),
        vec![all_ipv4()],
    );
    tokio::spawn(bridge::proxy(tun, gateway.clone(), outbound));
    Ok(gateway)
}
