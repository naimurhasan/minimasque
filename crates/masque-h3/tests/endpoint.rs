use masque_h3::{ALPN, endpoint, tls};
use rustls::RootCertStore;

#[tokio::test]
async fn completes_quic_handshake_with_h3_alpn() {
    let (cert, key) = tls::self_signed(vec!["localhost".into()]).unwrap();
    let mut roots = RootCertStore::empty();
    roots.add(cert.clone()).unwrap();

    let server = endpoint::server("127.0.0.1:0".parse().unwrap(), cert, key).unwrap();
    let addr = server.local_addr().unwrap();
    let accept = tokio::spawn(async move { server.accept().await.unwrap().await.unwrap() });

    let client = endpoint::client("127.0.0.1:0".parse().unwrap(), roots).unwrap();
    let conn = client.connect(addr, "localhost").unwrap().await.unwrap();
    let alpn = conn
        .handshake_data()
        .unwrap()
        .downcast::<quinn::crypto::rustls::HandshakeData>()
        .unwrap()
        .protocol;

    assert_eq!(alpn.as_deref(), Some(ALPN));
    assert_eq!(
        accept.await.unwrap().remote_address(),
        client.local_addr().unwrap()
    );
}
