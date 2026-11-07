use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use masque_core::Target;
use masque_h3::client::Client;
use masque_h3::event::Event;
use masque_h3::stats::Stats;
use masque_h3::{endpoint, server, tls};
use rustls::RootCertStore;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::Receiver;

const PATH: &str = "/.well-known/masque/udp/{target_host}/{target_port}/";
const URL: &str = "https://localhost/.well-known/masque/udp/{target_host}/{target_port}/";

async fn start(stats: Arc<Stats>) -> (SocketAddr, RootCertStore) {
    let (cert, key) = tls::self_signed(vec!["localhost".into()]).unwrap();
    let mut roots = RootCertStore::empty();
    roots.add(cert.clone()).unwrap();
    let proxy = endpoint::server("127.0.0.1:0".parse().unwrap(), cert, key).unwrap();
    let addr = proxy.local_addr().unwrap();
    tokio::spawn(server::serve(proxy, Arc::from(PATH), stats));
    (addr, roots)
}

async fn client() -> Client {
    client_with(Arc::default()).await
}

async fn client_with(stats: Arc<Stats>) -> Client {
    let (addr, roots) = start(stats).await;
    let local = endpoint::client("127.0.0.1:0".parse().unwrap(), roots).unwrap();
    Client::connect(&local, addr, "localhost").await.unwrap()
}

async fn echo() -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut buf = [0; 1500];
        loop {
            let (n, from) = socket.recv_from(&mut buf).await.unwrap();
            socket.send_to(&buf[..n], from).await.unwrap();
        }
    });
    port
}

fn local(port: u16) -> Target {
    Target {
        host: "127.0.0.1".into(),
        port,
    }
}

#[tokio::test]
async fn relays_udp_both_ways() {
    let client = client().await;
    let mut tunnel = client.open(URL, &local(echo().await)).await.unwrap();
    tunnel.send(b"ping").unwrap();
    let reply = tokio::time::timeout(Duration::from_secs(2), tunnel.recv())
        .await
        .unwrap();
    assert_eq!(reply.as_deref(), Some(&b"ping"[..]));

    let snapshot = client.stats().snapshot();
    assert_eq!(snapshot.tunnels.len(), 1);
    assert_eq!(
        (snapshot.totals.bytes_up, snapshot.totals.bytes_down),
        (4, 4)
    );
    drop(tunnel);
    assert!(client.stats().snapshot().tunnels.is_empty());
}

#[tokio::test]
async fn multiplexes_tunnels_on_one_connection() {
    let client = client().await;
    let mut a = client.open(URL, &local(echo().await)).await.unwrap();
    let mut b = client.open(URL, &local(echo().await)).await.unwrap();
    a.send(b"a").unwrap();
    b.send(b"b").unwrap();
    let timeout = Duration::from_secs(2);
    assert_eq!(
        tokio::time::timeout(timeout, b.recv())
            .await
            .unwrap()
            .as_deref(),
        Some(&b"b"[..])
    );
    assert_eq!(
        tokio::time::timeout(timeout, a.recv())
            .await
            .unwrap()
            .as_deref(),
        Some(&b"a"[..])
    );
}

#[tokio::test]
async fn reports_refused_tunnel() {
    let client = client().await;
    let refused = client
        .open(
            "https://localhost/wrong/{target_host}/{target_port}/",
            &local(53),
        )
        .await;
    assert!(refused.err().unwrap().to_string().contains("400"));
}

async fn next(events: &mut Receiver<Event>) -> Event {
    tokio::time::timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn proxy_reports_protocol_events() {
    let stats = Arc::new(Stats::default());
    let mut events = stats.subscribe();
    let client = client_with(stats).await;
    let port = echo().await;
    let tunnel = client.open(URL, &local(port)).await.unwrap();
    drop(tunnel);

    let Event::Request {
        method,
        protocol,
        path,
        headers,
        stream_id,
        ..
    } = next(&mut events).await
    else {
        panic!("expected request")
    };
    assert_eq!(
        (method.as_str(), protocol.as_deref()),
        ("CONNECT", Some("connect-udp"))
    );
    assert_eq!(path, format!("/.well-known/masque/udp/127.0.0.1/{port}/"));
    assert!(headers.contains(&("capsule-protocol".into(), "?1".into())));
    assert!(
        matches!(next(&mut events).await, Event::Opened { stream_id: id, .. } if id == stream_id)
    );
    assert!(matches!(
        next(&mut events).await,
        Event::Response { status: 200, .. }
    ));
    assert!(matches!(next(&mut events).await, Event::Closed { .. }));
}
