use crate::app::App;
use crate::{input, ui};
use ratatui::DefaultTerminal;
use std::io;

pub fn run(app: &mut App) -> io::Result<()> {
    ratatui::run(|terminal| run_loop(terminal, app))?;
    Ok(())
}

fn run_loop(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_exit() {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let Some(key_event) = input::next_key_event()? {
            input::handle_key_event(app, key_event);
        }
    }

    Ok(())
}
