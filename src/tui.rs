use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};

use crate::{
    app::{App, Effect},
    error::{AppErr, AppResult},
    ui,
};

type Backend = CrosstermBackend<Stdout>;
type AppTerminal = Terminal<Backend>;

pub fn run(app: App) -> AppResult<()> {
    let mut terminal = init_terminal()?;
    let result = event_loop(&mut terminal, app);
    restore_terminal(&mut terminal)?;
    result
}

fn init_terminal() -> AppResult<AppTerminal> {
    enable_raw_mode().map_err(|_| AppErr::Terminal)?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    Terminal::new(CrosstermBackend::new(stdout)).map_err(AppErr::Io)
}

fn restore_terminal(terminal: &mut AppTerminal) -> AppResult<()> {
    disable_raw_mode().map_err(|_| AppErr::Terminal)?;
    execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor().map_err(AppErr::Io)
}

fn event_loop(terminal: &mut AppTerminal, mut app: App) -> AppResult<()> {
    let mut selection = tuikit::TextSelection::default();
    loop {
        app.drain_events();
        app.load_open_section();
        app.frame_tick();
        terminal
            .draw(|frame| {
                render(frame, &app);
                selection.capture(frame.buffer_mut(), app.text_area.get());
                selection.paint(
                    frame,
                    ratatui::style::Style::new().add_modifier(ratatui::style::Modifier::REVERSED),
                );
            })
            .map_err(AppErr::Io)?;
        if let Some(effect) = next_effect(&mut app, &mut selection)? {
            match effect {
                Effect::Quit => return Ok(()),
                Effect::None | Effect::Message(_) | Effect::ToggleHelp => {}
                Effect::RunInteractive { goal, specs } => {
                    restore_terminal(terminal)?;
                    let success = app.execute_interactive_goal(&goal, &specs);
                    *terminal = init_terminal()?;
                    app.finish_interactive_goal(&goal, &specs, success);
                }
            }
        }
    }
}

fn next_effect(app: &mut App, selection: &mut tuikit::TextSelection) -> AppResult<Option<Effect>> {
    if !event::poll(Duration::from_millis(250))? {
        return Ok(None);
    }

    let event = event::read()?;
    if selection.event(&event) {
        return Ok(None);
    }
    let Event::Key(key) = event else {
        return Ok(None);
    };

    // Nothing is visible below the minimum size, so only quitting is safe.
    let (width, height) = crossterm::terminal::size().map_err(AppErr::Io)?;
    if !tuikit::fits(width, height) && key.code != crossterm::event::KeyCode::Char('q') {
        return Ok(None);
    }
    (key.kind == KeyEventKind::Press)
        .then(|| app.handle_key(key))
        .transpose()
}

fn render(frame: &mut ratatui::Frame<'_>, app: &App) {
    app.text_area.set(ratatui::layout::Rect::default());
    if tuikit::too_small(frame, frame.area()) {
        return;
    }
    let [main, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .areas(frame.area());

    ui::render(frame, main, app);
    crate::leader::draw_status(frame, footer, app);
    crate::leader::draw_which_key(frame, app);
    if app.which_key.is_open() {
        app.text_area.set(ratatui::layout::Rect::default());
    }

    if app.show_help {
        app.help_popup.render(
            frame,
            main,
            " [?] Help ",
            ui::help_lines(app),
            app.theme.focused_border(),
        );
    }
    if app.show_help {
        app.text_area.set(app.help_popup.text_area.get());
    }
    if let Some(view) = &app.output_view {
        ui::render_output(frame, ui::centered_rect(92, 92, frame.area()), app, view);
    }
}
