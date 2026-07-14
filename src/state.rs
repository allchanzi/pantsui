use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::Effect,
    entities::Panel,
    features::version::ui::VersionPanel,
    shared::ui::UiTheme,
};

const HELP: &str = "? help • Tab cycle focus • h/j/k/l move focus • 1..5 jump to panel • q quit";

pub struct AppState {
    pub active_panel: Panel,
    pub version_panel: VersionPanel,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_panel: Panel::Version,
            version_panel: VersionPanel::new(true),
        }
    }

    pub fn help() -> &'static str {
        HELP
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Effect {
        if key.modifiers.contains(KeyModifiers::ALT) {
            return self.move_focus_by_key(key.code);
        }

        match key.code {
            KeyCode::Char('?') => Effect::ToggleHelp,
            KeyCode::Char('q') | KeyCode::Esc => Effect::Quit,
            KeyCode::Tab => self.set_active_panel(self.active_panel.next()),
            KeyCode::Char('1') => self.set_active_panel(Panel::Version),
            KeyCode::Char('2') => self.set_active_panel(Panel::Mode),
            KeyCode::Char('3') => self.set_active_panel(Panel::FileExplorer),
            KeyCode::Char('4') => self.set_active_panel(Panel::Main),
            KeyCode::Char('5') => self.set_active_panel(Panel::Runs),
            KeyCode::Char('h') | KeyCode::Left => self.set_active_panel(self.active_panel.move_left()),
            KeyCode::Char('l') | KeyCode::Right => {
                self.set_active_panel(self.active_panel.move_right())
            }
            KeyCode::Char('k') | KeyCode::Up => self.set_active_panel(self.active_panel.move_up()),
            KeyCode::Char('j') | KeyCode::Down => {
                self.set_active_panel(self.active_panel.move_down())
            }
            _ => Effect::None,
        }
    }

    pub fn panel_content(&self, panel: Panel, theme: &UiTheme) -> Paragraph<'static> {
        let lines = match panel {
            Panel::Version => vec![
                Line::from(vec![
                    Span::styled("Source:", theme.kind()),
                    Span::raw(" "),
                    Span::styled("pants version", theme.value()),
                ]),
                Line::from(""),
                Line::from("Use Tab or h/j/k/l to move focus."),
                Line::from("Sub-tabs are planned next, same shell style stays."),
            ],
            Panel::Mode => vec![
                Line::from(Span::styled("Planned modes", theme.title())),
                Line::from(""),
                Line::from(vec![Span::styled("• Browse", theme.kind()), Span::raw(" targets")]),
                Line::from(vec![Span::styled("• Pick", theme.kind()), Span::raw(" goal")]),
                Line::from(vec![
                    Span::styled("• Execute", theme.kind()),
                    Span::raw(" / inspect results"),
                ]),
            ],
            Panel::FileExplorer => vec![
                Line::from(Span::styled("Planned source tree", theme.title())),
                Line::from(""),
                Line::from("This panel will list Pants-relevant source roots / files."),
                Line::from(vec![
                    Span::styled("Status:", theme.kind()),
                    Span::raw(" "),
                    Span::styled("placeholder", theme.warning()),
                ]),
            ],
            Panel::Main => vec![
                Line::from(Span::styled("Planned active workflow", theme.title())),
                Line::from(""),
                Line::from("Targets / goals / details will land here."),
            ],
            Panel::Runs => vec![
                Line::from(Span::styled("Planned run history", theme.title())),
                Line::from(""),
                Line::from("Recent Pants actions, statuses and logs will appear here."),
                Line::from(vec![
                    Span::styled("Style:", theme.kind()),
                    Span::raw(" PDE-like shell, pantsui content"),
                ]),
            ],
        };

        Paragraph::new(lines)
    }

    pub fn set_active_panel(&mut self, panel: Panel) -> Effect {
        self.active_panel = panel;
        self.version_panel.active(panel == Panel::Version);
        Effect::Message(format!("Focus: {}", panel.title()))
    }

    fn move_focus_by_key(&mut self, key: KeyCode) -> Effect {
        let next = match key {
            KeyCode::Char('h') | KeyCode::Char('a') => self.active_panel.move_left(),
            KeyCode::Char('l') | KeyCode::Char('d') => self.active_panel.move_right(),
            KeyCode::Char('k') | KeyCode::Char('w') => self.active_panel.move_up(),
            KeyCode::Char('j') | KeyCode::Char('s') => self.active_panel.move_down(),
            _ => self.active_panel,
        };
        self.set_active_panel(next)
    }
}
