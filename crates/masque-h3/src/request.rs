use h3::ext::Protocol;
use http::{Method, Request, Response, StatusCode};
use masque_core::{Target, connect_udp};

pub const CAPSULE_PROTOCOL: &str = "capsule-protocol";

pub fn validate(req: &Request<()>, template: &str) -> Result<Target, StatusCode> {
    let is_connect_udp = req.method() == Method::CONNECT
        && req.extensions().get::<Protocol>() == Some(&Protocol::CONNECT_UDP);
    let path = req
        .uri()
        .path_and_query()
        .filter(|_| is_connect_udp)
        .ok_or(StatusCode::BAD_REQUEST)?;
    connect_udp::parse(template, path.as_str()).map_err(|_| StatusCode::BAD_REQUEST)
}

pub fn response(status: StatusCode) -> Response<()> {
    let builder = Response::builder().status(status);
    let builder = if status.is_success() {
        builder.header(CAPSULE_PROTOCOL, "?1")
    } else {
        builder
    };
    builder.body(()).unwrap_or_default()
}
