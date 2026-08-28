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
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout)).map_err(AppErr::Io)
}

fn restore_terminal(terminal: &mut AppTerminal) -> AppResult<()> {
    disable_raw_mode().map_err(|_| AppErr::Terminal)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor().map_err(AppErr::Io)
}

fn event_loop(terminal: &mut AppTerminal, mut app: App) -> AppResult<()> {
    loop {
        app.drain_events();
        app.frame_tick();
        terminal
            .draw(|frame| render(frame, &app))
            .map_err(AppErr::Io)?;
        if let Some(effect) = next_effect(&mut app)? {
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

fn next_effect(app: &mut App) -> AppResult<Option<Effect>> {
    if !event::poll(Duration::from_millis(250))? {
        return Ok(None);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(None);
    };

    (key.kind == KeyEventKind::Press)
        .then(|| app.handle_key(key))
        .transpose()
}

fn render(frame: &mut ratatui::Frame<'_>, app: &App) {
    let [main, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(4)])
        .areas(frame.area());

    ui::render(frame, main, app);

    frame.render_widget(
        Paragraph::new(ui::footer_lines(app))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Status / shortcuts • theme: {} ", app.theme.name))
                    .border_style(app.theme.inactive_border()),
            )
            .wrap(Wrap { trim: true }),
        footer,
    );

    if app.show_help {
        let area = ui::centered_rect(68, 76, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(ui::help_lines(app))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Help (?) ")
                        .border_style(app.theme.focused_border()),
                )
                .wrap(Wrap { trim: true }),
            area,
        );
    }
}
