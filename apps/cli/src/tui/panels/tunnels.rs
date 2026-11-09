use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::widgets::{Block, Row, Table};

use crate::format;
use crate::tui::App;

const WIDTHS: [Constraint; 6] = [
    Constraint::Length(4),
    Constraint::Min(16),
    Constraint::Length(7),
    Constraint::Length(10),
    Constraint::Length(10),
    Constraint::Length(7),
];

pub fn tunnels(app: &App) -> Table<'_> {
    let rows = app.snapshot.tunnels.iter().map(|tunnel| {
        let c = &tunnel.counters;
        Row::new([
            tunnel.id.to_string(),
            tunnel.target.to_string(),
            format::duration(tunnel.age),
            format::bytes(c.bytes_up),
            format::bytes(c.bytes_down),
            c.dropped.to_string(),
        ])
    });
    Table::new(rows, WIDTHS)
        .header(
            Row::new(["id", "target", "age", "↑", "↓", "dropped"]).style(Style::new().bold().dim()),
        )
        .block(Block::bordered().title(format!(" tunnels ({}) ", app.snapshot.tunnels.len())))
}
