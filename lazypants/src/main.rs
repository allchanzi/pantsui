mod domain;
mod enums;
mod error;
mod terminal;

use crossterm::{event, event::Event, event::KeyCode};
use domain::state::AppState;
use ratatui::{Frame, layout::Constraint, layout::Layout, widgets::Block};
use terminal::session::TerminalSession;

use crate::error::{AppErr, AppResult};

fn main() -> AppResult<()> {
    let app_state = AppState::default();
    let mut terminal_session = TerminalSession::setup_session()?;

    loop {
        terminal_session
            .terminal
            .draw(|frame| ui(frame, &app_state))
            .map_err(|_| AppErr::Io)?;
        if let Event::Key(key) = event::read().map_err(|_| AppErr::Io)?
            && key.code == KeyCode::Char('q')
        {
            break;
        }
    }

    terminal_session.restore_session()?;
    Ok(())
}

fn ui(frame: &mut Frame, state: &AppState) {
    let [left, right] = Layout::horizontal([Constraint::Fill(1); 2]).areas(frame.area());
    let [top_right, bottom_right] = Layout::vertical([Constraint::Fill(1); 2]).areas(right);

    frame.render_widget(Block::bordered().title("Left Block"), left);
    frame.render_widget(Block::bordered().title("Top Right Block"), top_right);
    frame.render_widget(Block::bordered().title("Bottom Right Block"), bottom_right);
}
