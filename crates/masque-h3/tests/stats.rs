use std::sync::Arc;

use masque_core::Target;
use masque_h3::counters::CounterSnapshot;
use masque_h3::stats::Stats;

fn target(port: u16) -> Target {
    Target {
        host: "1.1.1.1".into(),
        port,
    }
}

#[test]
fn counts_per_tunnel_and_totals() {
    let stats = Arc::new(Stats::default());
    let dns = stats.track(0, target(53));
    let quic = stats.track(4, target(443));
    dns.up(40);
    dns.down(100);
    quic.up(1200);
    quic.drop_one();

    let snapshot = stats.snapshot();
    assert_eq!(snapshot.opened, 2);
    assert_eq!(
        snapshot.totals,
        CounterSnapshot {
            packets_up: 2,
            bytes_up: 1240,
            packets_down: 1,
            bytes_down: 100,
            dropped: 1
        }
    );
    let targets: Vec<_> = snapshot
        .tunnels
        .iter()
        .map(|t| t.target.to_string())
        .collect();
    assert_eq!(
        targets,
        [target(53), target(443)].map(|t| format!("{}:{}", t.host, t.port))
    );
    assert_eq!(snapshot.tunnels[0].counters.bytes_down, 100);
}

#[test]
fn closed_tunnels_leave_open_list_but_keep_totals() {
    let stats = Arc::new(Stats::default());
    let tunnel = stats.track(0, target(53));
    tunnel.up(10);
    drop(tunnel);

    let snapshot = stats.snapshot();
    assert!(snapshot.tunnels.is_empty());
    assert_eq!(snapshot.opened, 1);
    assert_eq!(snapshot.totals.bytes_up, 10);
}
