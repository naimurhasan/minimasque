use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

fn pressed(key: KeyEvent, code: KeyCode) -> bool {
    key.kind == KeyEventKind::Press && key.code == code
}

pub fn migrates(key: KeyEvent) -> bool {
    pressed(key, KeyCode::Char('m'))
}

pub fn quits(key: KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
        && match key.code {
            KeyCode::Char('q') | KeyCode::Esc => true,
            KeyCode::Char('c') => key.modifiers.contains(KeyModifiers::CONTROL),
            _ => false,
        }
}
