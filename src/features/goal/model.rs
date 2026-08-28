#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalKind {
    Data,
    Action,
    Mutating,
    Interactive,
}

impl GoalKind {
    pub fn for_name(name: &str) -> Self {
        match name {
            "list" | "peek" | "dependencies" | "dependents" | "filedeps" | "paths" | "roots"
            | "count-loc" | "version" | "help" | "help-all" => Self::Data,
            "fmt"
            | "fix"
            | "tailor"
            | "update-build-files"
            | "generate-lockfiles"
            | "publish"
            | "migrate-call-by-name" => Self::Mutating,
            "run" | "repl" | "experimental-explorer" | "experimental-deploy" => Self::Interactive,
            _ => Self::Action,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Data => "data",
            Self::Action => "action",
            Self::Mutating => "mutating",
            Self::Interactive => "interactive",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_and_unknown_goals() {
        assert_eq!(GoalKind::for_name("list"), GoalKind::Data);
        assert_eq!(GoalKind::for_name("fmt"), GoalKind::Mutating);
        assert_eq!(GoalKind::for_name("run"), GoalKind::Interactive);
        assert_eq!(GoalKind::for_name("future-plugin-goal"), GoalKind::Action);
    }
}
