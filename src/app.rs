use crossterm::event::KeyEvent;

use crate::{error::AppResult, shared::ui::UiTheme, state::AppState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    None,
    Quit,
    Message(String),
    ToggleHelp,
}

pub struct App {
    pub state: AppState,
    pub theme: UiTheme,
    pub message: String,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
            theme: UiTheme::default(),
            message: AppState::help().into(),
            show_help: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppResult<Effect> {
        let effect = self.state.handle_key(key);
        self.apply_local_effect(effect)
    }

    fn apply_local_effect(&mut self, effect: Effect) -> AppResult<Effect> {
        Ok(match effect {
            Effect::Message(message) => {
                self.message = message;
                Effect::None
            }
            Effect::ToggleHelp => {
                self.show_help = !self.show_help;
                Effect::None
            }
            passthrough => passthrough,
        })
    }
}
