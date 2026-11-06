use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use masque_core::Target;

#[derive(Parser)]
#[command(version, about = "MASQUE CONNECT-UDP proxy and client over HTTP/3")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Run the MASQUE proxy")]
    Server {
        #[arg(long, default_value = "0.0.0.0:4433", help = "Address to listen on")]
        listen: SocketAddr,
        #[arg(long, default_values = ["localhost"], help = "Certificate names (DNS or IP), repeatable")]
        name: Vec<String>,
        #[arg(
            long,
            default_value = "minimasque-cert.der",
            help = "Where to write the generated certificate"
        )]
        cert_out: PathBuf,
    },
    #[command(about = "Forward a local UDP port through the proxy")]
    Client {
        #[arg(long, help = "Proxy address, e.g. 192.168.1.50:4433")]
        proxy: SocketAddr,
        #[arg(
            long,
            default_value = "localhost",
            help = "Proxy name to verify in its certificate"
        )]
        name: String,
        #[arg(
            long,
            default_value = "minimasque-cert.der",
            help = "Proxy certificate to trust"
        )]
        ca: PathBuf,
        #[arg(
            long,
            default_value = "127.0.0.1:5300",
            help = "Local UDP address apps send to"
        )]
        local: SocketAddr,
        #[arg(long, help = "UDP target behind the proxy, e.g. 1.1.1.1:53")]
        target: Target,
    },
}
