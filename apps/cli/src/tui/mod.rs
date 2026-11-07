mod app;
mod feed;
mod history;
mod keys;
mod panels;
mod supervise;
mod view;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event};

pub use app::App;
pub use feed::Feed;
pub use history::{History, rate};
pub use keys::{migrates, quits};
pub use supervise::supervise;
pub use view::render;

const TICK: Duration = Duration::from_millis(250);

pub fn run(app: App, feed: Feed, stop: &AtomicBool) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, app, feed, stop);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut DefaultTerminal,
    mut app: App,
    mut feed: Feed,
    stop: &AtomicBool,
) -> io::Result<()> {
    let mut last = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        let (snapshot, events) = feed.poll();
        app.update(snapshot, events, now - last);
        last = now;
        terminal.draw(|frame| view::render(frame, &app))?;
        if event::poll(TICK)?
            && let Event::Key(key) = event::read()?
        {
            if quits(key) {
                return Ok(());
            }
            if migrates(key) {
                app.migrate();
            }
        }
    }
    Ok(())
}
