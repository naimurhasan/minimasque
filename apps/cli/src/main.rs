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
        } => client::run(proxy, name, ca, local, target).await,
    }
}
