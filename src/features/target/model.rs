use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetCacheState {
    Unknown,
    CachedSuccess,
    CachedFailure,
    NeedsRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetRunState {
    Idle,
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetStatus {
    Unknown,
    Loaded,
    Selected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetItem {
    pub spec: String,
    pub description: Option<String>,
    pub status: TargetStatus,
    pub cache_state: TargetCacheState,
    pub run_state: TargetRunState,
    pub last_exit_code: Option<i32>,
}

impl TargetItem {
    pub fn new(spec: impl Into<String>) -> Self {
        Self {
            spec: spec.into(),
            description: None,
            status: TargetStatus::Unknown,
            cache_state: TargetCacheState::Unknown,
            run_state: TargetRunState::Idle,
            last_exit_code: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn selected(mut self) -> Self {
        self.status = TargetStatus::Selected;
        self
    }

    pub fn cached_success(mut self) -> Self {
        self.cache_state = TargetCacheState::CachedSuccess;
        self
    }

    pub fn cached_failure(mut self) -> Self {
        self.cache_state = TargetCacheState::CachedFailure;
        self
    }

    pub fn needs_run(mut self) -> Self {
        self.cache_state = TargetCacheState::NeedsRun;
        self
    }

    pub fn running(mut self) -> Self {
        self.run_state = TargetRunState::Running;
        self
    }

    pub fn run_finished(mut self, success: bool, exit_code: Option<i32>) -> Self {
        self.run_state = TargetRunState::Idle;
        self.last_exit_code = exit_code;
        self.cache_state = if success {
            TargetCacheState::CachedSuccess
        } else {
            TargetCacheState::CachedFailure
        };
        self
    }

    pub fn display_label(&self) -> String {
        match &self.description {
            Some(description) if !description.is_empty() => {
                format!("{} — {}", self.spec, description)
            }
            _ => self.spec.clone(),
        }
    }
}
