use masque_core::{Error, Target, connect_udp};

const URL: &str = "https://example.org/.well-known/masque/udp/{target_host}/{target_port}/";
const PATH: &str = "/.well-known/masque/udp/{target_host}/{target_port}/";

fn target(host: &str, port: u16) -> Target {
    Target {
        host: host.into(),
        port,
    }
}

#[test]
fn expands_dns_name() {
    assert_eq!(
        connect_udp::expand(URL, &target("192.0.2.6", 443)),
        Ok("https://example.org/.well-known/masque/udp/192.0.2.6/443/".into())
    );
}

#[test]
fn percent_encodes_ipv6() {
    assert_eq!(
        connect_udp::expand(URL, &target("2001:db8::42", 443)),
        Ok("https://example.org/.well-known/masque/udp/2001%3Adb8%3A%3A42/443/".into())
    );
}

#[test]
fn parses_request_path() {
    assert_eq!(
        connect_udp::parse(PATH, "/.well-known/masque/udp/2001%3Adb8%3A%3A42/53/"),
        Ok(target("2001:db8::42", 53))
    );
}

#[test]
fn round_trips_through_path() {
    let original = target("dns.example", 853);
    let path = connect_udp::expand(PATH, &original).unwrap();
    assert_eq!(connect_udp::parse(PATH, &path), Ok(original));
}

#[test]
fn rejects_invalid_paths() {
    for path in [
        "/.well-known/masque/udp/host/notaport/",
        "/.well-known/masque/udp/host/99999/",
        "/.well-known/masque/udp//443/",
        "/.well-known/masque/udp/host/443",
        "/other/host/443/",
        "/.well-known/masque/udp/%zz/443/",
    ] {
        assert_eq!(
            connect_udp::parse(PATH, path),
            Err(Error::InvalidTarget),
            "{path}"
        );
    }
}

#[test]
fn rejects_unsupported_template() {
    assert_eq!(
        connect_udp::expand(
            "https://p.example/masque{?target_host,target_port}",
            &target("h", 1)
        ),
        Err(Error::InvalidTemplate)
    );
}

#[test]
fn parses_host_port_strings() {
    assert_eq!("1.1.1.1:53".parse(), Ok(target("1.1.1.1", 53)));
    assert_eq!(
        "[2001:db8::42]:443".parse(),
        Ok(target("2001:db8::42", 443))
    );
    assert_eq!("dns.example:853".parse(), Ok(target("dns.example", 853)));
    for bad in ["1.1.1.1", ":53", "host:x", "[::1:53", "2001:db8::42"] {
        assert_eq!(bad.parse::<Target>(), Err(Error::InvalidTarget), "{bad}");
    }
}

#[test]
fn builds_well_known_url() {
    assert_eq!(
        connect_udp::well_known_url("proxy.example:4433"),
        "https://proxy.example:4433/.well-known/masque/udp/{target_host}/{target_port}/"
    );
}

#[test]
fn accepts_only_connect_udp_requests() {
    let path = "/.well-known/masque/udp/1.1.1.1/53/";
    assert_eq!(
        connect_udp::accept("CONNECT", Some("connect-udp"), path, PATH),
        Ok(target("1.1.1.1", 53))
    );
    for (method, protocol) in [
        ("GET", Some("connect-udp")),
        ("CONNECT", None),
        ("CONNECT", Some("connect-ip")),
    ] {
        assert_eq!(
            connect_udp::accept(method, protocol, path, PATH),
            Err(Error::InvalidRequest)
        );
    }
}
