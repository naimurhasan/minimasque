use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use masque_core::{Target, connect_udp};

#[derive(Parser)]
#[command(version, about = "MASQUE CONNECT-UDP proxy and client over HTTP/3")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Run the MASQUE proxy")]
    Server(ServerArgs),
    #[command(about = "Forward a local UDP port through the proxy")]
    Client(ClientArgs),
}

#[derive(Args)]
pub struct ServerArgs {
    #[arg(long, default_value = "0.0.0.0:4433", help = "Address to listen on")]
    pub listen: SocketAddr,
    #[arg(long, default_values = ["localhost"], help = "Certificate names (DNS or IP), repeatable")]
    pub name: Vec<String>,
    #[arg(
        long,
        default_value = "minimasque-cert.der",
        help = "Where to write the generated certificate"
    )]
    pub cert_out: PathBuf,
    #[arg(long, help = "Show the live dashboard")]
    pub tui: bool,
}

#[derive(Args)]
pub struct ClientArgs {
    #[arg(long, help = "Proxy address, e.g. 192.168.1.50:4433")]
    pub proxy: SocketAddr,
    #[arg(
        long,
        default_value = "localhost",
        help = "Proxy name to verify in its certificate"
    )]
    pub name: String,
    #[arg(
        long,
        default_value = "minimasque-cert.der",
        help = "Proxy certificate to trust"
    )]
    pub ca: PathBuf,
    #[arg(
        long,
        default_value = "127.0.0.1:5300",
        help = "Local UDP address apps send to"
    )]
    pub local: SocketAddr,
    #[arg(long, help = "UDP target behind the proxy, e.g. 1.1.1.1:53")]
    pub target: Target,
    #[arg(
        long,
        help = "Proxy URI template [default: https://<name>:<port>/.well-known/masque/udp/...]"
    )]
    pub template: Option<String>,
    #[arg(long, help = "Show the live dashboard")]
    pub tui: bool,
}

impl ClientArgs {
    pub fn template(&self) -> String {
        self.template.clone().unwrap_or_else(|| {
            connect_udp::well_known_url(&format!("{}:{}", self.name, self.proxy.port()))
        })
    }
}
