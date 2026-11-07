use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use crate::tui::App;

pub fn header(app: &App) -> Paragraph<'_> {
    Paragraph::new(Line::from(vec![
        " minimasque ".bold().black().on_cyan(),
        format!("  {}  ", app.mode).bold(),
        app.address.as_str().into(),
        "   HTTP/3 · QUIC · CONNECT-UDP".dim(),
    ]))
    .block(Block::bordered())
}
