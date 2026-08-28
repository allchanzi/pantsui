use crate::features::target::{
    data::{parse_target_specs_response, sort_targets},
    model::{TargetCacheState, TargetItem},
};
use crate::shared::pants::{cache::PantsCache, commands, errors::PantsResult, models::PantsGoal};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub struct PantsClient {
    goals: Vec<PantsGoal>,
    cache: PantsCache,
    root: PathBuf,
}

impl Clone for PantsClient {
    fn clone(&self) -> Self {
        Self {
            goals: self.goals.clone(),
            cache: self.cache.clone(),
            root: self.root.clone(),
        }
    }
}

impl PantsClient {
    pub fn new(root: PathBuf) -> Self {
        let goals =
            commands::pants_help_all(&root).map_or_else(|_| Vec::new(), |info| info.goals());
        let cache = PantsCache::new(
            root.join(".pantsui-cache"),
            std::time::Duration::from_secs(600),
        );
        cache.ensure_gitignore_entry();
        Self { goals, cache, root }
    }

    pub fn load_targets(&self) -> PantsResult<Vec<TargetItem>> {
        if let Some(targets) = self.cache.load_targets() {
            return Ok(self.decorate_target_status(targets));
        }

        let test_specs =
            parse_target_specs_response(&commands::pants_list_python_tests(&self.root)?)
                .into_iter()
                .map(|target| target.spec)
                .collect::<BTreeSet<_>>();
        let mut targets = parse_target_specs_response(&commands::pants_list_targets(&self.root)?)
            .into_iter()
            .map(|mut target| {
                target.is_test = test_specs.contains(&target.spec);
                target
            })
            .collect::<Vec<_>>();

        // A backend may omit generated test addresses from an unfiltered listing.
        // Keep the filtered result authoritative so those tests still remain usable.
        let known_specs = targets
            .iter()
            .map(|target| target.spec.clone())
            .collect::<BTreeSet<_>>();
        targets.extend(
            test_specs
                .difference(&known_specs)
                .map(|spec| TargetItem::new(spec.clone()).test_target()),
        );
        let targets = sort_targets(targets);
        self.cache.store_targets(&targets);
        Ok(self.decorate_target_status(targets))
    }

    pub fn goals(&self) -> &[PantsGoal] {
        &self.goals
    }

    pub fn version_string(&self) -> PantsResult<String> {
        commands::pants_version(&self.root)
    }

    pub fn refresh_targets(&self) -> PantsResult<Vec<TargetItem>> {
        self.cache.clear();
        self.load_targets()
    }

    pub fn run_python_test_target_with_output(
        &self,
        spec: &str,
    ) -> PantsResult<(bool, String, String)> {
        let output = commands::pants_test_target_output(&self.root, spec)?;
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        self.cache.store_target_status(
            spec,
            success,
            output.status.code(),
            stdout.clone(),
            stderr.clone(),
        );
        Ok((success, stdout, stderr))
    }

    pub fn affected_python_tests(&self, spec: &str) -> PantsResult<Vec<TargetItem>> {
        let affected_specs =
            parse_target_specs_response(&commands::pants_affected_python_tests(&self.root, spec)?)
                .into_iter()
                .map(|target| target.spec)
                .collect::<BTreeSet<_>>();
        Ok(self
            .load_targets()?
            .into_iter()
            .filter(|target| target.is_test && affected_specs.contains(&target.spec))
            .collect())
    }

    pub fn run_goal_with_output(
        &self,
        goal: &str,
        specs: &[String],
    ) -> PantsResult<(bool, String, String)> {
        let output = commands::pants_goal_output(&self.root, goal, specs)?;
        Ok((
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    pub fn run_interactive_goal(&self, goal: &str, specs: &[String]) -> PantsResult<bool> {
        commands::pants_interactive_goal(&self.root, goal, specs)
    }

    fn decorate_target_status(&self, mut targets: Vec<TargetItem>) -> Vec<TargetItem> {
        for target in &mut targets {
            if let Some((success, exit_code, stdout, stderr)) =
                self.cache.load_target_run_log(&target.spec)
            {
                *target = target.clone().run_finished(success, exit_code);
                target.description = Some(format!(
                    "{}{}",
                    if stdout.is_empty() {
                        ""
                    } else {
                        "stdout cached"
                    },
                    if stderr.is_empty() {
                        ""
                    } else {
                        " stderr cached"
                    }
                ));
            } else if target.is_test {
                target.cache_state = TargetCacheState::NeedsRun;
            }
        }
        targets
    }
}
