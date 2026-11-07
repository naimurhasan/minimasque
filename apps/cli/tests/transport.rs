use std::net::SocketAddr;
use std::sync::Arc;

use masque_core::connect_udp::WELL_KNOWN_PATH;
use minimasque::cli::ClientArgs;
use minimasque::transport::{Connected, Transport};
use rustls::RootCertStore;
use tokio::net::TcpListener;

async fn http2_only_proxy() -> (SocketAddr, RootCertStore) {
    let (cert, key) = masque_h3::tls::self_signed(vec!["localhost".into()]).unwrap();
    let mut roots = RootCertStore::empty();
    roots.add(cert.clone()).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let acceptor = masque_h2::tls::acceptor(cert, key).unwrap();
    tokio::spawn(masque_h2::server::serve(
        listener,
        acceptor,
        Arc::from(WELL_KNOWN_PATH),
        Arc::default(),
    ));
    (addr, roots)
}

fn args(proxy: SocketAddr, transport: Transport) -> ClientArgs {
    ClientArgs {
        proxy,
        name: "localhost".into(),
        ca: "unused".into(),
        local: "127.0.0.1:0".parse().unwrap(),
        target: "127.0.0.1:53".parse().unwrap(),
        template: None,
        transport,
        tui: false,
    }
}

#[tokio::test]
async fn auto_falls_back_to_http2_when_quic_is_unreachable() {
    let (addr, roots) = http2_only_proxy().await;
    let connected = Connected::connect(&args(addr, Transport::Auto), roots)
        .await
        .unwrap();
    assert_eq!(connected.label(), "HTTP/2 · TCP");
}

#[tokio::test]
async fn forced_http3_fails_without_quic() {
    let (addr, roots) = http2_only_proxy().await;
    assert!(
        Connected::connect(&args(addr, Transport::H3), roots)
            .await
            .is_err()
    );
}
