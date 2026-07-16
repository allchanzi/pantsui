use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::{app::App, entities::Panel, features::target::ui as target_ui};

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

    frame.render_widget(
        app.state.version_panel.clone_with_theme(&app.theme),
        ui_layout.version_area,
    );
    render_placeholder_panel(frame, ui_layout.mode_area, Panel::Mode, app);
    target_ui::render(
        frame,
        ui_layout.file_tree_area,
        &app.state.target_panel,
        app.state.active_panel == Panel::FileExplorer,
    );
    render_target_detail(frame, ui_layout.active_area, app);
    render_placeholder_panel(frame, ui_layout.runners_area, Panel::Runs, app);
}

fn render_target_detail(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.state.active_panel == Panel::Main;
    let title = format!("Selected / [{}] {}", Panel::Main.shortcut(), Panel::Main.title());
    let block: Block<'static> = app.theme.panel_block(title, focused);
    let content = app
        .state
        .target_panel
        .current_row()
        .map(|row| {
            if row.has_children {
                vec![
                    Line::from(Span::styled("Branch", app.theme.title())),
                    Line::from(row.label.clone()),
                    Line::from(""),
                    Line::from(Span::styled("Path", app.theme.title())),
                    Line::from(row.path.clone()),
                    Line::from(""),
                    Line::from(Span::styled("Children", app.theme.title())),
                    Line::from(format!(
                        "{} visible targets",
                        app.state
                            .target_panel
                            .rows
                            .iter()
                            .filter(|candidate| candidate.path.starts_with(&row.path) && !candidate.has_children)
                            .count()
                    )),
                    Line::from(""),
                    Line::from(Span::styled("Action", app.theme.title())),
                    Line::from("Press t to run all targets under this branch."),
                    Line::from("Press Space to expand/collapse this branch."),
                ]
            } else {
                let item = app
                    .state
                    .target_panel
                    .items
                    .get(app.state.target_panel.selected);
                match item {
                    Some(item) => vec![
                        Line::from(Span::styled("Target spec", app.theme.title())),
                        Line::from(item.spec.clone()),
                        Line::from(""),
                        Line::from(Span::styled("Status", app.theme.title())),
                        Line::from(format!("{:?}", item.status)),
                        Line::from(""),
                        Line::from(Span::styled("Cache", app.theme.title())),
                        Line::from(format!("{:?}", item.cache_state)),
                        Line::from(""),
                        Line::from(Span::styled("Run", app.theme.title())),
                        Line::from(format!(
                            "{} {:?}",
                            if item.run_state == crate::features::target::model::TargetRunState::Running {
                                app.spinner().to_string()
                            } else {
                                String::from(" ")
                            },
                            item.run_state
                        )),
                        Line::from(""),
                        Line::from(Span::styled("Last exit", app.theme.title())),
                        Line::from(
                            item.last_exit_code
                                .map(|code| code.to_string())
                                .unwrap_or_else(|| String::from("n/a")),
                        ),
                        Line::from(""),
                        Line::from(Span::styled("Last run output", app.theme.title())),
                        Line::from(
                            if app.last_run_output.is_empty() {
                                String::from("n/a")
                            } else {
                                app.last_run_output.clone()
                            },
                        ),
                    ],
                    None => vec![
                        Line::from(Span::styled("No target selected", app.theme.warning())),
                        Line::from(""),
                        Line::from("Load Pants targets to inspect details here."),
                    ],
                }
            }
        })
        .unwrap_or_else(|| {
            vec![
                Line::from(Span::styled("No target selected", app.theme.warning())),
                Line::from(""),
                Line::from("Load Pants targets to inspect details here."),
            ]
        });

    frame.render_widget(Paragraph::new(content).block(block).wrap(Wrap { trim: true }), area);
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
    let widget = if panel == Panel::Runs {
        Paragraph::new(runs_panel_lines(app))
            .block(block)
            .wrap(Wrap { trim: true })
    } else {
        app.state
            .panel_content(panel, &app.theme)
            .block(block)
            .wrap(Wrap { trim: true })
    };
    frame.render_widget(widget, area);
}

fn runs_panel_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled("Available Pants goals/scripts", app.theme.help_title())),
        Line::from(""),
    ];
    lines.extend(
        app.available_goal_lines()
            .into_iter()
            .map(Line::from),
    );
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Live run output", app.theme.title())));
    lines.push(Line::from(
        if app.last_run_output.is_empty() {
            String::from("n/a")
        } else {
            app.last_run_output.clone()
        },
    ));
    lines
}

pub fn footer_lines(app: &App) -> Vec<Line<'static>> {
    let message = app.message.trim();
    let shortcuts = crate::state::AppState::help();
    let running = app.running_count();
    let status = if running > 0 {
        format!("{} running {}", app.spinner(), running)
    } else {
        String::from("idle")
    };

    if message.is_empty() || message == shortcuts {
        return vec![
            Line::from(Span::styled(status, app.theme.kind())),
            Line::from(Span::styled(shortcuts.to_string(), app.theme.muted())),
        ];
    }

    vec![
        Line::from(message.to_string()),
        Line::from(Span::styled(status, app.theme.kind())),
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
        Line::from(vec![Span::styled("t", app.theme.title()), Span::raw(" .......... run selected target or branch")]),
        Line::from(vec![Span::styled("A", app.theme.title()), Span::raw(" .......... run all visible targets")]),
        Line::from(vec![Span::styled("v", app.theme.title()), Span::raw(" .......... cycle target views")]),
        Line::from(vec![Span::styled("Space", app.theme.title()), Span::raw(" .... toggle selected node")]),
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
