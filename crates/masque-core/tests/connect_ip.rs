use masque_core::Error;
use masque_core::connect_ip::{self, Scope, WELL_KNOWN_PATH};

const URL: &str = "https://example.org/.well-known/masque/ip/{target}/{ipproto}/";

fn scope(target: Option<&str>, ipproto: Option<u8>) -> Scope {
    Scope {
        target: target.map(str::to_owned),
        ipproto,
    }
}

#[test]
fn expands_wildcards_literally() {
    assert_eq!(
        connect_ip::expand(URL, &Scope::default()),
        Ok("https://example.org/.well-known/masque/ip/*/*/".into())
    );
}

#[test]
fn percent_encodes_prefix_target() {
    assert_eq!(
        connect_ip::expand(URL, &scope(Some("192.0.2.0/24"), Some(17))),
        Ok("https://example.org/.well-known/masque/ip/192.0.2.0%2F24/17/".into())
    );
}

#[test]
fn parses_scoped_and_wildcard_paths() {
    assert_eq!(
        connect_ip::parse(WELL_KNOWN_PATH, "/.well-known/masque/ip/*/*/"),
        Ok(Scope::default())
    );
    assert_eq!(
        connect_ip::parse(
            WELL_KNOWN_PATH,
            "/.well-known/masque/ip/2001%3Adb8%3A%3A%2F32/6/"
        ),
        Ok(scope(Some("2001:db8::/32"), Some(6)))
    );
}

#[test]
fn round_trips_through_path() {
    let original = scope(Some("example.com"), Some(17));
    let path = connect_ip::expand(WELL_KNOWN_PATH, &original).unwrap();
    assert_eq!(connect_ip::parse(WELL_KNOWN_PATH, &path), Ok(original));
}

#[test]
fn rejects_invalid_paths() {
    for path in [
        "/.well-known/masque/ip/*/256/",
        "/.well-known/masque/ip/*/tcp/",
        "/.well-known/masque/ip//*/",
        "/.well-known/masque/ip/*/",
        "/.well-known/masque/udp/*/*/",
    ] {
        assert_eq!(
            connect_ip::parse(WELL_KNOWN_PATH, path),
            Err(Error::InvalidTarget),
            "{path}"
        );
    }
}

#[test]
fn accepts_only_connect_ip() {
    let path = "/.well-known/masque/ip/*/*/";
    assert!(connect_ip::accept("CONNECT", Some("connect-ip"), path, WELL_KNOWN_PATH).is_ok());
    for (method, protocol) in [
        ("GET", Some("connect-ip")),
        ("CONNECT", Some("connect-udp")),
        ("CONNECT", None),
    ] {
        assert_eq!(
            connect_ip::accept(method, protocol, path, WELL_KNOWN_PATH),
            Err(Error::InvalidRequest)
        );
    }
}
