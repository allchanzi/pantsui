use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{
    app::{App, OutputView},
    entities::Panel,
    features::target::ui as target_ui,
};

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

    let version = app.state.version_panel.clone_with_theme(&app.theme);
    let (version_max, version_rows) = version.scroll_range(ui_layout.version_area);
    app.version_max.set(version_max);
    if app.state.active_panel == Panel::Version {
        app.panel_rows.set(version_rows);
    }
    frame.render_widget(version, ui_layout.version_area);
    if app.state.active_panel == Panel::FileExplorer {
        app.panel_rows
            .set(ui_layout.file_tree_area.height.saturating_sub(2) as usize);
    }
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
    if focused {
        app.panel_rows.set(area.height.saturating_sub(2) as usize);
    }
    let title = format!(
        "[{}] Selected / {}",
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
                        "press [a] to find them",
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
                Line::from("[r]un scoped • [a]ffected • [T] run affected"),
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

    let paragraph = Paragraph::new(content).wrap(Wrap { trim: true });
    let max = paragraph
        .line_count(area.width.saturating_sub(2))
        .saturating_sub(area.height.saturating_sub(2) as usize);
    app.detail_max.set(max);
    let scroll = usize::from(app.detail_scroll).min(max) as u16;
    frame.render_widget(paragraph.block(block).scroll((scroll, 0)), area);
}

fn render_placeholder_panel(frame: &mut Frame<'_>, area: Rect, panel: Panel, app: &App) {
    let focused = app.state.active_panel == panel;
    if focused {
        app.panel_rows.set(area.height.saturating_sub(2) as usize);
    }
    let title = match panel {
        Panel::Mode => format!("[{}] Global / {}", panel.shortcut(), panel.title()),
        Panel::FileExplorer => format!("[{}] Global / {}", panel.shortcut(), panel.title()),
        Panel::Main => format!("[{}] Selected / {}", panel.shortcut(), panel.title()),
        Panel::Runs => format!("[{}] Selected / {}", panel.shortcut(), panel.title()),
        Panel::Version => format!("[{}] {}", panel.shortcut(), panel.title()),
    };
    let block: Block<'static> = app.theme.panel_block(title, focused);
    let widget = if panel == Panel::Runs {
        Paragraph::new(runs_panel_lines(
            app,
            area.height.saturating_sub(2) as usize,
        ))
        .block(block)
        .wrap(Wrap { trim: true })
    } else if panel == Panel::Mode {
        Paragraph::new(goal_panel_lines(
            app,
            area.height.saturating_sub(2) as usize,
        ))
        .block(block)
        .wrap(Wrap { trim: true })
    } else {
        Paragraph::new("").block(block)
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
        "[j/k] select • [Enter] run",
        app.theme.muted(),
    )));
    lines
}

const RUN_PREVIEW_LINES: usize = 8;

fn runs_panel_lines(app: &App, height: usize) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(Span::styled(
        format!(
            "Run history ({}) • [j/k] select • [Enter/o] output",
            app.run_history.len()
        ),
        app.theme.help_title(),
    ))];
    let Some(selected) = app.selected_run_index() else {
        lines.push(Line::from(Span::styled("No runs yet", app.theme.muted())));
        return lines;
    };

    // Newest first; keep the selection high enough for its preview to fit.
    let position = app.run_history.len() - 1 - selected;
    let rows = height.saturating_sub(1);
    let start = position.saturating_sub(rows.saturating_sub(RUN_PREVIEW_LINES + 1));
    for (index, run) in app.run_history.iter().enumerate().rev().skip(start) {
        let (badge, style) = run_badge(run.status, app);
        let is_selected = index == selected;
        lines.push(Line::from(vec![
            Span::styled(if is_selected { "▶ " } else { "  " }, app.theme.value()),
            Span::styled(format!("{badge} "), style),
            Span::raw(run.spec.clone()),
        ]));
        if is_selected {
            // The end of the output is where a failure is reported.
            let output = run.output.lines().collect::<Vec<_>>();
            let tail = output.len().saturating_sub(RUN_PREVIEW_LINES);
            lines.extend(
                output[tail..]
                    .iter()
                    .map(|line| Line::from(Span::styled(format!("    {line}"), app.theme.muted()))),
            );
        }
    }
    lines
}

fn run_badge(
    status: crate::features::run::model::RunStatus,
    app: &App,
) -> (&'static str, ratatui::style::Style) {
    use crate::features::run::model::RunStatus;

    match status {
        RunStatus::Running => ("…", app.theme.warning()),
        RunStatus::Passed => ("✓", app.theme.value()),
        RunStatus::Failed => ("✗", app.theme.warning()),
    }
}

fn wrap_output(output: &str, width: usize) -> Vec<String> {
    let mut wrapped = Vec::new();
    for line in output.lines() {
        let characters = line.chars().collect::<Vec<_>>();
        if characters.is_empty() {
            wrapped.push(String::new());
        }
        wrapped.extend(
            characters
                .chunks(width.max(1))
                .map(|chunk| chunk.iter().collect::<String>()),
        );
    }
    wrapped
}

pub fn render_output(frame: &mut Frame<'_>, area: Rect, app: &App, view: &OutputView) {
    use crate::features::run::model::RunStatus;

    let Some(run) = app.run_history.get(view.run) else {
        return;
    };
    let (badge, _) = run_badge(run.status, app);
    let block: Block<'static> = app.theme.panel_block(format!("{badge} {}", run.spec), true);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    let [body, hint] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner);

    let lines = wrap_output(&run.output, body.width as usize);
    let height = body.height as usize;
    let max_scroll = lines.len().saturating_sub(height);
    view.max_scroll.set(max_scroll);
    view.page.set(height.max(1));
    let scroll = view.scroll.min(max_scroll);
    let content = if lines.is_empty() {
        vec![Line::from(Span::styled(
            if run.status == RunStatus::Running {
                "Still running, no output yet."
            } else {
                "No output was captured."
            },
            app.theme.muted(),
        ))]
    } else {
        lines
            .iter()
            .skip(scroll)
            .take(height)
            .map(|line| Line::from(line.clone()))
            .collect()
    };
    frame.render_widget(Paragraph::new(content), body);
    frame.render_widget(
        Paragraph::new(format!(
            "[j/k] scroll • [gg/G] ends • [Esc] close • {}/{}",
            (scroll + height).min(lines.len()),
            lines.len()
        ))
        .style(app.theme.muted()),
        hint,
    );
}

pub fn help_lines(app: &App) -> Vec<Line<'static>> {
    let row = |keys: &'static str, text: &'static str| {
        Line::from(vec![
            Span::styled(format!("{keys:<16}"), app.theme.title()),
            Span::raw(text),
        ])
    };
    let panel = app.state.active_panel;
    let mut lines = vec![Line::from(Span::styled(
        format!("{} shortcuts", panel.title()),
        app.theme.help_title(),
    ))];
    lines.extend(match panel {
        Panel::FileExplorer => vec![
            row("[j/k]", "move"),
            row("[h/l]", "collapse or parent / expand"),
            row("[Enter]", "toggle folder"),
            row("[r]un", "tests under the selection (also [t])"),
            row("[o]utput", "last output of the selected target"),
            row("[/]", "filter, smartcase; [Enter] applies, [Esc] cancels"),
            row("[v]iew", "cycle target views"),
        ],
        Panel::Mode => vec![
            row("[j/k]", "select goal"),
            row("[Enter]", "run the selected goal; mutating goals ask first"),
        ],
        Panel::Main => vec![row("[j/k]", "scroll details")],
        Panel::Runs => vec![
            row("[j/k]", "select run, newest first"),
            row("[Enter/o]", "full output of the selected run"),
        ],
        Panel::Version => vec![
            row("[h/l]", "previous / next section"),
            row("[j/k]", "scroll the section"),
            row("Dependencies", "of the selection in [3]; [R] reloads"),
            row("Sub-projects", "folders holding the targets, with counts"),
            row("[Ctrl+l]", "leave for the panel on the right"),
        ],
    });
    lines.extend([
        Line::from(""),
        Line::from(Span::styled("Everywhere", app.theme.help_title())),
        row("[Tab/S-Tab]", "next / previous panel"),
        row("[1-5]", "jump to a panel"),
        row("[Ctrl+h/j/k/l]", "focus the panel in that direction"),
        row("[gg/G]", "first / last"),
        row("[Ctrl+u/d]", "half page; [Ctrl+b/f] full page"),
        row("[a]ffected", "find affected tests; [T] runs them"),
        row("[A]", "run all visible tests"),
        row("[R]", "refresh Pants targets"),
        row("[Space]", "action menu"),
        row("[?]", "this help; [Esc] closes"),
        row("[Ctrl+c]", "cancel what is open or pending"),
        row("[q]uit", "asks while runs are active"),
    ]);
    lines
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_output_lines_wrap_and_blank_lines_survive() {
        assert_eq!(wrap_output("abcde\n\nf", 2), ["ab", "cd", "e", "", "f"]);
        assert_eq!(wrap_output("ab", 0), ["a", "b"]);
    }
}
