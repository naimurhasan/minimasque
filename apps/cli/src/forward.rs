use masque_h3::Error;
use tokio::net::UdpSocket;

use crate::transport::Tunnel;

const MAX_UDP: usize = 65_535;

pub async fn run(socket: UdpSocket, mut tunnel: Tunnel) -> Result<(), Error> {
    let mut buf = vec![0; MAX_UDP];
    let mut peer = None;
    loop {
        tokio::select! {
            received = socket.recv_from(&mut buf) => {
                let (len, from) = received?;
                peer = Some(from);
                let _ = tunnel.send(&buf[..len]);
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
