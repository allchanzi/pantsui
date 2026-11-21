use crate::{
    app::Effect,
    entities::Panel,
    features::target::ui::TargetPanel,
    features::version::{data::sub_projects, ui::VersionPanel},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const HELP: &str = "Space menu • ? help • Tab/Shift+Tab switch panels • Ctrl+h/j/k/l move focus • 1..5 jump • q quit";

pub struct AppState {
    pub active_panel: Panel,
    pub version_panel: VersionPanel,
    pub target_panel: TargetPanel,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_panel: Panel::FileExplorer,
            version_panel: VersionPanel::new(false),
            target_panel: TargetPanel::new(),
        }
    }

    /// Sub-projects are derived from the targets, so they follow every target reload.
    pub fn sync_sub_projects(&mut self) {
        self.version_panel.sub_projects = sub_projects(
            self.target_panel
                .items
                .iter()
                .map(|target| (target.address_path(), target.is_test)),
        );
    }

    pub fn help() -> &'static str {
        HELP
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Effect {
        // Alt belongs to the terminal multiplexer.
        if key
            .modifiers
            .intersects(KeyModifiers::ALT | KeyModifiers::SUPER)
        {
            return Effect::None;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return self.move_focus_by_key(key.code);
        }

        match key.code {
            KeyCode::Char('?') => Effect::ToggleHelp,
            KeyCode::Char('q') => Effect::Quit,
            KeyCode::Tab => self.set_active_panel(self.active_panel.next()),
            KeyCode::BackTab => self.set_active_panel(self.active_panel.previous()),
            KeyCode::Char('1') => self.set_active_panel(Panel::Version),
            KeyCode::Char('2') => self.set_active_panel(Panel::Mode),
            KeyCode::Char('3') => self.set_active_panel(Panel::FileExplorer),
            KeyCode::Char('4') => self.set_active_panel(Panel::Main),
            KeyCode::Char('5') => self.set_active_panel(Panel::Runs),
            KeyCode::Char('h') | KeyCode::Left if self.active_panel == Panel::Version => {
                self.version_panel.previous_tab();
                Effect::None
            }
            KeyCode::Char('l') | KeyCode::Right if self.active_panel == Panel::Version => {
                self.version_panel.next_tab();
                Effect::None
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.set_active_panel(self.active_panel.move_left())
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.set_active_panel(self.active_panel.move_right())
            }
            _ => Effect::None,
        }
    }

    pub fn set_active_panel(&mut self, panel: Panel) -> Effect {
        self.active_panel = panel;
        self.version_panel.active(panel == Panel::Version);
        self.target_panel.sync_selection();
        Effect::Message(format!("Focus: {}", panel.title()))
    }

    fn move_focus_by_key(&mut self, key: KeyCode) -> Effect {
        let next = match key {
            KeyCode::Char('h') | KeyCode::Left => self.active_panel.move_left(),
            KeyCode::Char('l') | KeyCode::Right => self.active_panel.move_right(),
            KeyCode::Char('k') | KeyCode::Up => self.active_panel.move_up(),
            KeyCode::Char('j') | KeyCode::Down => self.active_panel.move_down(),
            _ => return Effect::None,
        };
        self.set_active_panel(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn tab_and_backtab_switch_panels_in_both_directions() {
        let mut state = AppState::new();
        state.set_active_panel(Panel::Version);

        state.handle_key(key(KeyCode::Tab));
        assert_eq!(state.active_panel, Panel::Mode);
        state.handle_key(key(KeyCode::BackTab));
        assert_eq!(state.active_panel, Panel::Version);
        state.handle_key(key(KeyCode::BackTab));
        assert_eq!(state.active_panel, Panel::Runs);
    }

    #[test]
    fn h_and_l_switch_sections_in_the_version_panel_and_ctrl_leaves_it() {
        use crate::features::version::ui::SubTab;
        let mut state = AppState::new();
        state.set_active_panel(Panel::Version);
        state.handle_key(key(KeyCode::Char('l')));
        assert_eq!(state.active_panel, Panel::Version);
        assert_eq!(state.version_panel.active_tab, SubTab::Dependencies);
        state.handle_key(key(KeyCode::Char('h')));
        assert_eq!(state.version_panel.active_tab, SubTab::Version);
        state.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL));
        assert_eq!(state.active_panel, Panel::Main);
    }

    #[test]
    fn sub_projects_come_from_the_loaded_targets() {
        use crate::features::target::model::TargetItem;
        let mut state = AppState::new();
        state.target_panel = TargetPanel::with_items(vec![
            TargetItem::new("projects/a/x.py:lib"),
            TargetItem::new("projects/b/test_y.py:tests").test_target(),
        ]);
        state.sync_sub_projects();
        let summary = state
            .version_panel
            .sub_projects
            .iter()
            .map(|project| (project.path.as_str(), project.targets, project.tests))
            .collect::<Vec<_>>();
        assert_eq!(summary, [("projects/a", 1, 0), ("projects/b", 1, 1)]);
    }

    #[test]
    fn number_keys_jump_directly_to_panels() {
        let mut state = AppState::new();

        state.handle_key(key(KeyCode::Char('3')));
        assert_eq!(state.active_panel, Panel::FileExplorer);
        state.handle_key(key(KeyCode::Char('5')));
        assert_eq!(state.active_panel, Panel::Runs);
    }

    #[test]
    fn control_navigation_moves_focus_without_using_local_list_keys() {
        let mut state = AppState::new();
        state.set_active_panel(Panel::FileExplorer);

        state.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL));

        assert_eq!(state.active_panel, Panel::Runs);
    }
}
