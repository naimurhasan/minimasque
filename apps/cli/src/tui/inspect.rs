use masque_h3::event::Event;

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
            "● tunnel {tunnel_id} open  stream {stream_id} ⇄ {}:{}  context 0 = UDP payload",
            target.host, target.port
        )],
        Event::Closed { tunnel_id } => vec![format!("○ tunnel {tunnel_id} closed")],
    }
}
