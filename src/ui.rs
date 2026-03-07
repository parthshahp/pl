use crate::app::{App, InputMode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph,
};

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

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(key, KEYBIND_STYLE),
        Span::raw(format!("  {desc}")),
    ])
}

fn centered_rect(area: Rect, max_width: u16, max_height: u16) -> Rect {
    let width = max_width.min(area.width);
    let height = max_height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect { x, y, width, height }
}

fn render_help_popup(frame: &mut Frame) {
    let help_rect = centered_rect(frame.area(), 50, 20);
    frame.render_widget(Clear, help_rect);

    let lines = vec![
        help_line("j / ↓", "move down"),
        help_line("k / ↑", "move up"),
        help_line("g", "go to first"),
        help_line("G", "go to last"),
        help_line("/", "search"),
        help_line("Enter", "open project"),
        help_line("o", "open remote in browser"),
        help_line("s", "cycle sort (A-Z / Recent)"),
        help_line("?", "toggle this help"),
        help_line("q / Esc", "quit"),
    ];

    let widget = Paragraph::new(lines).block(
        Block::default()
            .title(Line::raw("Keybinds").centered())
            .borders(Borders::ALL)
            .padding(Padding::new(2, 2, 1, 1)),
    );

    frame.render_widget(widget, help_rect);
}

fn render_readme(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let contents = app.selected_readme().unwrap_or("No README");
    let widget = Paragraph::new(contents).block(Block::bordered().title("README"));
    frame.render_widget(widget, area);
}
