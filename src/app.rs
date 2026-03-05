use crate::config::{UserConfig, load_user_config};
use crate::project::{Project, get_all_projects};
use ratatui::widgets::ListState;
use std::io;
use std::path::PathBuf;
use tui_input::Input;

#[derive(Debug)]
pub struct App {
    pub filtered_projects: Vec<Project>,
    pub state: ListState,
    pub input: Input,
    pub input_mode: InputMode,
    user_config: UserConfig,
    projects: Vec<Project>,
    exit: bool,
    open_target: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let user_config = load_user_config()?;
        let projects = get_all_projects(&user_config.project_dirs);
        let filtered_projects = projects.clone();

        let mut state = ListState::default();
        if filtered_projects.is_empty() {
            state.select(None);
        } else {
            state.select_first();
        }

        Ok(Self {
            filtered_projects,
            state,
            input: Input::default(),
            input_mode: InputMode::Editing,
            user_config,
            projects,
            exit: false,
            open_target: None,
        })
    }

    pub fn should_exit(&self) -> bool {
        self.exit
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn start_editing(&mut self) {
        self.input_mode = InputMode::Editing;
    }

    pub fn stop_editing(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn filter_results(&mut self) {
        let query = self.input.value().to_lowercase();

        if query.is_empty() {
            self.filtered_projects = self.projects.clone();
        } else {
            self.filtered_projects = self
                .projects
                .iter()
                .filter(|project| {
                    project
                        .project_name
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&query)
                })
                .cloned()
                .collect();
        }

        if self.filtered_projects.is_empty() {
            self.state.select(None);
        } else {
            self.state.select_first();
        }
    }

    pub fn open_selected_project(&mut self) {
        self.open_target = self
            .selected_project()
            .map(|project| project.project_path.clone());
        self.exit = true;
    }

    pub fn selected_project(&self) -> Option<&Project> {
        self.state
            .selected()
            .and_then(|index| self.filtered_projects.get(index))
    }

    pub fn take_open_target(&mut self) -> Option<PathBuf> {
        self.open_target.take()
    }

    pub fn editor_command(&self) -> &str {
        &self.user_config.editor_command
    }

    pub fn open_project_remote(&self) {
        let Some(project) = self.selected_project() else {
            return;
        };

        let remote = normalize_url(&project.project_remote);
        if remote.is_empty() {
            return;
        }

        if let Err(err) = open::that(&remote) {
            eprintln!("failed to open remote '{remote}': {err}");
        }
    }
}

fn normalize_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.contains("://") {
        return trimmed.to_string();
    }

    format!("https://{trimmed}")
}
