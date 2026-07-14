use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Wrap},
};

use crate::{app::App, entities::Panel};

struct UiLayout {
    version_area: Rect,
    active_area: Rect,
    file_tree_area: Rect,
    runners_area: Rect,
    mode_area: Rect,
}

fn main_layout(area: Rect) -> UiLayout {
    let [main_left, main_right] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(area);

    let [version_area, mode_area, file_tree_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(22),
            Constraint::Percentage(18),
            Constraint::Percentage(60),
        ])
        .areas(main_left);
    let [active_area, runners_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .areas(main_right);

    UiLayout {
        version_area,
        active_area,
        file_tree_area,
        runners_area,
        mode_area,
    }
}

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let ui_layout = main_layout(area);

    frame.render_widget(app.state.version_panel.clone_with_theme(&app.theme), ui_layout.version_area);
    render_placeholder_panel(frame, ui_layout.mode_area, Panel::Mode, app);
    render_placeholder_panel(frame, ui_layout.file_tree_area, Panel::FileExplorer, app);
    render_placeholder_panel(frame, ui_layout.active_area, Panel::Main, app);
    render_placeholder_panel(frame, ui_layout.runners_area, Panel::Runs, app);
}

fn render_placeholder_panel(frame: &mut Frame<'_>, area: Rect, panel: Panel, app: &App) {
    let focused = app.state.active_panel == panel;
    let title = match panel {
        Panel::Mode => format!("Global / [{}] {}", panel.shortcut(), panel.title()),
        Panel::FileExplorer => format!("Global / [{}] {}", panel.shortcut(), panel.title()),
        Panel::Main => format!("Selected / [{}] {}", panel.shortcut(), panel.title()),
        Panel::Runs => format!("Selected / [{}] {}", panel.shortcut(), panel.title()),
        Panel::Version => format!("[{}] {}", panel.shortcut(), panel.title()),
    };
    let block: Block<'static> = app.theme.panel_block(title, focused);
    let widget = app
        .state
        .panel_content(panel, &app.theme)
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

pub fn footer_lines(app: &App) -> Vec<Line<'static>> {
    let message = app.message.trim();
    let shortcuts = crate::state::AppState::help();

    if message.is_empty() || message == shortcuts {
        return vec![Line::from(Span::styled(
            shortcuts.to_string(),
            app.theme.muted(),
        ))];
    }

    vec![
        Line::from(message.to_string()),
        Line::from(Span::styled(shortcuts, app.theme.muted())),
    ]
}

pub fn help_lines(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled("pantsui shell help", app.theme.help_title())),
        Line::from(""),
        Line::from(vec![Span::styled("Tab", app.theme.title()), Span::raw(" ........ cycle focus")]),
        Line::from(vec![Span::styled("h/j/k/l", app.theme.title()), Span::raw(" .... move focus by direction")]),
        Line::from(vec![
            Span::styled("Alt+h/j/k/l", app.theme.title()),
            Span::raw("  move focus by direction"),
        ]),
        Line::from(vec![Span::styled("1..5", app.theme.title()), Span::raw(" ....... jump directly to a panel")]),
        Line::from(vec![Span::styled("?", app.theme.title()), Span::raw(" .......... toggle this help")]),
        Line::from(vec![Span::styled("q / Esc", app.theme.title()), Span::raw(" .... quit")]),
        Line::from(""),
        Line::from("Current implementation mirrors PDE-style shell structure:"),
        Line::from(vec![
            Span::styled("app.rs", app.theme.kind()),
            Span::raw(" = state/effects, "),
            Span::styled("tui.rs", app.theme.kind()),
            Span::raw(" = runtime, "),
            Span::styled("ui.rs", app.theme.kind()),
            Span::raw(" = layout composition."),
        ]),
    ]
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, vertical, _] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .areas(area);

    let [_, horizontal, _] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .areas(vertical);

    horizontal
}
