use crate::shared::{
    self,
    ui::{TAB_JOINER, UiTheme},
};
use itertools::Itertools;
use ratatui::{
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use strum::{Display, EnumIter, IntoEnumIterator};

const SHORTCUT_VALUE: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Display)]
pub enum SubTab {
    #[strum(to_string = "Version")]
    Version,
    #[strum(to_string = "Dependencies")]
    Dependencies,
    #[strum(to_string = "Sub-projects")]
    SubProjects,
}

pub fn version() -> String {
    match shared::pants::commands::pants_version() {
        Ok(version_string) => version_string,
        Err(err) => err.to_string(),
    }
}

pub fn dependencies() -> String {
    "TBD ...".to_string()
}

pub fn sub_projects() -> String {
    "TBD ...".to_string()
}

#[derive(Debug, Clone)]
pub struct VersionPanel {
    pub version_string: String,
    pub dependencies: Vec<String>,
    pub sub_projects: Vec<String>,
    pub active_tab: SubTab,
    pub active: bool,
    pub theme: UiTheme,
}

impl VersionPanel {
    pub fn new(active: bool) -> Self {
        Self {
            version_string: version(),
            dependencies: vec![dependencies()],
            sub_projects: vec![sub_projects()],
            active_tab: SubTab::Version,
            active,
            theme: UiTheme::default(),
        }
    }

    pub fn clone_with_theme(&self, theme: &UiTheme) -> Self {
        let mut clone = self.clone();
        clone.theme = theme.clone();
        clone
    }

    fn header(&self) -> Line<'static> {
        let mut tab_spans: Vec<Span<'static>> = SubTab::iter()
            .map(|tab| {
                if self.active_tab == tab && self.active {
                    self.theme.active_span(tab.to_string())
                } else {
                    self.theme.inactive_span(tab.to_string())
                }
            })
            .collect();
        tab_spans.insert(0, {
            if self.active {
                self.theme.active_span(format!("[{}]", SHORTCUT_VALUE))
            } else {
                self.theme.inactive_span(format!("[{}]", SHORTCUT_VALUE))
            }
        });
        let separated_spans = Itertools::intersperse(
            tab_spans.into_iter(),
            self.theme.inactive_span(String::from(TAB_JOINER)),
        )
        .collect::<Vec<Span<'static>>>();
        Line::from_iter(separated_spans)
    }

    fn content(&self) -> Paragraph<'static> {
        match self.active_tab {
            SubTab::Version => Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Global / Version", self.theme.title()),
                    Span::styled("  live command output", self.theme.muted()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Value:", self.theme.kind()),
                    Span::raw(" "),
                    Span::styled(self.version_string.trim().to_string(), self.theme.value()),
                ]),
            ]),
            SubTab::Dependencies => Paragraph::new(self.dependencies.join(" ")),
            SubTab::SubProjects => Paragraph::new(self.sub_projects.join(" ")),
        }
    }

    pub fn active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Widget for VersionPanel {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = self
            .theme
            .panel_block(format!("Global / {}", self.header()), self.active)
            .title(self.header());

        self.content().left_aligned().block(block).render(area, buf)
    }
}
