use super::data::SubProject;
use crate::shared::ui::{TAB_JOINER, UiTheme};
use itertools::Itertools;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};
use std::collections::HashMap;
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

impl SubTab {
    fn next(self) -> Self {
        match self {
            Self::Version => Self::Dependencies,
            Self::Dependencies => Self::SubProjects,
            Self::SubProjects => Self::Version,
        }
    }

    fn previous(self) -> Self {
        self.next().next()
    }
}

type DependencyResult = Result<Vec<String>, String>;

/// A dependency lookup the app has to run. `generation` lets a refresh drop late answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyRequest {
    pub scope: String,
    pub generation: u64,
}

#[derive(Debug, Clone)]
pub struct VersionPanel {
    pub version_string: String,
    pub sub_projects: Vec<SubProject>,
    pub active_tab: SubTab,
    pub active: bool,
    pub theme: UiTheme,
    pub scroll: usize,
    scope: Option<String>,
    /// Pants needs seconds per lookup, so going back to a selection must not repeat it.
    dependencies: HashMap<String, DependencyResult>,
    in_flight: Option<String>,
    generation: u64,
}

impl VersionPanel {
    pub fn new(active: bool) -> Self {
        Self {
            // App bootstrap fills this through its repository-scoped PantsClient.
            // Keeping construction side-effect free also makes shell navigation testable.
            version_string: String::from("unavailable"),
            sub_projects: Vec::new(),
            active_tab: SubTab::Version,
            active,
            theme: UiTheme::default(),
            scroll: 0,
            scope: None,
            dependencies: HashMap::new(),
            in_flight: None,
            generation: 0,
        }
    }

    pub fn clone_with_theme(&self, theme: &UiTheme) -> Self {
        let mut clone = self.clone();
        clone.theme = theme.clone();
        clone
    }

    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        self.scroll = 0;
    }

    pub fn previous_tab(&mut self) {
        self.active_tab = self.active_tab.previous();
        self.scroll = 0;
    }

    /// Follows the selection while Dependencies is open.
    ///
    /// Only one lookup runs at a time: moving through the tree must not start a Pants
    /// process per row. When it ends, the next call asks for wherever the cursor is now.
    pub fn request_dependencies(&mut self, scope: Option<String>) -> Option<DependencyRequest> {
        if self.active_tab != SubTab::Dependencies {
            return None;
        }
        if self.scope != scope {
            self.scope = scope;
            self.scroll = 0;
        }
        let scope = self.scope.as_ref()?;
        if self.in_flight.is_some() || self.dependencies.contains_key(scope) {
            return None;
        }
        self.in_flight = Some(scope.clone());
        Some(DependencyRequest {
            scope: scope.clone(),
            generation: self.generation,
        })
    }

    pub fn finish_dependencies(&mut self, request: DependencyRequest, result: DependencyResult) {
        if request.generation != self.generation {
            return;
        }
        self.in_flight = None;
        self.dependencies.insert(request.scope, result);
    }

    #[cfg(test)]
    pub fn has_answer(&self, scope: &str) -> bool {
        self.dependencies.contains_key(scope)
    }

    /// BUILD files may have changed: forget the answers and any lookup still running.
    pub fn invalidate_dependencies(&mut self) {
        self.dependencies.clear();
        self.in_flight = None;
        self.generation += 1;
    }

    /// `[1] Version • Dependencies • Sub-projects`, with the open section highlighted.
    fn header(&self) -> Line<'static> {
        let tab_spans: Vec<Span<'static>> = SubTab::iter()
            .map(|tab| {
                if self.active_tab == tab {
                    self.theme.active_span(tab.to_string())
                } else {
                    self.theme.inactive_span(tab.to_string())
                }
            })
            .collect();
        let mut spans = vec![self.theme.inactive_span(format!("[{SHORTCUT_VALUE}] "))];
        spans.extend(Itertools::intersperse(
            tab_spans.into_iter(),
            self.theme.inactive_span(String::from(TAB_JOINER)),
        ));
        Line::from_iter(spans)
    }

    fn dependency_lines(&self) -> Vec<Line<'static>> {
        let muted = |text: String| Line::from(Span::styled(text, self.theme.muted()));
        let Some(scope) = &self.scope else {
            return vec![muted(String::from(
                "Select a target or folder in [3] to see its dependencies.",
            ))];
        };
        match self.dependencies.get(scope) {
            None => vec![muted(format!("Loading dependencies of {scope}…"))],
            Some(Err(error)) => {
                let mut lines = vec![Line::from(Span::styled(
                    format!("Could not load {scope} • [R] refresh"),
                    self.theme.warning(),
                ))];
                lines.extend(error.lines().map(|line| Line::from(line.to_string())));
                lines
            }
            Some(Ok(addresses)) => {
                let relation = if scope.ends_with("::") {
                    "outside it"
                } else {
                    "direct"
                };
                let mut lines = vec![Line::from(vec![
                    Span::styled(scope.clone(), self.theme.kind()),
                    Span::styled(
                        format!(" • {} {relation}", addresses.len()),
                        self.theme.muted(),
                    ),
                ])];
                if addresses.is_empty() {
                    lines.push(muted(String::from("No dependencies.")));
                }
                lines.extend(addresses.iter().map(|address| Line::from(address.clone())));
                lines
            }
        }
    }

    fn sub_project_lines(&self) -> Vec<Line<'static>> {
        if self.sub_projects.is_empty() {
            return vec![Line::from(Span::styled(
                "No targets loaded.",
                self.theme.muted(),
            ))];
        }
        let mut lines = vec![Line::from(Span::styled(
            "targets  tests  folder",
            self.theme.muted(),
        ))];
        lines.extend(self.sub_projects.iter().map(|project| {
            Line::from(vec![
                Span::styled(
                    format!("{:>7}  {:>5}  ", project.targets, project.tests),
                    self.theme.muted(),
                ),
                Span::styled(project.path.clone(), self.theme.value()),
            ])
        }));
        lines
    }

    fn body(&self) -> Paragraph<'static> {
        let lines = match self.active_tab {
            SubTab::Version => vec![Line::from(vec![
                Span::styled("pants version:", self.theme.kind()),
                Span::raw(" "),
                Span::styled(self.version_string.trim().to_string(), self.theme.value()),
            ])],
            SubTab::Dependencies => self.dependency_lines(),
            SubTab::SubProjects => self.sub_project_lines(),
        };
        Paragraph::new(lines).wrap(Wrap { trim: false })
    }

    /// The scrolling rows and the hint row of a panel, with the hint only when focused.
    fn split(&self, inner: Rect) -> (Rect, Option<Rect>) {
        if self.active && inner.height >= 2 {
            let [body, hint] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(inner);
            (body, Some(hint))
        } else {
            (inner, None)
        }
    }

    /// Rows the body can scroll, and its visible height, for a panel drawn in `area`.
    pub fn scroll_range(&self, area: Rect) -> (usize, usize) {
        let inner = self
            .theme
            .panel_block_line(self.header(), self.active)
            .inner(area);
        let (body, _) = self.split(inner);
        let height = usize::from(body.height);
        (
            self.body().line_count(body.width).saturating_sub(height),
            height,
        )
    }

    pub fn active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Widget for VersionPanel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = self.theme.panel_block_line(self.header(), self.active);
        let (body, hint) = self.split(block.inner(area));
        block.render(area, buf);

        let (max, _) = self.scroll_range(area);
        let scroll = self.scroll.min(max).min(usize::from(u16::MAX)) as u16;
        self.body().scroll((scroll, 0)).render(body, buf);
        if let Some(hint) = hint {
            let text = if self.active_tab == SubTab::Version {
                "[h/l] section"
            } else {
                "[h/l] section • [j/k] scroll"
            };
            Paragraph::new(Span::styled(text, self.theme.muted())).render(hint, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn screen(panel: &VersionPanel) -> String {
        let mut terminal = Terminal::new(TestBackend::new(60, 5)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(panel.clone(), frame.area()))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn title_has_the_number_once_and_at_the_start() {
        let text = screen(&VersionPanel::new(true));
        assert!(
            text.starts_with("┌[1] Version • Dependencies • Sub-projects"),
            "{text}"
        );
        assert_eq!(text.matches("[1]").count(), 1, "{text}");
        assert_eq!(text.matches("Dependencies").count(), 1, "{text}");
    }

    #[test]
    fn sections_cycle_both_ways() {
        let mut panel = VersionPanel::new(true);
        panel.next_tab();
        assert_eq!(panel.active_tab, SubTab::Dependencies);
        assert!(screen(&panel).contains("Select a target or folder"));
        panel.previous_tab();
        panel.previous_tab();
        assert_eq!(panel.active_tab, SubTab::SubProjects);
        assert!(screen(&panel).contains("No targets loaded."));
    }

    fn dependencies_open() -> VersionPanel {
        let mut panel = VersionPanel::new(true);
        panel.next_tab();
        panel
    }

    fn scope(text: &str) -> Option<String> {
        Some(text.to_string())
    }

    #[test]
    fn dependencies_load_only_while_their_section_is_open() {
        let mut panel = VersionPanel::new(true);
        assert_eq!(panel.request_dependencies(scope("a::")), None);
        panel.next_tab();
        assert!(panel.request_dependencies(scope("a::")).is_some());
    }

    #[test]
    fn nothing_is_looked_up_without_a_selection() {
        assert_eq!(dependencies_open().request_dependencies(None), None);
    }

    #[test]
    fn moving_during_a_lookup_starts_no_second_process_and_ends_on_the_cursor() {
        let mut panel = dependencies_open();
        let first = panel.request_dependencies(scope("a::")).unwrap();
        assert_eq!(panel.request_dependencies(scope("b::")), None);
        assert_eq!(panel.request_dependencies(scope("c::")), None);
        assert!(screen(&panel).contains("Loading dependencies of c::"));

        panel.finish_dependencies(first, Ok(vec![]));
        assert_eq!(
            panel.request_dependencies(scope("c::")).unwrap().scope,
            "c::"
        );
    }

    #[test]
    fn a_known_answer_is_shown_without_asking_pants_again() {
        let mut panel = dependencies_open();
        let request = panel.request_dependencies(scope("a:a")).unwrap();
        panel.finish_dependencies(request, Ok(vec![String::from("b:b")]));
        assert_eq!(panel.request_dependencies(scope("a:a")), None);
        let text = screen(&panel);
        assert!(text.contains("a:a • 1 direct"), "{text}");
        assert!(text.contains("b:b"), "{text}");
    }

    #[test]
    fn a_failure_is_shown_with_the_way_to_retry() {
        let mut panel = dependencies_open();
        let request = panel.request_dependencies(scope("a:a")).unwrap();
        panel.finish_dependencies(request, Err(String::from("first\nsecond")));
        let text = screen(&panel);
        assert!(text.contains("Could not load a:a • [R] refresh"), "{text}");
        assert!(text.contains("first"), "{text}");
        // The rest of the message is one scroll away, not cut off.
        assert_eq!(panel.scroll_range(Rect::new(0, 0, 60, 5)).0, 1);
    }

    #[test]
    fn a_refresh_forgets_answers_and_ignores_a_lookup_still_running() {
        let mut panel = dependencies_open();
        let old = panel.request_dependencies(scope("a:a")).unwrap();
        panel.invalidate_dependencies();
        let new = panel.request_dependencies(scope("a:a")).unwrap();

        panel.finish_dependencies(old, Ok(vec![String::from("stale:stale")]));
        assert!(screen(&panel).contains("Loading"));
        panel.finish_dependencies(new, Ok(vec![String::from("fresh:fresh")]));
        assert!(screen(&panel).contains("fresh:fresh"));
    }

    #[test]
    fn a_long_list_scrolls_and_keeps_the_hint_visible() {
        let mut panel = dependencies_open();
        let request = panel.request_dependencies(scope("a:a")).unwrap();
        let addresses = (0..20).map(|index| format!("dep:{index:02}")).collect();
        panel.finish_dependencies(request, Ok(addresses));

        let area = Rect::new(0, 0, 60, 5);
        // 2 border rows and 1 hint row leave 2; 21 lines leave 19 to scroll.
        assert_eq!(panel.scroll_range(area), (19, 2));
        panel.scroll = 19;
        let text = screen(&panel);
        assert!(text.contains("dep:19"), "{text}");
        assert!(!text.contains("dep:00"), "{text}");
        assert!(text.contains("[j/k] scroll"), "{text}");
    }

    #[test]
    fn sub_projects_show_their_counts() {
        let mut panel = VersionPanel::new(false);
        panel.sub_projects = vec![SubProject {
            path: String::from("projects/a"),
            targets: 12,
            tests: 3,
        }];
        panel.next_tab();
        panel.next_tab();
        let text = screen(&panel);
        assert!(text.contains("     12      3  projects/a"), "{text}");
    }
}
