use crate::{
    domain::{run::TestRun, target::TargetItem},
    enums::{Mode, PaneType},
};

#[derive(Debug)]
pub struct AppState {
    focus_pane: PaneType,
    mode: Mode,
    targets: Vec<TargetItem>,
    runs: Vec<TestRun>,
}

impl Default for AppState {
    fn default() -> AppState {
        AppState {
            focus_pane: PaneType::Tests,
            mode: Mode::Normal,
            targets: Vec::from([TargetItem::default()]),
            runs: Vec::from([TestRun::default()]),
        }
    }
}
