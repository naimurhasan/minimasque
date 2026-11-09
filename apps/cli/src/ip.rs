use masque_client::transport::{ConnectOptions, Connected, Transport};
use masque_core::connect_ip::Scope;
use masque_h3::Error;
use masque_tun::{bridge, device, route};

use crate::cli::IpArgs;
use crate::trust;
use crate::tui::{self, App, Feed};

pub async fn run(args: IpArgs) -> Result<(), Error> {
    let options = ConnectOptions {
        proxy: args.proxy,
        name: args.name.clone(),
        transport: Transport::H3,
    };
    let client = Connected::connect(&options, trust::roots(&args.ca)?)
        .await?
        .into_h3()?;
    let feed = Feed::new(client.stats());
    let session = client.open_ip(&args.template(), &Scope::default()).await?;
    let address = session
        .addresses
        .first()
        .ok_or("proxy assigned no address")?
        .prefix;
    let tun = device::open(address)?;
    let name = device::name(&tun)?;
    for prefix in &args.route {
        route::add(prefix, &name).await?;
    }
    let routes: Vec<String> = args.route.iter().map(ToString::to_string).collect();
    let summary = format!("{address} on {name} · routes {}", routes.join(", "));
    println!("ip tunnel {summary} via {}", args.proxy);
    let migrator = client.migrator();
    tui::supervise(
        async move {
            let _client = client;
            bridge::client(tun, session).await
        },
        args.tui
            .then(|| App::new("ip", summary, "HTTP/3 · CONNECT-IP").with_migrator(Some(migrator))),
        feed,
    )
    .await
}
