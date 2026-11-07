mod cli;
mod client;
mod forward;
mod server;

use clap::Parser;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<(), masque_h3::Error> {
    match Cli::parse().command {
        Command::Server {
            listen,
            name,
            cert_out,
        } => server::run(listen, name, cert_out).await,
        Command::Client {
            proxy,
            name,
            ca,
            local,
            target,
            template,
        } => {
            let template = template.unwrap_or_else(|| {
                masque_core::connect_udp::well_known_url(&format!("{name}:{}", proxy.port()))
            });
            client::run(proxy, name, ca, local, target, template).await
        }
    }
}
