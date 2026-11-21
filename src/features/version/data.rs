use std::collections::BTreeMap;

/// A part of the repository, with what it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubProject {
    pub path: String,
    pub targets: usize,
    pub tests: usize,
}

/// The folder of a Pants address path. `a/b/file.py` lives in `a/b`, the root is empty.
fn directory(path: &str) -> Vec<&str> {
    if path == "." {
        return Vec::new();
    }
    let mut parts = path.split('/').collect::<Vec<_>>();
    if parts.last().is_some_and(|last| last.contains('.')) {
        parts.pop();
    }
    parts
}

/// Groups targets into sub-projects, from `(address path, is test)` pairs.
///
/// Pants has no such concept and many repositories have a single source root, so the
/// folders decide: each top-level folder splits into the folders below the part all of
/// its targets share. `projects/{a,b}` gives two groups; `src/python/{a,b}` does too.
pub fn sub_projects<'a>(targets: impl Iterator<Item = (&'a str, bool)>) -> Vec<SubProject> {
    let targets = targets
        .map(|(path, is_test)| (directory(path), is_test))
        .collect::<Vec<_>>();

    let mut shared: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (directory, _) in &targets {
        let Some(top) = directory.first() else {
            continue;
        };
        shared
            .entry(top)
            .and_modify(|prefix| {
                let common = prefix
                    .iter()
                    .zip(directory)
                    .take_while(|(left, right)| left == right)
                    .count();
                prefix.truncate(common);
            })
            .or_insert_with(|| directory.clone());
    }

    let mut groups: BTreeMap<String, SubProject> = BTreeMap::new();
    for (directory, is_test) in &targets {
        let path = match directory.first() {
            None => String::from("."),
            Some(top) => {
                let depth = (shared[top].len() + 1).min(directory.len());
                directory[..depth].join("/")
            }
        };
        let group = groups.entry(path.clone()).or_insert(SubProject {
            path,
            targets: 0,
            tests: 0,
        });
        group.targets += 1;
        group.tests += usize::from(*is_test);
    }
    groups.into_values().collect()
}

/// The addresses a selection depends on. A folder's targets also depend on each other,
/// which would bury what the folder needs from elsewhere, so those are left out.
pub fn outside_scope(scope: &str, dependencies: Vec<String>) -> Vec<String> {
    match scope.strip_suffix("::") {
        Some(folder) => dependencies
            .into_iter()
            .filter(|address| {
                !address.starts_with(&format!("{folder}/"))
                    && !address.starts_with(&format!("{folder}:"))
            })
            .collect(),
        None => dependencies
            .into_iter()
            .filter(|address| address != scope)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(path: &str, targets: usize, tests: usize) -> SubProject {
        SubProject {
            path: path.into(),
            targets,
            tests,
        }
    }

    #[test]
    fn a_top_folder_splits_into_its_projects_and_counts_tests() {
        let groups = sub_projects(
            [
                ("projects/a", false),
                ("projects/a/x/test_y.py", true),
                ("projects/b/z.py", false),
                ("shared", false),
                ("shared/lib/u.py", false),
                (".", false),
            ]
            .into_iter(),
        );
        assert_eq!(
            groups,
            [
                group(".", 1, 0),
                group("projects/a", 2, 1),
                group("projects/b", 1, 0),
                group("shared", 1, 0),
                group("shared/lib", 1, 0),
            ]
        );
    }

    #[test]
    fn a_shared_path_is_skipped_so_the_real_projects_show() {
        let groups = sub_projects(
            [
                ("src/python/a/m.py", false),
                ("src/python/b/deep/n.py", false),
                ("docs", false),
                ("docs", false),
            ]
            .into_iter(),
        );
        assert_eq!(
            groups,
            [
                group("docs", 2, 0),
                group("src/python/a", 1, 0),
                group("src/python/b", 1, 0),
            ]
        );
    }

    #[test]
    fn no_targets_means_no_groups() {
        assert!(sub_projects(std::iter::empty()).is_empty());
    }

    #[test]
    fn a_folder_lists_only_what_it_needs_from_elsewhere() {
        let dependencies = [
            "3rdparty/python:default#django",
            "projects/a/x.py:lib",
            "projects/a:a",
            "projects/ab/y.py:lib",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(
            outside_scope("projects/a::", dependencies),
            ["3rdparty/python:default#django", "projects/ab/y.py:lib"]
        );
    }

    #[test]
    fn a_target_keeps_its_folder_neighbours() {
        let dependencies = ["a/x.py:lib", "a/y.py:lib"].map(String::from).to_vec();
        assert_eq!(outside_scope("a/x.py:lib", dependencies), ["a/y.py:lib"]);
    }
}
