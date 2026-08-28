use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::features::target::model::TargetItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    created_at: u64,
    targets: Vec<TargetItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TargetStatusEntry {
    spec: String,
    success: bool,
    exit_code: Option<i32>,
    updated_at: u64,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone)]
pub struct PantsCache {
    root: PathBuf,
    ttl: Duration,
}

impl PantsCache {
    pub fn new(root: impl Into<PathBuf>, ttl: Duration) -> Self {
        Self {
            root: root.into(),
            ttl,
        }
    }

    pub fn ensure_gitignore_entry(&self) {
        if self.root.file_name().and_then(|name| name.to_str()) != Some(".pantsui-cache") {
            return;
        }
        let Some(repo_root) = self.root.parent() else {
            return;
        };
        let gitignore_path = repo_root.join(".gitignore");
        let entry = ".pantsui-cache/";

        let current = fs::read_to_string(&gitignore_path).unwrap_or_default();
        if current.lines().any(|line| line.trim() == entry) {
            return;
        }

        let mut updated = current;
        if !updated.is_empty() && !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(entry);
        updated.push('\n');

        let _ = fs::write(gitignore_path, updated);
    }

    pub fn load_targets(&self) -> Option<Vec<TargetItem>> {
        let path = self.targets_cache_path();
        let data = fs::read_to_string(path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&data).ok()?;
        if self.is_expired(entry.created_at) {
            return None;
        }
        Some(entry.targets)
    }

    pub fn store_targets(&self, targets: &[TargetItem]) {
        let _ = fs::create_dir_all(&self.root);
        self.ensure_gitignore_entry();
        let entry = CacheEntry {
            created_at: now_secs(),
            targets: targets.to_vec(),
        };
        let _ = fs::write(
            self.targets_cache_path(),
            serde_json::to_string_pretty(&entry).unwrap_or_default(),
        );
    }

    pub fn clear(&self) {
        let _ = fs::remove_file(self.targets_cache_path());
    }

    #[cfg(test)]
    pub fn load_target_status(&self, spec: &str) -> Option<(bool, Option<i32>)> {
        let path = self.target_status_path(spec);
        let data = fs::read_to_string(path).ok()?;
        let entry: TargetStatusEntry = serde_json::from_str(&data).ok()?;
        Some((entry.success, entry.exit_code))
    }

    pub fn load_target_run_log(&self, spec: &str) -> Option<(bool, Option<i32>, String, String)> {
        let path = self.target_status_path(spec);
        let data = fs::read_to_string(path).ok()?;
        let entry: TargetStatusEntry = serde_json::from_str(&data).ok()?;
        Some((entry.success, entry.exit_code, entry.stdout, entry.stderr))
    }

    pub fn store_target_status(
        &self,
        spec: &str,
        success: bool,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    ) {
        let _ = fs::create_dir_all(&self.root);
        let _ = fs::create_dir_all(self.root.join("target-status"));
        self.ensure_gitignore_entry();
        let entry = TargetStatusEntry {
            spec: spec.to_string(),
            success,
            exit_code,
            updated_at: now_secs(),
            stdout,
            stderr,
        };
        let _ = fs::write(
            self.target_status_path(spec),
            serde_json::to_string_pretty(&entry).unwrap_or_default(),
        );
    }

    fn targets_cache_path(&self) -> PathBuf {
        // v2 stores both all targets and their test classification. Keeping the
        // version in the filename avoids silently reading the older tests-only cache.
        self.root.join("targets-v2.json")
    }

    fn target_status_path(&self, spec: &str) -> PathBuf {
        self.root
            .join("target-status")
            .join(format!("{}.json", sanitize_spec(spec)))
    }

    fn is_expired(&self, created_at: u64) -> bool {
        now_secs().saturating_sub(created_at) > self.ttl.as_secs()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn sanitize_spec(spec: &str) -> String {
    spec.chars()
        .map(|ch| match ch {
            '/' | ':' | ' ' => '_',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_cache() -> PantsCache {
        let root = std::env::temp_dir().join(format!(
            "pantsui-cache-test-{}-{}-{:?}",
            now_secs(),
            std::process::id(),
            std::thread::current().id()
        ));
        PantsCache::new(root, Duration::from_secs(3600))
    }

    #[test]
    fn stores_and_loads_targets() {
        let cache = temp_cache();
        let targets = vec![TargetItem::new("src/app:test")];

        cache.store_targets(&targets);

        assert_eq!(cache.load_targets(), Some(targets));
    }

    #[test]
    fn clear_removes_cache_file() {
        let cache = temp_cache();
        cache.store_targets(&[TargetItem::new("src/app:test")]);

        cache.clear();

        assert_eq!(cache.load_targets(), None);
    }

    #[test]
    fn stores_and_loads_target_status() {
        let cache = temp_cache();
        cache.store_target_status(
            "tests/api:test_api",
            true,
            Some(0),
            String::from("ok"),
            String::new(),
        );

        assert_eq!(
            cache.load_target_status("tests/api:test_api"),
            Some((true, Some(0)))
        );
    }

    #[test]
    fn stores_and_loads_run_log() {
        let cache = temp_cache();
        cache.store_target_status(
            "tests/api:test_api",
            true,
            Some(0),
            String::from("ok"),
            String::from("warn"),
        );

        assert_eq!(
            cache.load_target_run_log("tests/api:test_api"),
            Some((true, Some(0), String::from("ok"), String::from("warn")))
        );
    }
}
