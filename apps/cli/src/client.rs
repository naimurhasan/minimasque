use masque_client::forward;
use masque_client::transport::{ConnectOptions, Connected};
use masque_h3::Error;
use tokio::net::UdpSocket;

use crate::cli::ClientArgs;
use crate::trust;
use crate::tui::{self, App, Feed};

pub async fn run(args: ClientArgs) -> Result<(), Error> {
    let roots = trust::roots(&args.ca)?;
    let options = ConnectOptions {
        proxy: args.proxy,
        name: args.name.clone(),
        transport: args.transport,
    };
    let connected = Connected::connect(&options, roots).await?;
    let transport = connected.label();
    let feed = Feed::new(connected.stats());
    let migrator = connected.migrator();
    let tunnel = connected.open(&args.template(), &args.target).await?;
    let socket = UdpSocket::bind(args.local).await?;
    let route = format!(
        "udp {} -> {}:{} via {}",
        socket.local_addr()?,
        args.target.host,
        args.target.port,
        args.proxy
    );
    println!("forwarding {route} over {transport}");
    tui::supervise(
        forward::run(socket, tunnel),
        args.tui
            .then(|| App::new("client", route, transport).with_migrator(migrator)),
        feed,
    )
    .await
}
