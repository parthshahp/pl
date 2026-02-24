use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
};
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
    projects: Vec<Project>,
    state: ListState,
    exit: bool,
}

#[derive(Debug)]
struct Project {
    project_name: OsString,
    project_path: PathBuf,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.projects = get_all_projects();
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
            KeyCode::Char('j') => self.state.select_next(),
            KeyCode::Char('k') => self.state.select_previous(),
            KeyCode::Char('G') => self.state.select_last(),
            // TODO: Implement 'gg'
            KeyCode::Enter => self.select_project(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_project(&mut self) {
        if let Some(selected) = self.state.selected() {
            let p = self.projects.get(selected).unwrap();
            println!(
                "Project {:?} at {:?} selected",
                p.project_name, p.project_path
            )
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(area);
        self.render_project_list(left_area, buf);
        self.render_readme(right_area, buf);
    }
}

/// UI Logic
impl App {
    fn render_project_list(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .projects
            .iter()
            .map(|p| ListItem::new(p.project_name.to_string_lossy()))
            .collect();
        let list = List::new(items)
            .block(Block::default().title("Projects").borders(Borders::ALL))
            .highlight_symbol(">")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn render_readme(&self, area: Rect, buf: &mut Buffer) {
        // TODO: Either handle the None as 0 or make it so that unselected state is impossible
        let selected_index = self.state.selected().unwrap();
        let selected_path = &self.projects.get(selected_index).unwrap().project_path;
        let readme_path = selected_path.join("README.md");

        if let Ok(contents) = std::fs::read_to_string(&readme_path) {
            Paragraph::new(contents)
                .block(Block::bordered())
                .render(area, buf);
        } else {
            Paragraph::new("No README")
                .centered()
                .block(Block::bordered())
                .render(area, buf);
        }
    }
}

fn get_all_projects() -> Vec<Project> {
    // TODO: Learn why we need the double flat_map
    PROJ_DIRS
        .iter()
        .map(|d| parse_dir(d))
        .flat_map(|dir| {
            fs::read_dir(&dir)
                .ok()
                .into_iter()
                .flat_map(|rd| rd.filter_map(Result::ok))
        })
        .filter(|entry| entry.path().join(".git").exists())
        .map(|entry| Project {
            project_name: entry.file_name(),
            project_path: entry.path(),
        })
        .collect()
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
