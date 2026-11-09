use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use masque_core::connect_ip::{Scope, WELL_KNOWN_PATH};
use masque_core::ip_capsule::Range;
use masque_core::ip_packet;
use masque_h3::client::Client;
use masque_h3::event::Event;
use masque_h3::gateway::Gateway;
use masque_h3::stats::Stats;
use masque_h3::{endpoint, server, tls};
use rustls::RootCertStore;

const UDP_PATH: &str = "/.well-known/masque/udp/{target_host}/{target_port}/";
const URL: &str = "https://localhost/.well-known/masque/ip/{target}/{ipproto}/";
const TIMEOUT: Duration = Duration::from_secs(2);

fn all_ipv4() -> Range {
    Range {
        start: "0.0.0.0".parse().unwrap(),
        end: "255.255.255.255".parse().unwrap(),
        protocol: 0,
    }
}

fn packet(src: [u8; 4], dst: [u8; 4], body: &[u8]) -> Vec<u8> {
    let mut out = vec![0x45, 0, 0, 0, 0, 0, 0, 0, 64, 17, 0, 0];
    out.extend_from_slice(&src);
    out.extend_from_slice(&dst);
    out.extend_from_slice(body);
    out
}

fn reflect(packet: &[u8]) -> Bytes {
    let mut reply = packet.to_vec();
    reply[12..16].copy_from_slice(&packet[16..20]);
    reply[16..20].copy_from_slice(&packet[12..16]);
    reply.into()
}

async fn start(gateway: Option<Arc<Gateway>>, stats: Arc<Stats>) -> (SocketAddr, RootCertStore) {
    let (cert, key) = tls::self_signed(vec!["localhost".into()]).unwrap();
    let mut roots = RootCertStore::empty();
    roots.add(cert.clone()).unwrap();
    let proxy = endpoint::server("127.0.0.1:0".parse().unwrap(), cert, key).unwrap();
    let addr = proxy.local_addr().unwrap();
    tokio::spawn(server::serve_with(
        proxy,
        Arc::from(UDP_PATH),
        gateway,
        stats,
    ));
    (addr, roots)
}

async fn reflecting_proxy(stats: Arc<Stats>) -> Client {
    let (gateway, mut outbound) = Gateway::new(
        Arc::from(WELL_KNOWN_PATH),
        ("10.89.0.0".parse().unwrap(), 24),
        vec![all_ipv4()],
    );
    let internet = gateway.clone();
    tokio::spawn(async move {
        while let Some(packet) = outbound.recv().await {
            internet.deliver(reflect(&packet));
        }
    });
    connect(Some(gateway), stats).await
}

async fn connect(gateway: Option<Arc<Gateway>>, stats: Arc<Stats>) -> Client {
    let (addr, roots) = start(gateway, stats).await;
    let local = endpoint::client("127.0.0.1:0".parse().unwrap(), roots).unwrap();
    Client::connect(&local, addr, "localhost").await.unwrap()
}

fn ip(text: &str) -> IpAddr {
    text.parse().unwrap()
}

#[tokio::test]
async fn assigns_address_and_relays_packets() {
    let client = reflecting_proxy(Arc::default()).await;
    let mut session = client.open_ip(URL, &Scope::default()).await.unwrap();
    assert_eq!(session.addresses[0].prefix.addr, ip("10.89.0.2"));
    assert_eq!(session.addresses[0].prefix.len, 32);
    assert_eq!(session.routes, [all_ipv4()]);

    session
        .send(&packet([10, 89, 0, 2], [1, 1, 1, 1], b"ping"))
        .unwrap();
    let reply = tokio::time::timeout(TIMEOUT, session.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ip_packet::source(&reply), Some(ip("1.1.1.1")));
    assert_eq!(ip_packet::destination(&reply), Some(ip("10.89.0.2")));
    assert_eq!(&reply[20..], b"ping");
}

#[tokio::test]
async fn gives_each_session_its_own_address() {
    let client = reflecting_proxy(Arc::default()).await;
    let first = client.open_ip(URL, &Scope::default()).await.unwrap();
    let second = client.open_ip(URL, &Scope::default()).await.unwrap();
    assert_eq!(
        [
            first.addresses[0].prefix.addr,
            second.addresses[0].prefix.addr
        ],
        [ip("10.89.0.2"), ip("10.89.0.3")]
    );
}

#[tokio::test]
async fn drops_packets_with_spoofed_source() {
    let proxy_stats = Arc::new(Stats::default());
    let client = reflecting_proxy(proxy_stats.clone()).await;
    let mut session = client.open_ip(URL, &Scope::default()).await.unwrap();
    session
        .send(&packet([10, 89, 0, 99], [1, 1, 1, 1], b"spoof"))
        .unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(300), session.recv())
            .await
            .is_err()
    );
    assert_eq!(proxy_stats.snapshot().totals.dropped, 1);
}

#[tokio::test]
async fn reports_capsules_on_both_ends() {
    let proxy_stats = Arc::new(Stats::default());
    let mut proxy_events = proxy_stats.subscribe();
    let client = reflecting_proxy(proxy_stats).await;
    let mut client_events = client.stats().subscribe();
    let _session = client.open_ip(URL, &Scope::default()).await.unwrap();
    for events in [&mut proxy_events, &mut client_events] {
        let mut seen = Vec::new();
        while let Ok(event) = events.try_recv() {
            seen.push(event);
        }
        assert!(
            seen.iter().any(
                |e| matches!(e, Event::Request { protocol: Some(p), .. } if p == "connect-ip")
            )
        );
        assert!(
            seen.iter()
                .any(|e| matches!(e, Event::AddressAssign { .. }))
        );
        assert!(
            seen.iter()
                .any(|e| matches!(e, Event::RouteAdvertisement { .. }))
        );
    }
}

#[tokio::test]
async fn refuses_connect_ip_without_gateway() {
    let client = connect(None, Arc::default()).await;
    let refused = client.open_ip(URL, &Scope::default()).await;
    assert!(refused.err().unwrap().to_string().contains("501"));
}
