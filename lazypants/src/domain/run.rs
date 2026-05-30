use std::time::Duration;

use crate::domain::target::TargetItem;
use crate::enums::TestRunStatus;

#[derive(Debug)]
pub struct TestRun {
    name: String,
    status: TestRunStatus,
    target: TargetItem,
    duration: Duration,
}

impl Default for TestRun {
    fn default() -> Self {
        TestRun {
            name: String::from("Test 1"),
            status: TestRunStatus::Running,
            target: TargetItem::default(),
            duration: Duration::default(),
        }
    }
}
