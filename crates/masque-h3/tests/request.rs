use h3::ext::Protocol;
use http::{Method, Request, StatusCode};
use masque_core::Target;
use masque_h3::request;

const TEMPLATE: &str = "/.well-known/masque/udp/{target_host}/{target_port}/";

fn connect(path: &str, protocol: Option<Protocol>) -> Request<()> {
    let mut req = Request::builder()
        .method(Method::CONNECT)
        .uri(format!("https://proxy.example{path}"))
        .body(())
        .unwrap();
    if let Some(protocol) = protocol {
        req.extensions_mut().insert(protocol);
    }
    req
}

#[test]
fn accepts_connect_udp() {
    let req = connect(
        "/.well-known/masque/udp/1.1.1.1/53/",
        Some(Protocol::CONNECT_UDP),
    );
    assert_eq!(
        request::validate(&req, TEMPLATE),
        Ok(Target {
            host: "1.1.1.1".into(),
            port: 53
        })
    );
}

#[test]
fn rejects_wrong_protocol_or_method() {
    let path = "/.well-known/masque/udp/1.1.1.1/53/";
    let mut get = connect(path, Some(Protocol::CONNECT_UDP));
    *get.method_mut() = Method::GET;
    for req in [
        connect(path, None),
        connect(path, Some(Protocol::WEB_TRANSPORT)),
        get,
    ] {
        assert_eq!(
            request::validate(&req, TEMPLATE),
            Err(StatusCode::BAD_REQUEST)
        );
    }
}

#[test]
fn rejects_bad_target() {
    let req = connect(
        "/.well-known/masque/udp/1.1.1.1/x/",
        Some(Protocol::CONNECT_UDP),
    );
    assert_eq!(
        request::validate(&req, TEMPLATE),
        Err(StatusCode::BAD_REQUEST)
    );
}

#[test]
fn success_response_advertises_capsule_protocol() {
    let ok = request::response(StatusCode::OK);
    assert_eq!(ok.headers()[request::CAPSULE_PROTOCOL], "?1");
    let bad = request::response(StatusCode::BAD_REQUEST);
    assert!(!bad.headers().contains_key(request::CAPSULE_PROTOCOL));
}
