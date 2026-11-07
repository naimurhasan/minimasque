use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

use masque_core::connect_udp::WELL_KNOWN_PATH;
use masque_ffi::{Forwarder, ForwarderConfig};
use masque_h3::{endpoint, server, tls};

fn background_proxy() -> (String, Vec<u8>, u16) {
    let (cert, key) = tls::self_signed(vec!["localhost".into()]).unwrap();
    let der = cert.to_vec();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (addr, echo_port) = runtime.block_on(async {
        let proxy = endpoint::server("127.0.0.1:0".parse().unwrap(), cert, key).unwrap();
        let addr = proxy.local_addr().unwrap();
        tokio::spawn(server::serve(
            proxy,
            Arc::from(WELL_KNOWN_PATH),
            Arc::default(),
        ));
        let echo = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = echo.local_addr().unwrap().port();
        tokio::spawn(async move {
            let mut buf = [0; 1500];
            while let Ok((n, from)) = echo.recv_from(&mut buf).await {
                let _ = echo.send_to(&buf[..n], from).await;
            }
        });
        (addr, port)
    });
    std::thread::spawn(move || runtime.block_on(std::future::pending::<()>()));
    (addr.to_string(), der, echo_port)
}

#[test]
fn forwards_local_udp_through_the_proxy() {
    let (proxy, ca_der, echo_port) = background_proxy();
    let forwarder = Forwarder::start(ForwarderConfig {
        proxy,
        server_name: "localhost".into(),
        ca_der,
        target: format!("127.0.0.1:{echo_port}"),
        local: "127.0.0.1:0".into(),
        transport: "h3".into(),
        template: None,
    })
    .unwrap();
    assert_eq!(forwarder.transport(), "HTTP/3 · QUIC");

    let app = UdpSocket::bind("127.0.0.1:0").unwrap();
    app.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    app.send_to(b"hello", forwarder.local_address()).unwrap();
    let mut buf = [0; 64];
    let (n, _) = app.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"hello");

    let stats = forwarder.stats();
    assert_eq!(
        (stats.open_tunnels, stats.bytes_up, stats.bytes_down),
        (1, 5, 5)
    );
    let events = forwarder.drain_events();
    assert!(
        events.iter().any(|line| line.contains("CONNECT")),
        "{events:?}"
    );
    assert!(forwarder.is_running());
    forwarder.stop();
}

#[test]
fn reports_bad_config_as_error() {
    let result = Forwarder::start(ForwarderConfig {
        proxy: "not-an-address".into(),
        server_name: "localhost".into(),
        ca_der: vec![],
        target: "1.1.1.1:53".into(),
        local: "127.0.0.1:0".into(),
        transport: "auto".into(),
        template: None,
    });
    assert!(result.is_err());
}
