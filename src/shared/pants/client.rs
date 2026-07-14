use crate::shared::pants::{commands, models::PantsGoal};

pub struct PantsClient {
    goals: Vec<PantsGoal>,
}

impl PantsClient {
    pub fn new() -> Self {
        let goals = commands::pants_help_all().unwrap().goals();
        Self { goals }
    }
}
