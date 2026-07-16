use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use super::model::{TargetItem, TargetStatus};
use super::tree::{
    find_path, flatten_tree, toggle_path, TreeBadge, TargetTreeView, TreeNode,
    TreeRow,
};

pub struct TargetPanel {
    pub items: Vec<TargetItem>,
    pub selected: usize,
    pub tree: TreeNode,
    pub view: TargetTreeView,
    pub cursor: usize,
    pub rows: Vec<TreeRow>,
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
        };
        panel.sync_selection();
        panel
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    pub fn sync_selection(&mut self) {
        for (index, item) in self.items.iter_mut().enumerate() {
            item.status = if index == self.selected {
                TargetStatus::Selected
            } else {
                TargetStatus::Loaded
            };
        }
        let selected_spec = self.items.get(self.selected).map(|item| item.spec.as_str());
        self.tree = super::tree::build_target_tree(&self.items, self.view, selected_spec);
        self.rows = flatten_tree(&self.tree);
        if self.cursor >= self.rows.len() {
            self.cursor = self.rows.len().saturating_sub(1);
        }
    }

    pub fn set_view(&mut self, view: TargetTreeView) {
        self.view = view;
        self.sync_selection();
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

    pub fn current_selected_node(&self) -> Option<&TreeNode> {
        self.current_row()
            .and_then(|row| find_path(&self.tree, &row.path))
    }

    fn sync_selected_to_cursor(&mut self) {
        if let Some(row) = self.current_row() {
            if let Some(spec) = &row.spec {
                if let Some(index) = self.items.iter().position(|item| item.spec == *spec) {
                    self.selected = index;
                    self.sync_selection();
                }
            }
        }
    }
}

pub fn render(frame: &mut Frame<'_>, area: Rect, panel: &TargetPanel, focused: bool) {
    let items = panel
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let prefix = "  ".repeat(row.depth);
            ListItem::new(Line::from(vec![
                Span::raw(prefix),
                if row.has_children { Span::raw("▸ ") } else { Span::raw("  ") },
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
        "Targets / {}{}",
        panel.view.title(),
        if focused { " *" } else { "" }
    );
    let block = Block::default().borders(Borders::ALL).title(title);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
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
}
