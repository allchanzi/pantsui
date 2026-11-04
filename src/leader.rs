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
        Entry::group(
            'p',
            "Panel",
            vec![
                Entry::action('1', "Version", "panel1"),
                Entry::action('2', "Goals", "panel2"),
                Entry::action('3', "Source tree", "panel3"),
                Entry::action('4', "Active area", "panel4"),
                Entry::action('5', "Runs", "panel5"),
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
    let (label, accent) = if app.has_pending_goal() {
        ("CONFIRM", theme.red)
    } else if app.filter_mode {
        ("FILTER", theme.yellow_color())
    } else {
        ("NORMAL", theme.blue_color())
    };
    let running = app.running_count();
    let (state, state_color) = if running > 0 {
        (format!("{} running {running}", app.spinner()), theme.yellow_color())
    } else {
        ("idle".to_string(), theme.blue_color())
    };
    let message = app.message.trim();
    let (center, center_color) = if app.filter_mode {
        (format!("/{}", app.state.target_panel.filter), theme.yellow_color())
    } else if app.which_key.is_open() {
        (app.which_key.summary(), theme.subtle_color())
    } else if message.is_empty() || message == crate::state::AppState::help() {
        ("Space menu • ? help • q quit".to_string(), theme.subtle_color())
    } else {
        (message.to_string(), theme.text_color())
    };
    Rustline::new(
        Palette {
            base: theme.base,
            text: theme.text_color(),
        },
        Separators::Powerline,
    )
    .left(Segment::new(label, theme.base, accent).required().max_width(12))
    .left(
        Segment::new(&app.project, theme.base, theme.pink)
            .max_width(24)
            .min_width(3),
    )
    .left(
        Segment::new(app.state.active_panel.title(), theme.text_color(), theme.surface)
            .min_bar_width(80),
    )
    .right(Segment::new(state, theme.base, state_color).required().bold(false))
    .center(center, center_color)
    .draw(frame, area);
}
