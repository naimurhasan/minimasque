use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use masque_h3::describe;

use crate::tui::App;

fn styled(line: String) -> Line<'static> {
    let color = match line.chars().next() {
        Some('▶') => Color::Yellow,
        Some('◀') => Color::Green,
        Some('●') => Color::Cyan,
        Some('○') => Color::DarkGray,
        _ => return Line::from(line).dim(),
    };
    Line::from(line).style(Style::new().fg(color))
}

pub fn inspector(app: &App, height: u16) -> Paragraph<'static> {
    let all: Vec<String> = app.log.iter().flat_map(describe::lines).collect();
    let visible = usize::from(height.saturating_sub(2));
    let start = all.len().saturating_sub(visible);
    Paragraph::new(all.into_iter().skip(start).map(styled).collect::<Vec<_>>())
        .block(Block::bordered().title(" protocol inspector "))
}
