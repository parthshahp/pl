use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::style::palette::tailwind::GRAY;
use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
};
use ratatui::{DefaultTerminal, Frame};
use serde::Deserialize;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{fs, io};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct UserConfig {
    project_dirs: Vec<String>,
    editor_command: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            project_dirs: vec!["~/Projects".to_string()],
            editor_command: "nvim".to_string(),
        }
    }
}

fn load_user_config() -> io::Result<UserConfig> {
    // Move to config crate??
    let config_dir = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "config directory not found"))?;

    let config_path = config_dir.join("pl").join("config.toml");

    match fs::read_to_string(&config_path) {
        Ok(raw) => toml::from_str(&raw).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid config at {}: {e}", config_path.display()),
            )
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(UserConfig::default()),
        Err(err) => Err(err),
    }
}

#[derive(Debug, Default)]
struct App {
    projects: Vec<Project>,
    filtered_projects: Vec<Project>,
    user_config: UserConfig,
    state: ListState,
    input: InputStruct,
    pending_open: bool,
    exit: bool,
}

#[derive(Default, Debug)]
struct InputStruct {
    input_area: Input,
    input_mode: InputMode,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

#[derive(Debug, Clone)]
struct Project {
    project_name: OsString,
    project_path: PathBuf,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.user_config = load_user_config()?;

        self.input.input_mode = InputMode::Editing;

        // TODO: Remove the clone if possible
        let proj_dirs = self.user_config.project_dirs.clone();
        self.projects = get_all_projects(proj_dirs);
        self.filtered_projects = self.projects.clone();

        self.state.select_first();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        if let Some(selected) = self.state.selected()
            && self.pending_open
        {
            let p = self.projects.get(selected).unwrap();
            let _ = std::process::Command::new(&self.user_config.editor_command)
                .arg(&p.project_path)
                .status();
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
        match self.input.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit(),
                KeyCode::Char('j') | KeyCode::Down => self.state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.state.select_previous(),
                // TODO: Implement 'gg'
                KeyCode::Char('G') => self.state.select_last(),
                KeyCode::Char('/') => self.start_editing(),
                KeyCode::Enter => self.open_project(),
                _ => {}
            },
            InputMode::Editing => match (key_event.code, key_event.modifiers) {
                (KeyCode::Esc, KeyModifiers::NONE) => self.stop_editing(),
                (KeyCode::Enter, KeyModifiers::NONE) => self.stop_editing(),
                (KeyCode::Char('n'), KeyModifiers::CONTROL)
                | (KeyCode::Down, KeyModifiers::NONE) => self.state.select_next(),
                (KeyCode::Char('p'), KeyModifiers::CONTROL) | (KeyCode::Up, KeyModifiers::NONE) => {
                    self.state.select_previous()
                }
                _ => {
                    self.input.input_area.handle_event(&Event::Key(key_event));
                    self.filter_results();
                }
            },
        }
    }

    fn filter_results(&mut self) {
        let q = self.input.input_area.value().to_lowercase();

        if q.is_empty() {
            self.filtered_projects = self.projects.clone();
        } else {
            self.filtered_projects = self
                .projects
                .iter()
                .filter(|p| p.project_name.to_string_lossy().to_lowercase().contains(&q))
                .cloned()
                .collect();
        }

        if self.filtered_projects.is_empty() {
            self.state.select(None);
        } else {
            self.state.select_first();
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn open_project(&mut self) {
        self.pending_open = true;
        self.exit = true;
    }

    // Input Methods
    fn start_editing(&mut self) {
        self.input.input_mode = InputMode::Editing
    }

    fn stop_editing(&mut self) {
        self.input.input_mode = InputMode::Normal
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(area);
        let [input_area, proj_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(left_area);

        self.render_input(input_area, buf);
        self.render_project_list(proj_area, buf);
        self.render_readme(right_area, buf);
    }
}

/// UI Logic
impl App {
    fn render_input(&mut self, area: Rect, buf: &mut Buffer) {
        let style = match self.input.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::new().bg(GRAY.c900),
        };
        let input = Paragraph::new(self.input.input_area.value())
            .style(style)
            .block(Block::bordered().title("Input"));

        Widget::render(input, area, buf);
    }

    fn render_project_list(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .filtered_projects
            .iter()
            .map(|p| ListItem::new(p.project_name.to_string_lossy()))
            .collect();
        let list = List::new(items)
            .block(Block::default().title("Projects").borders(Borders::ALL))
            .highlight_symbol(">")
            .highlight_style(Style::new().bold().bg(GRAY.c900))
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn render_readme(&self, area: Rect, buf: &mut Buffer) {
        // TODO: Either handle the None as 0 or make it so that unselected state is impossible
        let Some(selected_index) = self.state.selected() else {
            return;
        };
        let selected_path = &self
            .filtered_projects
            .get(selected_index)
            .unwrap()
            .project_path;
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

fn get_all_projects(proj_dirs: Vec<String>) -> Vec<Project> {
    // TODO: Learn why we need the double flat_map
    proj_dirs
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
