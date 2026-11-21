//! Space-leader key tree (which-key) and the status line, styled like the Neovim setup.
use ratatui::{Frame, layout::Rect};
use rustline::{Palette, Rustline, Segment, Separators};
use whichkey::{Colors, Entry, Keymap, WhichKey};

use crate::{app::App, shared::ui::UiTheme};

pub fn which_key() -> WhichKey<&'static str> {
    WhichKey::new(Keymap::new(vec![
        Entry::action('?', "Help", "help").also('h'),
        Entry::action('q', "Quit", "quit"),
        Entry::action('f', "Filter targets", "filter"),
        Entry::action('v', "Cycle target view", "view"),
        Entry::action('R', "Refresh targets", "refresh"),
        Entry::action('t', "Tests under selection", "run_tests"),
        Entry::group(
            'r',
            "Run",
            vec![
                Entry::action('t', "Tests under selection", "run_tests"),
                Entry::action('a', "All visible targets", "run_all"),
                Entry::action('g', "Selected goal", "run_goal"),
            ],
        ),
        Entry::group(
            'a',
            "Affected",
            vec![
                Entry::action('l', "Load affected tests", "affected_load"),
                Entry::action('r', "Run affected tests", "affected_run"),
            ],
        ),
    ]))
}

fn colors(theme: &UiTheme) -> Colors {
    Colors {
        base: theme.base,
        border: theme.blue_color(),
        title: theme.lavender,
        key: theme.yellow_color(),
        group: theme.blue_color(),
        text: theme.text_color(),
        subtle: theme.subtle_color(),
    }
}

pub fn draw_which_key(frame: &mut Frame<'_>, app: &App) {
    app.which_key
        .draw(frame, frame.area(), 1, "Space", colors(&app.theme));
}

pub fn draw_status(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let theme = &app.theme;
    let (label, accent) = if app.show_help {
        ("HELP", theme.yellow_color())
    } else if app.has_pending_confirm() {
        ("CONFIRM", theme.red)
    } else if app.output_view.is_some() {
        ("OUTPUT", theme.yellow_color())
    } else if app.filter_mode {
        ("FILTER", theme.yellow_color())
    } else {
        ("NORMAL", theme.blue_color())
    };
    let running = app.running_count();
    let (state, state_color) = if running > 0 {
        (
            format!("{} running {running}", app.spinner()),
            theme.yellow_color(),
        )
    } else {
        ("idle".to_string(), theme.blue_color())
    };
    let message = app.message.trim();
    let (center, center_color) = if app.filter_mode {
        (
            format!("/{}", app.state.target_panel.filter),
            theme.yellow_color(),
        )
    } else if app.which_key.is_open() {
        (app.which_key.summary(), theme.subtle_color())
    } else if app.nav_pending() {
        ("g …".to_string(), theme.yellow_color())
    } else if message.is_empty() || message == crate::state::AppState::help() {
        (
            "[Space] menu • [?] help • [q]uit".to_string(),
            theme.subtle_color(),
        )
    } else {
        (message.to_string(), theme.text_color())
    };
    let mut line = Rustline::new(
        Palette {
            base: theme.base,
            text: theme.text_color(),
        },
        Separators::from_env(),
    )
    .left(
        Segment::new(label, theme.base, accent)
            .required()
            .max_width(12),
    );
    // A confirmation names its target and choices, so it gets the whole bar.
    if !app.has_pending_confirm() {
        line = line
            .left(
                Segment::new(&app.project, theme.base, theme.pink)
                    .max_width(24)
                    .min_width(3),
            )
            .left(
                Segment::new(
                    app.state.active_panel.title(),
                    theme.text_color(),
                    theme.surface,
                )
                .min_bar_width(80),
            );
    }
    line.right(
        Segment::new(state, theme.base, state_color)
            .required()
            .bold(false),
    )
    .center(center, center_color)
    .draw(frame, area);
}
