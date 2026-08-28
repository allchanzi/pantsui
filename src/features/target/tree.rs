use std::collections::BTreeMap;

use super::model::{TargetCacheState, TargetItem, TargetRunState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetTreeView {
    All,
    Tests,
    Cached,
    CachedFailures,
    Running,
    NeedsRun,
}

impl TargetTreeView {
    pub fn title(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Tests => "Tests",
            Self::Cached => "Cached",
            Self::CachedFailures => "Failed cache",
            Self::Running => "Running",
            Self::NeedsRun => "Needs run",
        }
    }

    pub fn matches(self, item: &TargetItem) -> bool {
        match self {
            Self::All => true,
            Self::Tests => item.is_test,
            Self::Cached => item.cache_state == TargetCacheState::CachedSuccess,
            Self::CachedFailures => item.cache_state == TargetCacheState::CachedFailure,
            Self::Running => item.run_state == TargetRunState::Running,
            Self::NeedsRun => item.cache_state == TargetCacheState::NeedsRun,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeBadge {
    Neutral,
    Cached,
    Failed,
    Running,
    NeedsRun,
    Selected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub name: String,
    pub spec: Option<String>,
    pub path: String,
    pub badge: TreeBadge,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

impl TreeNode {
    pub fn branch(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            spec: None,
            path: String::new(),
            badge: TreeBadge::Neutral,
            children: Vec::new(),
            expanded: false,
        }
    }

    pub fn leaf(item: &TargetItem, selected: bool, path: String) -> Self {
        let badge = if selected {
            TreeBadge::Selected
        } else {
            match (item.run_state, item.cache_state) {
                (TargetRunState::Running, _) => TreeBadge::Running,
                (_, TargetCacheState::CachedSuccess) => TreeBadge::Cached,
                (_, TargetCacheState::CachedFailure) => TreeBadge::Failed,
                (_, TargetCacheState::NeedsRun) => TreeBadge::NeedsRun,
                _ => TreeBadge::Neutral,
            }
        };

        Self {
            name: item.display_label(),
            spec: Some(item.spec.clone()),
            path,
            badge,
            children: Vec::new(),
            expanded: true,
        }
    }
}

pub fn build_target_tree(
    items: &[TargetItem],
    view: TargetTreeView,
    selected: Option<&str>,
) -> TreeNode {
    let mut root = TreeNode::branch(view.title());
    root.path = String::from(".");
    root.expanded = true;
    let mut grouped: BTreeMap<&str, Vec<&TargetItem>> = BTreeMap::new();

    for item in items.iter().filter(|item| view.matches(item)) {
        let group = item.address_path().split('/').next().unwrap_or(".");
        grouped.entry(group).or_default().push(item);
    }

    for (group, grouped_items) in grouped {
        if group == "." {
            root.children.extend(grouped_items.into_iter().map(|item| {
                TreeNode::leaf(
                    item,
                    selected == Some(item.spec.as_str()),
                    String::from("."),
                )
            }));
            continue;
        }
        let mut group_node = TreeNode::branch(group);
        group_node.path = group.to_string();
        group_node.expanded = grouped_items
            .iter()
            .any(|item| selected == Some(item.spec.as_str()));
        for item in grouped_items {
            let segments = path_segments(item.address_path());
            let remaining_segments = if segments.first().is_some_and(|segment| *segment == group) {
                &segments[1..]
            } else {
                &segments
            };
            insert_segments(
                &mut group_node,
                remaining_segments,
                item,
                selected == Some(item.spec.as_str()),
                group.to_string(),
            );
        }
        root.children.push(group_node);
    }

    root
}

fn path_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn insert_segments(
    current: &mut TreeNode,
    segments: &[&str],
    item: &TargetItem,
    selected: bool,
    current_path: String,
) {
    match segments {
        [] => current
            .children
            .push(TreeNode::leaf(item, selected, current_path)),
        [leaf] => current.children.push(TreeNode::leaf(
            item,
            selected,
            format!("{}/{}", current_path, leaf),
        )),
        [head, tail @ ..] => {
            let index = current
                .children
                .iter()
                .position(|child| child.name == *head && child.spec.is_none())
                .unwrap_or_else(|| {
                    let mut node = TreeNode::branch(*head);
                    node.path = format!("{}/{}", current_path, head);
                    node.expanded = selected;
                    current.children.push(node);
                    current.children.len() - 1
                });
            let child = current.children.get_mut(index).expect("node inserted");
            child.expanded |= selected;
            insert_segments(
                child,
                tail,
                item,
                selected,
                format!("{}/{}", current_path, head),
            );
        }
    }
}

pub fn flatten_tree(node: &TreeNode) -> Vec<TreeRow> {
    let mut lines = Vec::new();
    walk(node, 0, &mut lines);
    lines
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow {
    pub depth: usize,
    pub label: String,
    pub badge: TreeBadge,
    pub path: String,
    pub has_children: bool,
    pub expanded: bool,
    pub spec: Option<String>,
}

fn walk(node: &TreeNode, depth: usize, lines: &mut Vec<TreeRow>) {
    lines.push(TreeRow {
        depth,
        label: node.name.clone(),
        badge: node.badge,
        path: node.path.clone(),
        has_children: !node.children.is_empty(),
        expanded: node.expanded,
        spec: node.spec.clone(),
    });
    if node.expanded {
        for child in &node.children {
            walk(child, depth + 1, lines);
        }
    }
}

pub fn toggle_path(node: &mut TreeNode, path: &str) -> bool {
    if node.path == path {
        node.expanded = !node.expanded;
        return true;
    }

    for child in &mut node.children {
        if toggle_path(child, path) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_by_view() {
        let items = vec![
            TargetItem::new("src/lib:lib").cached_success(),
            TargetItem::new("tests/api:test_api")
                .test_target()
                .cached_failure(),
            TargetItem::new("tests/ui:test_ui").test_target().running(),
        ];

        let tree = build_target_tree(&items, TargetTreeView::Tests, None);

        assert_eq!(tree.children.len(), 1);
    }

    #[test]
    fn sets_selected_badge() {
        let items = vec![TargetItem::new("tests/api:test_api").cached_success()];
        let tree = build_target_tree(&items, TargetTreeView::All, Some("tests/api:test_api"));

        assert!(flatten_tree(&tree).iter().any(|row| {
            row.label.contains("tests/api:test_api") && row.badge == TreeBadge::Selected
        }));
    }

    #[test]
    fn uses_unique_non_duplicated_directory_paths() {
        let items = vec![TargetItem::new("tests/api/test_users.py:tests").test_target()];
        let tree = build_target_tree(
            &items,
            TargetTreeView::All,
            Some("tests/api/test_users.py:tests"),
        );
        let rows = flatten_tree(&tree);

        assert!(rows.iter().any(|row| row.path == "tests"));
        assert!(rows.iter().any(|row| row.path == "tests/api"));
        assert!(rows.iter().any(|row| row.path == "tests/api/test_users.py"));
        assert!(!rows.iter().any(|row| row.path.starts_with("tests/tests")));
    }

    #[test]
    fn tests_view_uses_target_type_instead_of_name_heuristics() {
        let items = vec![
            TargetItem::new("src/test_helpers:lib"),
            TargetItem::new("src/api/spec.py:unit").test_target(),
        ];
        let tree = build_target_tree(&items, TargetTreeView::Tests, None);

        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].name, "src");
    }
}
