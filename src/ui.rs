use crate::app::{App, InputMode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(frame.area());
    let [input_area, project_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(left_area);

    render_input(frame, app, input_area);
    render_project_list(frame, app, project_area);
    render_readme(frame, app, right_area);

    if app.input_mode == InputMode::Editing {
        let cursor_x = input_area.x + 1 + app.input.visual_cursor() as u16;
        let cursor_y = input_area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let style = match app.input_mode {
        InputMode::Normal => Style::default(),
        InputMode::Editing => Style::new().cyan(),
    };

    let widget = Paragraph::new(app.input.value())
        .style(style)
        .block(Block::bordered().title("Search"));

    frame.render_widget(widget, area);
}

fn render_project_list(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .filtered_projects
        .iter()
        .map(|project| ListItem::new(project.project_name.to_string_lossy()))
        .collect();

    let widget = List::new(items)
        .block(Block::default().title("Projects").borders(Borders::ALL))
        .highlight_symbol("> ")
        .highlight_style(Style::new().bold().cyan())
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(widget, area, &mut app.state);
}

fn render_readme(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let Some(project) = app.selected_project() else {
        return;
    };

    let readme_path = project.project_path.join("README.md");

    let widget = if let Ok(contents) = std::fs::read_to_string(&readme_path) {
        Paragraph::new(contents).block(Block::bordered().title("README"))
    } else {
        Paragraph::new("No README")
            .centered()
            .block(Block::bordered().title("README"))
    };

    frame.render_widget(widget, area);
}
