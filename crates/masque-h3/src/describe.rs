use std::net::SocketAddr;

use crate::event::Event;

fn protocol(number: u8) -> String {
    match number {
        0 => "any protocol".into(),
        6 => "tcp".into(),
        17 => "udp".into(),
        other => format!("protocol {other}"),
    }
}

fn address(addr: &SocketAddr) -> String {
    match addr.ip().is_unspecified() {
        true => format!("*:{}", addr.port()),
        false => addr.to_string(),
    }
}

fn header_lines(headers: &[(String, String)]) -> impl Iterator<Item = String> + '_ {
    headers
        .iter()
        .map(|(name, value)| format!("    {name}: {value}"))
}

pub fn lines(event: &Event) -> Vec<String> {
    match event {
        Event::Request {
            stream_id,
            method,
            protocol,
            authority,
            path,
            headers,
        } => [
            format!("▶ stream {stream_id}  {method}"),
            format!("    :protocol: {}", protocol.as_deref().unwrap_or("-")),
            format!("    :authority: {authority}"),
            format!("    :path: {path}"),
        ]
        .into_iter()
        .chain(header_lines(headers))
        .collect(),
        Event::Response {
            stream_id,
            status,
            headers,
        } => std::iter::once(format!("◀ stream {stream_id}  {status}"))
            .chain(header_lines(headers))
            .collect(),
        Event::Opened {
            tunnel_id,
            stream_id,
            target,
        } => vec![format!(
            "● tunnel {tunnel_id} open · stream {stream_id} ⇄ {target} · {} ctx 0",
            target.kind()
        )],
        Event::AddressAssign {
            stream_id,
            addresses,
        } => std::iter::once(format!("◇ stream {stream_id}  ADDRESS_ASSIGN"))
            .chain(addresses.iter().map(|address| {
                format!(
                    "    {}/{} · request {}",
                    address.prefix.addr, address.prefix.len, address.request_id
                )
            }))
            .collect(),
        Event::RouteAdvertisement { stream_id, routes } => {
            std::iter::once(format!("◇ stream {stream_id}  ROUTE_ADVERTISEMENT"))
                .chain(routes.iter().map(|range| {
                    format!(
                        "    {} – {} · {}",
                        range.start,
                        range.end,
                        protocol(range.protocol)
                    )
                }))
                .collect()
        }
        Event::Closed { tunnel_id } => vec![format!("○ tunnel {tunnel_id} closed")],
        Event::Migrated { from, to } => vec![
            format!("◆ path migrated {} → {}", address(from), address(to)),
            "    same connection · tunnels kept".into(),
        ],
    }
}
