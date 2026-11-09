use std::net::SocketAddr;

use http::{HeaderMap, Request, Response};
use masque_core::ip_capsule::{Address, Range};

use crate::remote::Remote;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Request {
        stream_id: u64,
        method: String,
        protocol: Option<String>,
        authority: String,
        path: String,
        headers: Vec<(String, String)>,
    },
    Response {
        stream_id: u64,
        status: u16,
        headers: Vec<(String, String)>,
    },
    Opened {
        tunnel_id: u64,
        stream_id: u64,
        target: Remote,
    },
    Closed {
        tunnel_id: u64,
    },
    AddressAssign {
        stream_id: u64,
        addresses: Vec<Address>,
    },
    RouteAdvertisement {
        stream_id: u64,
        routes: Vec<Range>,
    },
    Migrated {
        from: SocketAddr,
        to: SocketAddr,
    },
}

pub fn headers(map: &HeaderMap) -> Vec<(String, String)> {
    map.iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect()
}

pub fn request(stream_id: u64, req: &Request<()>, protocol: Option<&str>) -> Event {
    Event::Request {
        stream_id,
        method: req.method().to_string(),
        protocol: protocol.map(str::to_owned),
        authority: req
            .uri()
            .authority()
            .map(ToString::to_string)
            .unwrap_or_default(),
        path: req
            .uri()
            .path_and_query()
            .map(ToString::to_string)
            .unwrap_or_default(),
        headers: headers(req.headers()),
    }
}

pub fn response(stream_id: u64, resp: &Response<()>) -> Event {
    Event::Response {
        stream_id,
        status: resp.status().as_u16(),
        headers: headers(resp.headers()),
    }
}
