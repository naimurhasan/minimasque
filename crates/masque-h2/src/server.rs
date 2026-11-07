use std::sync::Arc;

use bytes::Bytes;
use h2::server::SendResponse;
use h2::{RecvStream, SendStream};
use http::{Request, StatusCode};
use masque_core::connect_udp;
use masque_h3::stats::{Stats, Tracked};
use masque_h3::{Error, event, request, udp};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio_rustls::TlsAcceptor;

use crate::capsules::{Reader, encode_udp};

const MAX_UDP: usize = 65_535;

pub async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    template: Arc<str>,
    stats: Arc<Stats>,
) {
    while let Ok((tcp, _)) = listener.accept().await {
        tokio::spawn(connection(
            tcp,
            acceptor.clone(),
            template.clone(),
            stats.clone(),
        ));
    }
}

async fn connection(
    tcp: TcpStream,
    acceptor: TlsAcceptor,
    template: Arc<str>,
    stats: Arc<Stats>,
) -> Result<(), Error> {
    let tls = acceptor.accept(tcp).await?;
    let mut h2 = h2::server::Builder::new()
        .enable_connect_protocol()
        .handshake(tls)
        .await?;
    while let Some(accepted) = h2.accept().await {
        let (req, respond) = accepted?;
        tokio::spawn(handle(req, respond, template.clone(), stats.clone()));
    }
    Ok(())
}

async fn handle(
    req: Request<RecvStream>,
    mut respond: SendResponse<Bytes>,
    template: Arc<str>,
    stats: Arc<Stats>,
) -> Result<(), Error> {
    let stream_id = u64::from(u32::from(respond.stream_id()));
    let protocol = req
        .extensions()
        .get::<h2::ext::Protocol>()
        .map(|p| p.as_str().to_owned());
    let (parts, body) = req.into_parts();
    let head = Request::from_parts(parts, ());
    stats.emit(event::request(stream_id, &head, protocol.as_deref()));
    let path = head.uri().path_and_query().map_or("", |p| p.as_str());
    let Ok(target) =
        connect_udp::accept(head.method().as_str(), protocol.as_deref(), path, &template)
    else {
        return respond_with(&mut respond, &stats, stream_id, StatusCode::BAD_REQUEST).map(drop);
    };
    let Ok(socket) = udp::connect(&target).await else {
        return respond_with(&mut respond, &stats, stream_id, StatusCode::BAD_GATEWAY).map(drop);
    };
    let tracked = stats.track(stream_id, target);
    let send = respond_with(&mut respond, &stats, stream_id, StatusCode::OK)?;
    relay(send, body, socket, tracked).await
}

fn respond_with(
    respond: &mut SendResponse<Bytes>,
    stats: &Stats,
    stream_id: u64,
    status: StatusCode,
) -> Result<SendStream<Bytes>, Error> {
    let resp = request::response(status);
    stats.emit(event::response(stream_id, &resp));
    Ok(respond.send_response(resp, !status.is_success())?)
}

async fn relay(
    mut send: SendStream<Bytes>,
    mut body: RecvStream,
    socket: UdpSocket,
    tracked: Tracked,
) -> Result<(), Error> {
    let mut reader = Reader::default();
    let mut buf = vec![0; MAX_UDP];
    loop {
        tokio::select! {
            chunk = body.data() => {
                let Some(chunk) = chunk else { return Ok(()) };
                let chunk = chunk?;
                let _ = body.flow_control().release_capacity(chunk.len());
                let decoded = reader.push(&chunk);
                (0..decoded.skipped).for_each(|_| tracked.drop_one());
                for udp in decoded.udp {
                    match socket.send(&udp).await {
                        Ok(_) => tracked.up(udp.len()),
                        Err(_) => tracked.drop_one(),
                    }
                }
            }
            received = socket.recv(&mut buf) => {
                let len = received?;
                send.send_data(encode_udp(&buf[..len])?, false)?;
                tracked.down(len);
            }
        }
    }
}
