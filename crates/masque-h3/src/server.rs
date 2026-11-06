use std::sync::Arc;

use bytes::Bytes;
use h3::server::RequestResolver;
use http::StatusCode;
use quinn::{Endpoint, Incoming};

use crate::router::{self, Router};
use crate::tunnel::Tunnel;
use crate::{Error, request, udp};

type Resolver = RequestResolver<h3_quinn::Connection, Bytes>;

pub async fn serve(endpoint: Endpoint, template: Arc<str>) {
    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(connection(incoming, template.clone()));
    }
}

async fn connection(incoming: Incoming, template: Arc<str>) -> Result<(), Error> {
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
        ));
    }
    Ok(())
}

async fn handle(
    resolver: Resolver,
    quic: quinn::Connection,
    router: Arc<Router>,
    template: Arc<str>,
) -> Result<(), Error> {
    let (req, mut stream) = resolver.resolve_request().await?;
    let target = match request::validate(&req, &template) {
        Ok(target) => target,
        Err(status) => return reject(stream, status).await,
    };
    let Ok(socket) = udp::connect(&target).await else {
        return reject(stream, StatusCode::BAD_GATEWAY).await;
    };
    let stream_id = stream.id().into_inner();
    let inbound = router.register(stream_id);
    stream
        .send_response(request::response(StatusCode::OK))
        .await?;
    Tunnel {
        quic,
        router,
        stream_id,
        socket,
    }
    .relay(inbound, &mut stream)
    .await
}

async fn reject(
    mut stream: h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    status: StatusCode,
) -> Result<(), Error> {
    stream.send_response(request::response(status)).await?;
    Ok(stream.finish().await?)
}
