use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
};

pub const TAB_JOINER: &str = " • ";

#[derive(Debug, Clone)]
pub struct UiTheme {
    pub name: String,
    text: Color,
    subtle: Color,
    surface: Color,
    blue: Color,
    green: Color,
    yellow: Color,
    cyan: Color,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            name: "catppuccin-mocha".into(),
            text: Color::Rgb(205, 214, 244),
            subtle: Color::Rgb(108, 112, 134),
            surface: Color::Rgb(49, 50, 68),
            blue: Color::Rgb(137, 180, 250),
            green: Color::Rgb(166, 227, 161),
            yellow: Color::Rgb(249, 226, 175),
            cyan: Color::Rgb(148, 226, 213),
        }
    }
}

impl UiTheme {
    pub fn panel_block(&self, title: impl Into<String>, focused: bool) -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(if focused {
                self.focused_border()
            } else {
                self.inactive_border()
            })
            .title(title.into())
    }

    pub fn focused_border(&self) -> Style {
        Style::new().fg(self.cyan).add_modifier(Modifier::BOLD)
    }

    pub fn inactive_border(&self) -> Style {
        Style::new().fg(self.subtle)
    }

    pub fn active_span(&self, text: impl Into<String>) -> Span<'static> {
        Span::styled(text.into(), self.focused_border())
    }

    pub fn inactive_span(&self, text: impl Into<String>) -> Span<'static> {
        Span::styled(text.into(), self.muted())
    }

    pub fn muted(&self) -> Style {
        Style::new().fg(self.subtle)
    }

    pub fn title(&self) -> Style {
        Style::new().fg(self.text).add_modifier(Modifier::BOLD)
    }

    pub fn kind(&self) -> Style {
        Style::new().fg(self.blue)
    }

    pub fn value(&self) -> Style {
        Style::new().fg(self.green).add_modifier(Modifier::BOLD)
    }

    pub fn warning(&self) -> Style {
        Style::new().fg(self.yellow).add_modifier(Modifier::BOLD)
    }

    pub fn help_title(&self) -> Style {
        Style::new().fg(self.cyan).add_modifier(Modifier::BOLD)
    }

    pub fn selected(&self) -> Style {
        Style::new()
            .fg(self.blue)
            .bg(self.surface)
            .add_modifier(Modifier::BOLD)
    }
}
