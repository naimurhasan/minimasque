use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use masque_core::connect_udp::WELL_KNOWN_PATH;
use masque_h3::{Error, endpoint, server, tls};

pub async fn run(listen: SocketAddr, names: Vec<String>, cert_out: PathBuf) -> Result<(), Error> {
    let (cert, key) = tls::self_signed(names)?;
    std::fs::write(&cert_out, &cert)?;
    let proxy = endpoint::server(listen, cert, key)?;
    println!(
        "proxy listening on {} (cert: {})",
        proxy.local_addr()?,
        cert_out.display()
    );
    server::serve(proxy, Arc::from(WELL_KNOWN_PATH)).await;
    Ok(())
}
