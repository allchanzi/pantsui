#[derive(Debug)]
pub enum Mode {
    Normal,
    Search,
    Running,
}

#[derive(Debug)]
pub enum PaneType {
    Tests,
    Target,
    Runs,
    Context,
}

#[derive(Debug)]
pub enum TestResult {
    Passed,
    Failed,
    Cached,
    Running,
    Unknown,
}

#[derive(Debug)]
pub enum TargetStatus {
    Ready,
}

#[derive(Debug)]
pub enum TestRunStatus {
    Finished,
    Running,
}

#[derive(Debug)]
