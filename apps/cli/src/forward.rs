use masque_h3::Error;
use masque_h3::client::UdpTunnel;
use tokio::net::UdpSocket;

const MAX_UDP: usize = 65_535;

pub async fn run(socket: UdpSocket, mut tunnel: UdpTunnel) -> Result<(), Error> {
    let mut buf = vec![0; MAX_UDP];
    let mut peer = None;
    loop {
        tokio::select! {
            received = socket.recv_from(&mut buf) => {
                let (len, from) = received?;
                peer = Some(from);
                if let Err(err) = tunnel.send(&buf[..len]) {
                    eprintln!("dropped packet: {err}");
                }
            }
            reply = tunnel.recv() => {
                let Some(reply) = reply else { return Ok(()) };
                if let Some(peer) = peer {
                    socket.send_to(&reply, peer).await?;
                }
            }
        }
    }
}
