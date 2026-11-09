use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use masque_core::ip_capsule::Prefix;
use masque_core::{Target, connect_ip, connect_udp};

use masque_client::transport::Transport;

#[derive(Parser)]
#[command(
    version,
    about = "MASQUE proxy and client: CONNECT-UDP over HTTP/3 with HTTP/2 fallback, and CONNECT-IP over HTTP/3"
)]
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
    #[command(about = "Tunnel IP packets through the proxy with CONNECT-IP (needs root)")]
    Ip(IpArgs),
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
    #[arg(long, short, help = "Print protocol events to stderr")]
    pub verbose: bool,
    #[arg(
        long,
        help = "Also serve CONNECT-IP, assigning addresses from this IPv4 pool through a TUN device (needs root), e.g. 10.89.0.0/24"
    )]
    pub ip_pool: Option<Prefix>,
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
    #[arg(
        long,
        default_value = "auto",
        help = "auto | h3 | h2 (auto tries HTTP/3, then falls back to HTTP/2)"
    )]
    pub transport: Transport,
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

#[derive(Args)]
pub struct IpArgs {
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
        help = "Destination to send through the tunnel, repeatable, e.g. 1.1.1.1/32"
    )]
    pub route: Vec<Prefix>,
    #[arg(
        long,
        help = "Proxy URI template [default: https://<name>:<port>/.well-known/masque/ip/...]"
    )]
    pub template: Option<String>,
    #[arg(long, help = "Show the live dashboard")]
    pub tui: bool,
}

impl IpArgs {
    pub fn template(&self) -> String {
        self.template.clone().unwrap_or_else(|| {
            connect_ip::well_known_url(&format!("{}:{}", self.name, self.proxy.port()))
        })
    }
}
