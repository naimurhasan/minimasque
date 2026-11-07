use std::sync::Arc;

use bytes::Bytes;
use h3::server::RequestResolver;
use http::StatusCode;
use quinn::{Endpoint, Incoming};

use crate::router::{self, Router};
use crate::stats::Stats;
use crate::tunnel::Tunnel;
use crate::{Error, event, request, udp};

type Resolver = RequestResolver<h3_quinn::Connection, Bytes>;

pub async fn serve(endpoint: Endpoint, template: Arc<str>, stats: Arc<Stats>) {
    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(connection(incoming, template.clone(), stats.clone()));
    }
}

async fn connection(
    incoming: Incoming,
    template: Arc<str>,
    stats: Arc<Stats>,
) -> Result<(), Error> {
    let quic = incoming.await?;
    let router = Arc::new(Router::default());
    tokio::spawn(router::demux(quic.clone(), router.clone()));
    let mut h3 = h3::server::builder()
        .enable_extended_connect(true)
        .enable_datagram(true)
        .build(h3_quinn::Connection::new(quic.clone()))
        .await?;
    while let Some(resolver) = h3.accept().await? {
        tokio::spawn(handle(
            resolver,
            quic.clone(),
            router.clone(),
            template.clone(),
            stats.clone(),
        ));
    }
    Ok(())
}

type Stream = h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>;

async fn handle(
    resolver: Resolver,
    quic: quinn::Connection,
    router: Arc<Router>,
    template: Arc<str>,
    stats: Arc<Stats>,
) -> Result<(), Error> {
    let (req, mut stream) = resolver.resolve_request().await?;
    let stream_id = stream.id().into_inner();
    stats.emit(event::request(stream_id, &req, request::protocol(&req)));
    let target = match request::validate(&req, &template) {
        Ok(target) => target,
        Err(status) => return reject(stream, &stats, status).await,
    };
    let Ok(socket) = udp::connect(&target).await else {
        return reject(stream, &stats, StatusCode::BAD_GATEWAY).await;
    };
    let tracked = stats.track(stream_id, target);
    let inbound = router.register(stream_id);
    respond(&mut stream, &stats, StatusCode::OK).await?;
    Tunnel {
        quic,
        router,
        stream_id,
        socket,
        tracked,
    }
    .relay(inbound, &mut stream)
    .await
}

async fn respond(stream: &mut Stream, stats: &Stats, status: StatusCode) -> Result<(), Error> {
    let resp = request::response(status);
    stats.emit(event::response(stream.id().into_inner(), &resp));
    Ok(stream.send_response(resp).await?)
}

async fn reject(mut stream: Stream, stats: &Stats, status: StatusCode) -> Result<(), Error> {
    respond(&mut stream, stats, status).await?;
    Ok(stream.finish().await?)
}
