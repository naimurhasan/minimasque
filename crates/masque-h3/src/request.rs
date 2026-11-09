use h3::ext::Protocol;
use http::{Request, Response, StatusCode};
use masque_core::connect_ip::{self, Scope};
use masque_core::{Target, connect_udp};

pub const CAPSULE_PROTOCOL: &str = "capsule-protocol";

pub fn protocol(req: &Request<()>) -> Option<&str> {
    req.extensions().get::<Protocol>().map(Protocol::as_str)
}

fn path(req: &Request<()>) -> &str {
    req.uri().path_and_query().map_or("", |p| p.as_str())
}

pub fn validate(req: &Request<()>, template: &str) -> Result<Target, StatusCode> {
    connect_udp::accept(req.method().as_str(), protocol(req), path(req), template)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

pub fn validate_ip(req: &Request<()>, template: &str) -> Result<Scope, StatusCode> {
    connect_ip::accept(req.method().as_str(), protocol(req), path(req), template)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

pub fn is_connect_ip(req: &Request<()>) -> bool {
    protocol(req) == Some(connect_ip::PROTOCOL)
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
