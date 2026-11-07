use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use masque_core::Target;
use masque_core::connect_udp::WELL_KNOWN_PATH;
use masque_h2::client::Client;
use masque_h2::{server, tls};
use masque_h3::event::Event;
use masque_h3::stats::Stats;
use rustls::RootCertStore;
use tokio::net::{TcpListener, UdpSocket};

const URL: &str = "https://localhost/.well-known/masque/udp/{target_host}/{target_port}/";

async fn start(stats: Arc<Stats>) -> (SocketAddr, RootCertStore) {
    let (cert, key) = masque_h3::tls::self_signed(vec!["localhost".into()]).unwrap();
    let mut roots = RootCertStore::empty();
    roots.add(cert.clone()).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let acceptor = tls::acceptor(cert, key).unwrap();
    tokio::spawn(server::serve(
        listener,
        acceptor,
        Arc::from(WELL_KNOWN_PATH),
        stats,
    ));
    (addr, roots)
}

async fn client(stats: Arc<Stats>) -> Client {
    let (addr, roots) = start(stats).await;
    Client::connect(addr, "localhost", &tls::connector(roots).unwrap())
        .await
        .unwrap()
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
async fn relays_udp_over_http2_capsules() {
    let client = client(Arc::default()).await;
    let mut tunnel = client.open(URL, &local(echo().await)).await.unwrap();
    for payload in [&b"one"[..], b"two", &[7; 1400]] {
        tunnel.send(payload).unwrap();
        let reply = tokio::time::timeout(Duration::from_secs(2), tunnel.recv())
            .await
            .unwrap();
        assert_eq!(reply.as_deref(), Some(payload));
    }
    let totals = client.stats().snapshot().totals;
    assert_eq!((totals.packets_up, totals.packets_down), (3, 3));
}

#[tokio::test]
async fn refuses_bad_target() {
    let client = client(Arc::default()).await;
    let refused = client
        .open(
            "https://localhost/wrong/{target_host}/{target_port}/",
            &local(53),
        )
        .await;
    assert!(refused.err().unwrap().to_string().contains("400"));
}

#[tokio::test]
async fn proxy_reports_extended_connect() {
    let stats = Arc::new(Stats::default());
    let mut events = stats.subscribe();
    let client = client(stats).await;
    let _tunnel = client.open(URL, &local(echo().await)).await.unwrap();
    let event = tokio::time::timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
        .unwrap();
    let Event::Request {
        method,
        protocol,
        headers,
        ..
    } = event
    else {
        panic!("expected request")
    };
    assert_eq!(
        (method.as_str(), protocol.as_deref()),
        ("CONNECT", Some("connect-udp"))
    );
    assert!(headers.contains(&("capsule-protocol".into(), "?1".into())));
}
