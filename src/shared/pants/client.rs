use crate::shared::pants::{
    cache::PantsCache,
    commands,
    errors::PantsResult,
    models::PantsGoal,
};
use crate::features::target::{
    data::{parse_target_specs_response, sort_targets},
    model::{TargetCacheState, TargetItem},
};

pub struct PantsClient {
    goals: Vec<PantsGoal>,
    cache: PantsCache,
}

impl Clone for PantsClient {
    fn clone(&self) -> Self {
        Self {
            goals: self.goals.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl PantsClient {
    pub fn new() -> Self {
        let goals = commands::pants_help_all().map_or_else(|_| Vec::new(), |info| info.goals());
        Self {
            goals,
            cache: PantsCache::default(),
        }
    }

    pub fn load_python_test_targets(&self) -> PantsResult<Vec<TargetItem>> {
        if let Some(targets) = self.cache.load_targets() {
            return Ok(self.decorate_target_status(targets));
        }

        let targets = sort_targets(parse_target_specs_response(
            &commands::pants_list_python_tests()?,
        ));
        self.cache.store_targets(&targets);
        Ok(self.decorate_target_status(targets))
    }

    pub fn goals(&self) -> &[PantsGoal] {
        &self.goals
    }

    pub fn refresh_python_test_targets(&self) -> PantsResult<Vec<TargetItem>> {
        self.cache.clear();
        self.load_python_test_targets()
    }

    pub fn run_python_test_target(&self, spec: &str) -> PantsResult<bool> {
        let (success, _, _) = self.run_python_test_target_with_output(spec)?;
        Ok(success)
    }

    pub fn run_python_test_target_with_output(
        &self,
        spec: &str,
    ) -> PantsResult<(bool, String, String)> {
        let output = commands::pants_test_target_output(spec)?;
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

    fn decorate_target_status(&self, mut targets: Vec<TargetItem>) -> Vec<TargetItem> {
        for target in &mut targets {
            if let Some((success, exit_code, stdout, stderr)) = self.cache.load_target_run_log(&target.spec) {
                *target = target.clone().run_finished(success, exit_code);
                target.description = Some(format!(
                    "{}{}",
                    if stdout.is_empty() { "" } else { "stdout cached" },
                    if stderr.is_empty() { "" } else { " stderr cached" }
                ));
            } else {
                target.cache_state = TargetCacheState::NeedsRun;
            }
        }
        targets
    }
}
