use std::sync::Arc;

use masque_core::connect_udp::WELL_KNOWN_PATH;
use masque_h3::stats::Stats;
use masque_h3::{Error, endpoint, server, tls};

use crate::cli::ServerArgs;
use crate::tui::{self, App, Feed};

pub async fn run(args: ServerArgs) -> Result<(), Error> {
    let (cert, key) = tls::self_signed(args.name)?;
    std::fs::write(&args.cert_out, &cert)?;
    let proxy = endpoint::server(args.listen, cert, key)?;
    let address = proxy.local_addr()?.to_string();
    println!(
        "proxy listening on {address} (cert: {})",
        args.cert_out.display()
    );
    let stats = Arc::new(Stats::default());
    let feed = Feed::new(stats.clone());
    let work = server::serve(proxy, Arc::from(WELL_KNOWN_PATH), stats.clone());
    tui::supervise(
        async move {
            work.await;
            Ok(())
        },
        args.tui.then(|| App::new("server", address)),
        feed,
    )
    .await
}
