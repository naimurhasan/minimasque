use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Sparkline};

use crate::format;
use crate::tui::History;

pub fn throughput<'a>(label: &str, history: &'a History, color: Color) -> Sparkline<'a> {
    Sparkline::default()
        .block(Block::bordered().title(format!(" {label} {} ", format::rate(history.latest()))))
        .data(history.samples().collect::<Vec<_>>())
        .style(Style::new().fg(color))
}
