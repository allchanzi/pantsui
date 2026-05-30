use std::io::{Stdout, stdout};

use crossterm::{ExecutableCommand, terminal};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::error::{AppErr, AppResult};

#[derive(Debug)]
pub struct TerminalSession {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    pub fn setup_session() -> AppResult<Self> {
        let backend = CrosstermBackend::new(stdout());
        terminal::enable_raw_mode().map_err(|_| AppErr::Terminal)?;
        stdout()
            .execute(terminal::EnterAlternateScreen)
                .map_err(|_| AppErr::Io)?;

        let terminal = Terminal::new(backend).map_err(|_| AppErr::Io)?;
        let session = TerminalSession { terminal };
        Ok(session)
    }

    pub fn restore_session(&mut self) -> AppResult<&Self> {
        stdout()
            .execute(terminal::LeaveAlternateScreen)
            .map_err(|_| AppErr::Io)?;
        terminal::disable_raw_mode().map_err(|_| AppErr::Terminal)?;
        Ok(self)
    }
}
