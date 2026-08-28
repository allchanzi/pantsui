#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunItem {
    pub spec: String,
    pub status: RunStatus,
    pub output: String,
}

impl RunItem {
    pub fn running(spec: impl Into<String>) -> Self {
        Self {
            spec: spec.into(),
            status: RunStatus::Running,
            output: String::new(),
        }
    }

    pub fn finish(&mut self, success: bool, stdout: String, stderr: String) {
        self.status = if success {
            RunStatus::Passed
        } else {
            RunStatus::Failed
        };
        self.output = format!("{stdout}\n{stderr}").trim().to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finishing_a_run_combines_output_and_sets_status() {
        let mut run = RunItem::running("tests/api:test");

        run.finish(false, String::from("out"), String::from("err"));

        assert_eq!(run.status, RunStatus::Failed);
        assert_eq!(run.output, "out\nerr");
    }
}
