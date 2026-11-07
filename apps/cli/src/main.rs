use std::process::ExitCode;

use clap::Parser;
use minimasque::cli::{Cli, Command};
use minimasque::{client, server};

#[tokio::main]
async fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Server(args) => server::run(args).await,
        Command::Client(args) => client::run(args).await,
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
