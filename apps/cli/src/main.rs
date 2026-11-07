use clap::Parser;
use minimasque::cli::{Cli, Command};
use minimasque::{client, server};

#[tokio::main]
async fn main() -> Result<(), masque_h3::Error> {
    match Cli::parse().command {
        Command::Server(args) => server::run(args).await,
        Command::Client(args) => client::run(args).await,
    }
}
