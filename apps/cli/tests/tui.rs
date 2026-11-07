use std::time::Duration;

use masque_h3::counters::CounterSnapshot;
use masque_h3::describe::lines as inspect_lines;
use masque_h3::event::Event;
use masque_h3::stats::Snapshot;
use minimasque::format;
use minimasque::tui::{App, History, quits, rate};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn formats_bytes() {
    assert_eq!(format::bytes(0), "0 B");
    assert_eq!(format::bytes(1023), "1023 B");
    assert_eq!(format::bytes(1536), "1.5 KiB");
    assert_eq!(format::bytes(5 * 1024 * 1024), "5.0 MiB");
}

#[test]
fn quits_on_q_esc_and_ctrl_c() {
    let key = |code, modifiers| KeyEvent::new(code, modifiers);
    assert!(quits(key(KeyCode::Char('q'), KeyModifiers::NONE)));
    assert!(quits(key(KeyCode::Esc, KeyModifiers::NONE)));
    assert!(quits(key(KeyCode::Char('c'), KeyModifiers::CONTROL)));
    assert!(!quits(key(KeyCode::Char('c'), KeyModifiers::NONE)));
}

#[test]
fn keeps_latest_events_only() {
    let mut app = App::new("server", "127.0.0.1:4433".into(), "HTTP/3 · QUIC");
    app.update(
        Snapshot::default(),
        (0..250).map(|tunnel_id| Event::Closed { tunnel_id }),
        Duration::from_secs(1),
    );
    assert_eq!(app.log.len(), 200);
    assert_eq!(app.log.front(), Some(&Event::Closed { tunnel_id: 50 }));
}

#[test]
fn formats_durations_and_rates() {
    assert_eq!(format::duration(Duration::from_secs(42)), "42s");
    assert_eq!(format::duration(Duration::from_secs(65)), "1m05s");
    assert_eq!(
        format::duration(Duration::from_secs(3 * 3600 + 120)),
        "3h02m"
    );
    assert_eq!(format::rate(2048), "2.0 KiB/s");
}

#[test]
fn computes_rates() {
    assert_eq!(rate(1000, 3000, Duration::from_millis(500)), 4000);
    assert_eq!(rate(3000, 1000, Duration::from_secs(1)), 0);
    assert_eq!(rate(0, 10, Duration::ZERO), 0);
}

#[test]
fn history_keeps_latest_samples() {
    let mut history = History::new(3);
    (1..=5).for_each(|sample| history.push(sample));
    assert_eq!(history.samples().collect::<Vec<_>>(), [3, 4, 5]);
    assert_eq!(history.latest(), 5);
}

#[test]
fn update_records_throughput() {
    let mut app = App::new("server", "127.0.0.1:4433".into(), "HTTP/3 · QUIC");
    let snapshot = Snapshot {
        totals: CounterSnapshot {
            bytes_up: 500,
            bytes_down: 2000,
            ..CounterSnapshot::default()
        },
        ..Snapshot::default()
    };
    app.update(snapshot, [], Duration::from_millis(250));
    assert_eq!((app.up.latest(), app.down.latest()), (2000, 8000));
}

fn screen(app: &App) -> String {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(110, 30)).unwrap();
    terminal
        .draw(|frame| minimasque::tui::render(frame, app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn renders_dashboard() {
    let mut app = App::new("server", "0.0.0.0:4433".into(), "HTTP/3 · QUIC");
    let snapshot = Snapshot {
        opened: 3,
        totals: CounterSnapshot {
            packets_up: 12,
            bytes_up: 1536,
            packets_down: 12,
            bytes_down: 9 * 1024,
            dropped: 1,
        },
        tunnels: vec![masque_h3::stats::TunnelSnapshot {
            id: 2,
            target: masque_core::Target {
                host: "1.1.1.1".into(),
                port: 53,
            },
            age: Duration::from_secs(65),
            counters: CounterSnapshot {
                packets_up: 4,
                bytes_up: 160,
                packets_down: 4,
                bytes_down: 900,
                dropped: 0,
            },
        }],
    };
    app.update(snapshot, [request()], Duration::from_secs(1));
    let screen = screen(&app);
    if std::env::var_os("SHOW_SCREEN").is_some() {
        println!("{screen}");
    }
    for expected in [
        "protocol inspector",
        ":protocol: connect-udp",
        "capsule-protocol: ?1",
        "minimasque",
        "server",
        "tunnels (1)",
        "1.1.1.1:53",
        "1m05s",
        "1.5 KiB",
        "9.0 KiB/s",
        "q quit",
    ] {
        assert!(screen.contains(expected), "missing {expected:?}\n{screen}");
    }
}

fn request() -> Event {
    Event::Request {
        stream_id: 0,
        method: "CONNECT".into(),
        protocol: Some("connect-udp".into()),
        authority: "localhost:4433".into(),
        path: "/.well-known/masque/udp/1.1.1.1/53/".into(),
        headers: vec![("capsule-protocol".into(), "?1".into())],
    }
}

#[test]
fn describes_protocol_events() {
    assert_eq!(
        inspect_lines(&request()),
        [
            "▶ stream 0  CONNECT",
            "    :protocol: connect-udp",
            "    :authority: localhost:4433",
            "    :path: /.well-known/masque/udp/1.1.1.1/53/",
            "    capsule-protocol: ?1",
        ]
    );
    let opened = Event::Opened {
        tunnel_id: 2,
        stream_id: 0,
        target: masque_core::Target {
            host: "1.1.1.1".into(),
            port: 53,
        },
    };
    assert_eq!(
        inspect_lines(&opened),
        ["● tunnel 2 open · stream 0 ⇄ 1.1.1.1:53 · udp ctx 0"]
    );
    assert_eq!(
        inspect_lines(&Event::Closed { tunnel_id: 2 }),
        ["○ tunnel 2 closed"]
    );
}

#[test]
fn feed_keeps_events_emitted_before_first_poll() {
    let stats = std::sync::Arc::new(masque_h3::stats::Stats::default());
    let mut feed = minimasque::tui::Feed::new(stats.clone());
    stats.emit(request());
    let tunnel = stats.track(
        0,
        masque_core::Target {
            host: "1.1.1.1".into(),
            port: 53,
        },
    );
    let (snapshot, events) = feed.poll();
    assert_eq!(snapshot.tunnels.len(), 1);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], request());
    drop(tunnel);
}
