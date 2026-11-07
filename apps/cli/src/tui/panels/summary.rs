use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::format;
use crate::tui::App;

fn stat<'a>(label: &'a str, value: String) -> [Span<'a>; 3] {
    [label.dim(), " ".into(), value.bold()]
}

pub fn summary(app: &App) -> Paragraph<'_> {
    let totals = &app.snapshot.totals;
    let spans: Vec<Span> = std::iter::once(Span::raw(" "))
        .chain(
            [
                stat("open", app.snapshot.tunnels.len().to_string()),
                stat("opened", app.snapshot.opened.to_string()),
                stat("↑", format::bytes(totals.bytes_up)),
                stat("↓", format::bytes(totals.bytes_down)),
                stat(
                    "packets",
                    (totals.packets_up + totals.packets_down).to_string(),
                ),
                stat("dropped", totals.dropped.to_string()),
            ]
            .into_iter()
            .flat_map(|parts| parts.into_iter().chain([Span::raw("    ")])),
        )
        .collect();
    Paragraph::new(Line::from(spans)).block(Block::bordered().title(" totals "))
}
