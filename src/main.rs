mod app;
mod config;
mod input;
mod project;
mod tui;
mod ui;

use crate::app::App;
use std::io;

fn main() -> io::Result<()> {
    let mut app = App::new()?;
    let editor_command = app.editor_command().to_string();

    tui::run(&mut app)?;

    if let Some(path) = app.take_open_target() {
        let _ = std::process::Command::new(&editor_command).arg(path).status();
    }

    Ok(())
}
