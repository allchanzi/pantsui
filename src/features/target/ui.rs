use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};
use std::collections::BTreeSet;

use super::model::{TargetItem, TargetStatus};
use super::tree::{TargetTreeView, TreeBadge, TreeNode, TreeRow, flatten_tree, toggle_path};
use crate::{entities::Panel, shared::ui::UiTheme};

pub struct TargetPanel {
    pub items: Vec<TargetItem>,
    pub selected: usize,
    pub tree: TreeNode,
    pub view: TargetTreeView,
    pub cursor: usize,
    pub rows: Vec<TreeRow>,
    pub filter: String,
}

impl TargetPanel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            tree: TreeNode::branch("Targets"),
            view: TargetTreeView::All,
            cursor: 0,
            rows: Vec::new(),
            filter: String::new(),
        }
    }

    pub fn with_items(items: Vec<TargetItem>) -> Self {
        let mut panel = Self {
            tree: super::tree::build_target_tree(&items, TargetTreeView::All, None),
            items,
            selected: 0,
            view: TargetTreeView::All,
            cursor: 0,
            rows: Vec::new(),
            filter: String::new(),
        };
        panel.sync_selection();
        panel
    }

    pub fn replace_items(&mut self, items: Vec<TargetItem>) {
        let selected_spec = self.items.get(self.selected).map(|item| item.spec.clone());
        let current_row = self
            .current_row()
            .map(|row| (row.path.clone(), row.spec.clone()));
        self.items = items;
        self.selected = selected_spec
            .and_then(|spec| self.items.iter().position(|item| item.spec == spec))
            .unwrap_or(0);
        self.sync_selection();

        if let Some((path, spec)) = current_row
            && let Some(cursor) = self
                .rows
                .iter()
                .position(|row| row.path == path && row.spec == spec)
        {
            self.cursor = cursor;
            self.sync_selected_to_cursor();
        }
    }

    #[cfg(test)]
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    #[cfg(test)]
    pub fn move_down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    pub fn sync_selection(&mut self) {
        let expanded_paths = expanded_paths(&self.tree);
        for (index, item) in self.items.iter_mut().enumerate() {
            item.status = if index == self.selected {
                TargetStatus::Selected
            } else {
                TargetStatus::Loaded
            };
        }
        let selected_spec = self.items.get(self.selected).map(|item| item.spec.as_str());
        let visible_items = self
            .items
            .iter()
            .filter(|item| {
                self.filter.is_empty()
                    || item
                        .spec
                        .to_lowercase()
                        .contains(&self.filter.to_lowercase())
            })
            .cloned()
            .collect::<Vec<_>>();
        self.tree = super::tree::build_target_tree(&visible_items, self.view, selected_spec);
        restore_expanded_paths(&mut self.tree, &expanded_paths);
        self.rows = flatten_tree(&self.tree);
        if self.cursor >= self.rows.len() {
            self.cursor = self.rows.len().saturating_sub(1);
        }
    }

    pub fn next_view(&mut self) {
        self.view = match self.view {
            TargetTreeView::All => TargetTreeView::Tests,
            TargetTreeView::Tests => TargetTreeView::Cached,
            TargetTreeView::Cached => TargetTreeView::CachedFailures,
            TargetTreeView::CachedFailures => TargetTreeView::Running,
            TargetTreeView::Running => TargetTreeView::NeedsRun,
            TargetTreeView::NeedsRun => TargetTreeView::All,
        };
        self.sync_selection();
    }

    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
        self.cursor = 0;
        self.sync_selection();
    }

    pub fn toggle_tree(&mut self) {
        if let Some(row) = self.rows.get(self.cursor) {
            let _ = toggle_path(&mut self.tree, &row.path);
            self.rows = flatten_tree(&self.tree);
            if self.cursor >= self.rows.len() {
                self.cursor = self.rows.len().saturating_sub(1);
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.sync_selected_to_cursor();
    }

    pub fn move_cursor_down(&mut self) {
        let len = self.rows.len();
        if self.cursor + 1 < len {
            self.cursor += 1;
        }
        self.sync_selected_to_cursor();
    }

    pub fn current_row(&self) -> Option<&TreeRow> {
        self.rows.get(self.cursor)
    }

    pub fn scoped_targets(&self) -> Vec<&TargetItem> {
        let Some(row) = self.current_row() else {
            return Vec::new();
        };

        self.items
            .iter()
            .filter(|item| {
                if row.spec.is_some() {
                    item.address_path() == row.path
                } else {
                    path_is_within(item.address_path(), &row.path)
                }
            })
            .collect()
    }

    pub fn scoped_tests(&self) -> Vec<&TargetItem> {
        self.scoped_targets()
            .into_iter()
            .filter(|target| target.is_test)
            .collect()
    }

    pub fn scoped_test_specs(&self) -> Vec<String> {
        self.scoped_tests()
            .into_iter()
            .map(|target| target.spec.clone())
            .collect()
    }

    pub fn visible_test_specs(&self) -> Vec<String> {
        let filter = self.filter.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                item.is_test
                    && self.view.matches(item)
                    && (filter.is_empty() || item.spec.to_lowercase().contains(&filter))
            })
            .map(|item| item.spec.clone())
            .collect()
    }

    pub fn current_pants_spec(&self) -> Option<String> {
        self.current_row().map(|row| {
            if let Some(spec) = &row.spec {
                spec.clone()
            } else if row.path == "." {
                String::from("::")
            } else {
                format!("{}::", row.path)
            }
        })
    }

    fn sync_selected_to_cursor(&mut self) {
        if let Some(row) = self.current_row()
            && let Some(spec) = &row.spec
            && let Some(index) = self.items.iter().position(|item| item.spec == *spec)
        {
            self.selected = index;
            self.sync_selection();
        }
    }
}

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    panel: &TargetPanel,
    focused: bool,
    theme: &UiTheme,
) {
    let items = panel
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let prefix = "  ".repeat(row.depth);
            ListItem::new(Line::from(vec![
                Span::raw(prefix),
                if row.has_children && row.expanded {
                    Span::raw("▾ ")
                } else if row.has_children {
                    Span::raw("▸ ")
                } else {
                    Span::raw("  ")
                },
                badge_span(row.badge),
                Span::raw(" "),
                Span::raw(row.label.clone()),
                if index == panel.cursor {
                    Span::styled("  ←", Style::default().fg(Color::Cyan))
                } else {
                    Span::raw("")
                },
            ]))
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(panel.cursor));

    let title = format!(
        "Global / [{}] {} / {}{}",
        Panel::FileExplorer.shortcut(),
        Panel::FileExplorer.title(),
        panel.view.title(),
        if panel.filter.is_empty() {
            String::new()
        } else {
            format!(" / filter: {}", panel.filter)
        },
    );
    let block = theme.panel_block(title, focused);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
}

fn path_is_within(path: &str, scope: &str) -> bool {
    scope == "."
        || path == scope
        || path
            .strip_prefix(scope)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn expanded_paths(tree: &TreeNode) -> BTreeSet<String> {
    fn visit(node: &TreeNode, paths: &mut BTreeSet<String>) {
        if node.expanded && !node.children.is_empty() {
            paths.insert(node.path.clone());
        }
        for child in &node.children {
            visit(child, paths);
        }
    }

    let mut paths = BTreeSet::new();
    visit(tree, &mut paths);
    paths
}

fn restore_expanded_paths(node: &mut TreeNode, paths: &BTreeSet<String>) {
    if paths.contains(&node.path) {
        node.expanded = true;
    }
    for child in &mut node.children {
        restore_expanded_paths(child, paths);
    }
}

fn badge_span(badge: TreeBadge) -> Span<'static> {
    match badge {
        TreeBadge::Neutral => Span::raw(" "),
        TreeBadge::Cached => Span::styled("●", Style::default().fg(Color::Green)),
        TreeBadge::Failed => Span::styled("●", Style::default().fg(Color::Red)),
        TreeBadge::Running => Span::styled("●", Style::default().fg(Color::Yellow)),
        TreeBadge::NeedsRun => Span::styled("●", Style::default().fg(Color::Blue)),
        TreeBadge::Selected => Span::styled("▶", Style::default().fg(Color::Cyan)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_does_not_move_below_zero() {
        let mut panel = TargetPanel::with_items(vec![TargetItem::new("a"), TargetItem::new("b")]);

        panel.move_up();

        assert_eq!(panel.selected, 0);
    }

    #[test]
    fn selection_does_not_move_past_end() {
        let mut panel = TargetPanel::with_items(vec![TargetItem::new("a"), TargetItem::new("b")]);

        panel.move_down();
        panel.move_down();
        panel.move_down();

        assert_eq!(panel.selected, 1);
    }

    #[test]
    fn sync_selection_marks_selected_item() {
        let mut panel = TargetPanel::with_items(vec![TargetItem::new("a"), TargetItem::new("b")]);

        panel.selected = 1;
        panel.sync_selection();

        assert_eq!(panel.items[0].status, TargetStatus::Loaded);
        assert_eq!(panel.items[1].status, TargetStatus::Selected);
    }

    #[test]
    fn tree_builds_grouped_nodes() {
        let panel = TargetPanel::with_items(vec![
            TargetItem::new("src/lib:lib"),
            TargetItem::new("tests/api:test_api"),
        ]);

        assert_eq!(panel.tree.children.len(), 2);
    }

    #[test]
    fn next_view_cycles() {
        let mut panel = TargetPanel::new();
        panel.next_view();
        assert_eq!(panel.view, TargetTreeView::Tests);
    }

    #[test]
    fn toggle_tree_collapses_root() {
        let mut panel = TargetPanel::with_items(vec![TargetItem::new("src/lib:lib")]);
        panel.toggle_tree();
        assert!(!panel.tree.expanded);
    }

    #[test]
    fn directory_scope_returns_recursive_targets_and_tests() {
        let mut panel = TargetPanel::with_items(vec![
            TargetItem::new("src/api:lib"),
            TargetItem::new("src/api/test_users.py:tests").test_target(),
            TargetItem::new("src/other:lib"),
        ]);
        panel.cursor = panel
            .rows
            .iter()
            .position(|row| row.path == "src/api" && row.spec.is_none())
            .expect("src/api branch");

        assert_eq!(
            panel
                .scoped_targets()
                .into_iter()
                .map(|target| target.spec.as_str())
                .collect::<Vec<_>>(),
            vec!["src/api:lib", "src/api/test_users.py:tests"]
        );
        assert_eq!(
            panel.scoped_test_specs(),
            vec![String::from("src/api/test_users.py:tests")]
        );
    }

    #[test]
    fn file_scope_returns_all_owners_for_that_file() {
        let mut panel = TargetPanel::with_items(vec![
            TargetItem::new("src/api/users.py:lib"),
            TargetItem::new("src/api/users.py:generated"),
            TargetItem::new("src/api/other.py:lib"),
        ]);
        panel.cursor = panel
            .rows
            .iter()
            .position(|row| row.spec.as_deref() == Some("src/api/users.py:lib"))
            .expect("users.py target");

        assert_eq!(panel.scoped_targets().len(), 2);
    }

    #[test]
    fn prefix_matching_respects_path_boundaries() {
        assert!(path_is_within("src/api/test.py", "src/api"));
        assert!(!path_is_within("src/api_v2/test.py", "src/api"));
    }

    #[test]
    fn replacing_items_preserves_the_current_scope() {
        let mut panel = TargetPanel::with_items(vec![
            TargetItem::new("src/api:lib"),
            TargetItem::new("src/api/test_users.py:tests").test_target(),
        ]);
        panel.cursor = panel
            .rows
            .iter()
            .position(|row| row.path == "src/api" && row.spec.is_none())
            .expect("src/api branch");

        panel.replace_items(vec![
            TargetItem::new("src/api:lib"),
            TargetItem::new("src/api/test_users.py:tests")
                .test_target()
                .cached_success(),
        ]);

        assert_eq!(
            panel.current_row().map(|row| row.path.as_str()),
            Some("src/api")
        );
        assert!(panel.current_row().is_some_and(|row| row.spec.is_none()));
    }

    #[test]
    fn filter_limits_visible_tree_rows_without_losing_source_items() {
        let mut panel = TargetPanel::with_items(vec![
            TargetItem::new("src/api:lib"),
            TargetItem::new("src/payments:lib"),
        ]);

        panel.set_filter("payments");

        assert!(panel.rows.iter().any(|row| row.label.contains("payments")));
        assert!(!panel.rows.iter().any(|row| row.label.contains("api:lib")));
        assert_eq!(panel.items.len(), 2);
    }

    #[test]
    fn creates_recursive_pants_specs_for_directory_scopes() {
        let mut panel = TargetPanel::with_items(vec![
            TargetItem::new("src/api:lib"),
            TargetItem::new("src/api/users.py:lib"),
        ]);
        panel.cursor = panel
            .rows
            .iter()
            .position(|row| row.path == "src/api" && row.spec.is_none())
            .expect("directory row");

        assert_eq!(panel.current_pants_spec().as_deref(), Some("src/api::"));
    }

    #[test]
    fn visible_test_specs_honor_view_and_text_filter() {
        let mut panel = TargetPanel::with_items(vec![
            TargetItem::new("tests/api/test_users.py:tests").test_target(),
            TargetItem::new("tests/payments/test_cards.py:tests").test_target(),
            TargetItem::new("src/api:lib"),
        ]);
        panel.set_filter("payments");

        assert_eq!(
            panel.visible_test_specs(),
            vec![String::from("tests/payments/test_cards.py:tests")]
        );
    }
}
