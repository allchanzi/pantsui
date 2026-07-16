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
            Self::Tests => item.spec.contains("test"),
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
            expanded: true,
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

pub fn build_target_tree(items: &[TargetItem], view: TargetTreeView, selected: Option<&str>) -> TreeNode {
    let mut root = TreeNode::branch(view.title());
    let mut grouped: BTreeMap<&str, Vec<&TargetItem>> = BTreeMap::new();

    for item in items.iter().filter(|item| view.matches(item)) {
        let group = item
            .spec
            .split(':')
            .next()
            .unwrap_or(&item.spec)
            .split('/')
            .next()
            .unwrap_or(&item.spec);
        grouped.entry(group).or_default().push(item);
    }

    for (group, grouped_items) in grouped {
        let mut group_node = TreeNode::branch(group);
        for item in grouped_items {
            insert_segments(
                &mut group_node,
                &path_segments(&item.spec),
                item,
                selected == Some(item.spec.as_str()),
                group.to_string(),
            );
        }
        root.children.push(group_node);
    }

    root
}

fn path_segments(spec: &str) -> Vec<&str> {
    spec.split(':')
        .next()
        .unwrap_or(spec)
        .split('/')
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
        [] => current.children.push(TreeNode::leaf(item, selected, current_path)),
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
                    current.children.push(node);
                    current.children.len() - 1
                });
            let child = current.children.get_mut(index).expect("node inserted");
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
    pub spec: Option<String>,
}

fn walk(node: &TreeNode, depth: usize, lines: &mut Vec<TreeRow>) {
    lines.push(TreeRow {
        depth,
        label: node.name.clone(),
        badge: node.badge,
        path: node.path.clone(),
        has_children: !node.children.is_empty(),
        spec: node.spec.clone(),
    });
    if node.expanded {
        for child in &node.children {
            walk(child, depth + 1, lines);
        }
    }
}

pub fn set_expanded(node: &mut TreeNode, expanded: bool) {
    node.expanded = expanded;
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

pub fn find_path<'a>(node: &'a TreeNode, path: &str) -> Option<&'a TreeNode> {
    if node.path == path {
        return Some(node);
    }

    for child in &node.children {
        if let Some(found) = find_path(child, path) {
            return Some(found);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_by_view() {
        let items = vec![
            TargetItem::new("src/lib:lib").cached_success(),
            TargetItem::new("tests/api:test_api").cached_failure(),
            TargetItem::new("tests/ui:test_ui").running(),
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
}
