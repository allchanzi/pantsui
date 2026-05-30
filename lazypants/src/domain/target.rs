use crate::enums::TargetStatus;

#[derive(Debug)]
pub struct TargetItem {
    name: String,
    path: String,
    status: TargetStatus,
}

impl Default for TargetItem {
    fn default() -> Self {
        TargetItem {
            name: String::from("Test Item 1"),
            path: String::from("test/path"),
            status: TargetStatus::Ready,
        }
    }
}
