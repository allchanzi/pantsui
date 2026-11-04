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

/// Two columns when wide, one stacked column (focused panel enlarged) when narrow.
fn main_layout(area: Rect, active: Panel) -> UiLayout {
    let focus = match active {
        Panel::Version => 0,
        Panel::Mode => 1,
        Panel::FileExplorer => 2,
        Panel::Main => 3,
        Panel::Runs => 4,
    };
    let rects = panels::Flow::new([vec![22, 18, 60], vec![60, 40]])
        .wide_from(100)
        .layout(area, Some(focus));
    UiLayout {
        version_area: rects[0],
        mode_area: rects[1],
        file_tree_area: rects[2],
        active_area: rects[3],
        runners_area: rects[4],
    }
}

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let ui_layout = main_layout(area, app.state.active_panel);

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
        &app.theme,
    );
    render_target_detail(frame, ui_layout.active_area, app);
    render_placeholder_panel(frame, ui_layout.runners_area, Panel::Runs, app);
}

fn render_target_detail(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.state.active_panel == Panel::Main;
    let title = format!(
        "Selected / [{}] {}",
        Panel::Main.shortcut(),
        Panel::Main.title()
    );
    let block: Block<'static> = app.theme.panel_block(title, focused);
    let content = app
        .state
        .target_panel
        .current_row()
        .map(|row| {
            let targets = app.state.target_panel.scoped_targets();
            let tests = app.state.target_panel.scoped_tests();
            let mut lines = vec![
                Line::from(Span::styled(
                    if row.spec.is_some() {
                        "File / target"
                    } else {
                        "Directory"
                    },
                    app.theme.title(),
                )),
                Line::from(row.path.clone()),
                Line::from(""),
            ];
            if let Some(spec) = &row.spec
                && let Some(target) = app
                    .state
                    .target_panel
                    .items
                    .iter()
                    .find(|target| target.spec == *spec)
            {
                lines.extend([
                    Line::from(vec![
                        Span::styled("Status: ", app.theme.title()),
                        Span::raw(format!("{:?}", target.cache_state)),
                    ]),
                    Line::from(vec![
                        Span::styled("Run: ", app.theme.title()),
                        Span::raw(format!("{:?}", target.run_state)),
                    ]),
                    Line::from(""),
                ]);
            }
            lines.push(Line::from(Span::styled(
                format!("Tests under selection ({})", tests.len()),
                app.theme.title(),
            )));
            if tests.is_empty() {
                lines.push(Line::from(Span::styled("none", app.theme.muted())));
            } else {
                lines.extend(tests.into_iter().map(|test| {
                    Line::from(vec![
                        Span::styled("T ", app.theme.kind()),
                        Span::raw(test.spec.clone()),
                    ])
                }));
            }
            lines.extend([
                Line::from(""),
                Line::from(Span::styled(
                    if app.affected_loading {
                        String::from("Affected tests (loading…)")
                    } else {
                        format!("Affected tests ({})", app.affected_tests.len())
                    },
                    app.theme.title(),
                )),
            ]);
            if let Some(scope) = &app.affected_scope {
                lines.push(Line::from(Span::styled(
                    format!("scope: {scope}"),
                    app.theme.muted(),
                )));
            }
            if !app.affected_loading {
                if app.affected_tests.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "press d to calculate",
                        app.theme.muted(),
                    )));
                } else {
                    lines.extend(app.affected_tests.iter().map(|test| {
                        Line::from(vec![
                            Span::styled("A ", app.theme.warning()),
                            Span::raw(test.spec.clone()),
                        ])
                    }));
                }
            }
            lines.extend([
                Line::from(""),
                Line::from(Span::styled(
                    format!("Targets for selection ({})", targets.len()),
                    app.theme.title(),
                )),
            ]);
            if targets.is_empty() {
                lines.push(Line::from(Span::styled("none", app.theme.muted())));
            } else {
                lines.extend(targets.into_iter().map(|target| {
                    Line::from(vec![
                        Span::styled(if target.is_test { "T " } else { "• " }, app.theme.kind()),
                        Span::raw(target.spec.clone()),
                    ])
                }));
            }
            lines.extend([
                Line::from(""),
                Line::from(Span::styled("Actions", app.theme.title())),
                Line::from("t run scoped • d find affected • T run affected"),
            ]);
            lines
        })
        .unwrap_or_else(|| {
            vec![
                Line::from(Span::styled("No target selected", app.theme.warning())),
                Line::from(""),
                Line::from("Load Pants targets to inspect details here."),
            ]
        });

    frame.render_widget(
        Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((app.detail_scroll, 0)),
        area,
    );
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
            .scroll((app.runs_scroll, 0))
    } else if panel == Panel::Mode {
        Paragraph::new(goal_panel_lines(
            app,
            area.height.saturating_sub(2) as usize,
        ))
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

fn goal_panel_lines(app: &App, height: usize) -> Vec<Line<'static>> {
    use crate::features::goal::model::GoalKind;

    if app.goals().is_empty() {
        return vec![Line::from(Span::styled(
            "No Pants goals discovered",
            app.theme.warning(),
        ))];
    }

    let visible_goals = height.saturating_sub(2).max(1);
    let start = app.goal_selected.saturating_sub(visible_goals / 2);
    let mut lines = app
        .goals()
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_goals)
        .map(|(index, goal)| {
            let kind = GoalKind::for_name(&goal.name);
            Line::from(vec![
                Span::styled(
                    if index == app.goal_selected {
                        "▶ "
                    } else {
                        "  "
                    },
                    if index == app.goal_selected {
                        app.theme.value()
                    } else {
                        app.theme.muted()
                    },
                ),
                Span::styled(goal.name.clone(), app.theme.title()),
                Span::styled(format!(" [{}]", kind.label()), app.theme.muted()),
            ])
        })
        .collect::<Vec<_>>();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "j/k select • g/Enter run",
        app.theme.muted(),
    )));
    lines
}

fn runs_panel_lines(app: &App) -> Vec<Line<'static>> {
    use crate::features::run::model::RunStatus;

    let mut lines = vec![Line::from(Span::styled(
        format!("Run history ({})", app.run_history.len()),
        app.theme.help_title(),
    ))];
    if app.run_history.is_empty() {
        lines.push(Line::from(Span::styled("No runs yet", app.theme.muted())));
        return lines;
    }

    for run in app.run_history.iter().rev() {
        let (badge, style) = match run.status {
            RunStatus::Running => ("…", app.theme.warning()),
            RunStatus::Passed => ("✓", app.theme.value()),
            RunStatus::Failed => ("✗", app.theme.warning()),
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{badge} "), style),
            Span::raw(run.spec.clone()),
        ]));
        if !run.output.is_empty() {
            lines.extend(
                run.output
                    .lines()
                    .take(8)
                    .map(|line| Line::from(Span::styled(format!("  {line}"), app.theme.muted()))),
            );
        }
    }
    lines
}

pub fn help_lines(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled("pantsui shell help", app.theme.help_title())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", app.theme.title()),
            Span::raw(" ........ next panel"),
        ]),
        Line::from(vec![
            Span::styled("Shift+Tab", app.theme.title()),
            Span::raw(" .. previous panel"),
        ]),
        Line::from(vec![
            Span::styled("h/j/k/l", app.theme.title()),
            Span::raw(" .... move focus by direction"),
        ]),
        Line::from(vec![
            Span::styled("Alt/Ctrl+h/j/k/l", app.theme.title()),
            Span::raw("  move focus by direction"),
        ]),
        Line::from(vec![
            Span::styled("1..5", app.theme.title()),
            Span::raw(" ....... jump directly to a panel"),
        ]),
        Line::from(vec![
            Span::styled("?", app.theme.title()),
            Span::raw(" .......... toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("q / Esc", app.theme.title()),
            Span::raw(" .... quit"),
        ]),
        Line::from(vec![
            Span::styled("t", app.theme.title()),
            Span::raw(" .......... run selected target or branch"),
        ]),
        Line::from(vec![
            Span::styled("A", app.theme.title()),
            Span::raw(" .......... run all visible targets"),
        ]),
        Line::from(vec![
            Span::styled("v", app.theme.title()),
            Span::raw(" .......... cycle target views"),
        ]),
        Line::from(vec![
            Span::styled("Space", app.theme.title()),
            Span::raw(" .... leader menu (which-key): r run, a affected, p panel, f filter"),
        ]),
        Line::from(vec![
            Span::styled("R", app.theme.title()),
            Span::raw(" .......... refresh Pants targets"),
        ]),
        Line::from(vec![
            Span::styled("/", app.theme.title()),
            Span::raw(" .......... filter target tree"),
        ]),
        Line::from(vec![
            Span::styled("d / T", app.theme.title()),
            Span::raw(" ..... find / run affected tests"),
        ]),
        Line::from(vec![
            Span::styled("g / Enter", app.theme.title()),
            Span::raw("  run selected Pants goal"),
        ]),
        Line::from(vec![
            Span::styled("j / k", app.theme.title()),
            Span::raw(" ...... navigate or scroll focused panel"),
        ]),
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
