use crate::app::{App, InputMode};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use tui_input::backend::crossterm::EventHandler;

pub fn next_key_event() -> io::Result<Option<KeyEvent>> {
    match event::read()? {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => Ok(Some(key_event)),
        _ => Ok(None),
    }
}

pub fn handle_key_event(app: &mut App, key_event: KeyEvent) {
    match app.input_mode {
        InputMode::Normal => match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => app.exit(),
            KeyCode::Char('j') | KeyCode::Down => app.state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => app.state.select_previous(),
            KeyCode::Char('G') => app.state.select_last(),
            KeyCode::Char('/') => app.start_editing(),
            KeyCode::Enter => app.open_selected_project(),
            KeyCode::Char('o') => app.open_project_remote(),
            _ => {}
        },
        InputMode::Editing => match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.exit(),
            (KeyCode::Esc, KeyModifiers::NONE) => app.stop_editing(),
            (KeyCode::Enter, KeyModifiers::NONE) => app.open_selected_project(),
            (KeyCode::Char('n'), KeyModifiers::CONTROL) | (KeyCode::Down, KeyModifiers::NONE) => {
                app.state.select_next()
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) | (KeyCode::Up, KeyModifiers::NONE) => {
                app.state.select_previous()
            }
            _ => {
                app.input.handle_event(&Event::Key(key_event));
                app.filter_results();
            }
        },
    }
}
