use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

use super::{App, panels};

pub fn render(frame: &mut Frame, app: &App) {
    let [header, summary, graphs, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(7),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [up, down] = Layout::horizontal([Constraint::Fill(1); 2]).areas(graphs);
    let [tunnels, inspector] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(body);

    frame.render_widget(panels::header(app), header);
    frame.render_widget(panels::summary(app), summary);
    frame.render_widget(panels::throughput("↑ up", &app.up, Color::Green), up);
    frame.render_widget(panels::throughput("↓ down", &app.down, Color::Blue), down);
    frame.render_widget(panels::tunnels(app), tunnels);
    frame.render_widget(panels::inspector(app, inspector.height), inspector);
    frame.render_widget(Paragraph::new(" q quit").style(Style::new().dim()), footer);
}
