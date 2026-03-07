use crate::app::{App, InputMode};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph};

const KEYBIND_STYLE: Style = Style::new().bold().blue();

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(frame.area());
    let [input_area, project_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(left_area);

    render_input(frame, app, input_area);
    render_project_list(frame, app, project_area);
    render_readme(frame, app, right_area);

    if app.show_help {
        render_help_popup(frame);
    }

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

    let title = Line::from(vec![
        Span::default().content("Projects ["),
        Span::default().content(format!("{}]", app.sort_label())),
        Span::styled("s", KEYBIND_STYLE),
    ]);

    let title_bottom = Line::from(vec![
        Span::default().content("Keybinds: "),
        Span::styled("?", KEYBIND_STYLE),
    ])
    .centered();

    let widget = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .title_bottom(title_bottom),
        )
        .highlight_symbol("> ")
        .highlight_style(Style::new().bold().cyan())
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(widget, area, &mut app.state);
}

fn render_help_popup(frame: &mut Frame) {
    let help_rect = frame.area().inner(Margin::new(8, 8));
    frame.render_widget(Clear, help_rect);
    frame.render_widget(
        Block::default()
            .title_top("Keybinds")
            .title_bottom(vec![
                Span::styled("q", KEYBIND_STYLE),
                Span::default().content("uit"),
            ])
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL),
        help_rect,
    );
}

fn render_readme(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let contents = app.selected_readme().unwrap_or("No README");
    let widget = Paragraph::new(contents).block(Block::bordered().title("README"));
    frame.render_widget(widget, area);
}
