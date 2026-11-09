use masque_core::ip_capsule::Prefix;
use masque_tun::route::{Os, command};

fn prefix(text: &str) -> Prefix {
    text.parse().unwrap()
}

#[test]
fn builds_linux_route() {
    assert_eq!(
        command(Os::Linux, &prefix("1.1.1.1/32"), "mm1"),
        ["ip", "route", "replace", "1.1.1.1/32", "dev", "mm1"]
    );
}

#[test]
fn builds_macos_route() {
    assert_eq!(
        command(Os::MacOs, &prefix("10.0.0.0/8"), "utun7"),
        [
            "route",
            "-n",
            "add",
            "-net",
            "10.0.0.0/8",
            "-interface",
            "utun7"
        ]
    );
}
