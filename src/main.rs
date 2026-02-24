use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget};
use ratatui::{DefaultTerminal, Frame};
use std::ffi::OsString;
use std::path::PathBuf;
use std::{fs, io};

const PROJ_DIRS: &[&str; 1] = &["~/Projects"];

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))?;
    Ok(())
}

#[derive(Debug, Default)]
struct App {
    projects: Vec<OsString>,
    state: ListState,
    exit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.projects = get_all_projects()?;
        self.state.select_first();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('k') => self.select_prev(),
            KeyCode::Enter => self.select_project(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }

    fn select_prev(&mut self) {
        self.state.select_previous();
    }

    fn select_project(&mut self) {
        if let Some(selected) = self.state.selected() {
            println!("Project {} selected", selected)
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .projects
            .iter()
            .map(|p| ListItem::new(p.to_string_lossy()))
            .collect();
        let list = List::new(items)
            .block(Block::default().title("Projects").borders(Borders::ALL))
            .highlight_symbol(">")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

fn get_all_projects() -> Result<Vec<OsString>, io::Error> {
    let mut all_projects = vec![];
    for proj_dir in PROJ_DIRS {
        let proj_dir = parse_dir(proj_dir);

        let projects = fs::read_dir(proj_dir)?;
        for proj in projects {
            let proj = proj?;
            let proj_git_folder = proj.path().join(".git");
            if proj_git_folder.exists() {
                all_projects.push(proj.file_name());
            }
        }
    }
    Ok(all_projects)
}

fn parse_dir(proj_dir: &str) -> PathBuf {
    if proj_dir == "~" {
        return home_dir();
    }

    if let Some(stripped) = proj_dir.strip_prefix("~/") {
        return home_dir().join(stripped);
    }

    PathBuf::from(proj_dir)
}

fn home_dir() -> PathBuf {
    dirs::home_dir().expect("could not determine home directory")
}
