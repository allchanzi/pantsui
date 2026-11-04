use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
};

pub const TAB_JOINER: &str = " • ";

#[derive(Debug, Clone)]
pub struct UiTheme {
    text: Color,
    subtle: Color,
    blue: Color,
    green: Color,
    yellow: Color,
    cyan: Color,
    pub base: Color,
    pub pink: Color,
    pub red: Color,
    pub lavender: Color,
    pub surface: Color,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            text: Color::Rgb(205, 214, 244),
            subtle: Color::Rgb(108, 112, 134),
            blue: Color::Rgb(137, 180, 250),
            green: Color::Rgb(166, 227, 161),
            yellow: Color::Rgb(249, 226, 175),
            cyan: Color::Rgb(148, 226, 213),
            base: Color::Rgb(30, 30, 46),
            pink: Color::Rgb(245, 194, 231),
            red: Color::Rgb(243, 139, 168),
            lavender: Color::Rgb(180, 190, 254),
            surface: Color::Rgb(69, 71, 90),
        }
    }
}

impl UiTheme {
    pub fn text_color(&self) -> Color {
        self.text
    }

    pub fn blue_color(&self) -> Color {
        self.blue
    }

    pub fn yellow_color(&self) -> Color {
        self.yellow
    }

    pub fn subtle_color(&self) -> Color {
        self.subtle
    }

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
}
