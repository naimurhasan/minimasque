use std::sync::Arc;

use bytes::Bytes;
use masque_core::ip_packet;
use masque_h3::Error;
use masque_h3::gateway::Gateway;
use masque_h3::ip_session::IpSession;
use tokio::sync::mpsc::Receiver;
use tun_rs::AsyncDevice;

const MAX_PACKET: usize = 65_535;

pub async fn proxy(
    device: AsyncDevice,
    gateway: Arc<Gateway>,
    mut outbound: Receiver<Bytes>,
) -> Result<(), Error> {
    let mut buf = vec![0; MAX_PACKET];
    loop {
        tokio::select! {
            Some(packet) = outbound.recv() => {
                device.send(&packet).await?;
            }
            received = device.recv(&mut buf) => {
                let packet = &mut buf[..received?];
                ip_packet::fill_checksum(packet);
                gateway.deliver(Bytes::copy_from_slice(packet));
            }
        }
    }
}

pub async fn client(device: AsyncDevice, mut session: IpSession) -> Result<(), Error> {
    let mut buf = vec![0; MAX_PACKET];
    loop {
        tokio::select! {
            packet = session.recv() => {
                let Some(packet) = packet else { return Ok(()) };
                device.send(&packet).await?;
            }
            received = device.recv(&mut buf) => {
                let _ = session.send(&buf[..received?]);
            }
        }
    }
}
