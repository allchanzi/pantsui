use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use whichkey::{Outcome, WhichKey};
use std::path::PathBuf;
use std::sync::mpsc;

use crate::{
    error::AppResult,
    features::run::model::RunItem,
    shared::{pants::client::PantsClient, ui::UiTheme},
    state::AppState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    None,
    Quit,
    Message(String),
    ToggleHelp,
    RunInteractive { goal: String, specs: Vec<String> },
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    TargetRunStarted {
        spec: String,
    },
    TargetRunFinished {
        spec: String,
        success: bool,
        stdout: String,
        stderr: String,
    },
    TargetsRefreshed,
    AffectedTestsLoaded {
        scope: String,
        result: Result<Vec<crate::features::target::model::TargetItem>, String>,
    },
    GoalRunFinished {
        command: String,
        success: bool,
        stdout: String,
        stderr: String,
    },
}

pub struct App {
    pub state: AppState,
    pub theme: UiTheme,
    pub message: String,
    pub show_help: bool,
    pub last_run_output: String,
    pub run_history: Vec<RunItem>,
    pub affected_scope: Option<String>,
    pub affected_tests: Vec<crate::features::target::model::TargetItem>,
    pub affected_loading: bool,
    pub filter_mode: bool,
    pub which_key: WhichKey<&'static str>,
    pub project: String,
    pub detail_scroll: u16,
    pub runs_scroll: u16,
    pub goal_selected: usize,
    pants_client: PantsClient,
    event_tx: mpsc::Sender<AppEvent>,
    event_rx: mpsc::Receiver<AppEvent>,
    frame_tick: usize,
    pending_goal: Option<(String, Vec<String>)>,
}

impl App {
    pub fn new(root: PathBuf) -> Self {
        let project = root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let pants_client = PantsClient::new(root);
        let (event_tx, event_rx) = mpsc::channel();
        let mut state = AppState::new();
        if let Ok(version_string) = pants_client.version_string() {
            state.version_panel.version_string = version_string;
        }
        if let Ok(targets) = pants_client.load_targets() {
            state.target_panel = crate::features::target::ui::TargetPanel::with_items(targets);
        }

        Self {
            state,
            theme: UiTheme::default(),
            message: AppState::help().into(),
            show_help: false,
            pants_client,
            event_tx,
            event_rx,
            frame_tick: 0,
            last_run_output: String::new(),
            run_history: Vec::new(),
            affected_scope: None,
            affected_tests: Vec::new(),
            affected_loading: false,
            filter_mode: false,
            which_key: crate::leader::which_key(),
            project,
            detail_scroll: 0,
            runs_scroll: 0,
            goal_selected: 0,
            pending_goal: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppResult<Effect> {
        if self.pending_goal.is_some() {
            return Ok(match key.code {
                crossterm::event::KeyCode::Char('y') => {
                    let (goal, specs) = self.pending_goal.take().expect("pending goal exists");
                    self.start_goal(goal, specs)
                }
                crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Esc => {
                    self.pending_goal = None;
                    self.message = String::from("Goal cancelled");
                    Effect::None
                }
                _ => Effect::None,
            });
        }
        if self.filter_mode {
            match key.code {
                crossterm::event::KeyCode::Enter | crossterm::event::KeyCode::Esc => {
                    self.filter_mode = false;
                    self.message = if self.state.target_panel.filter.is_empty() {
                        String::from("Target filter cleared")
                    } else {
                        format!("Target filter: {}", self.state.target_panel.filter)
                    };
                }
                crossterm::event::KeyCode::Backspace => {
                    let mut filter = self.state.target_panel.filter.clone();
                    filter.pop();
                    self.state.target_panel.set_filter(filter);
                }
                crossterm::event::KeyCode::Char(character) => {
                    let mut filter = self.state.target_panel.filter.clone();
                    filter.push(character);
                    self.state.target_panel.set_filter(filter);
                }
                _ => {}
            }
            return Ok(Effect::None);
        }
        if self.which_key.is_open() {
            return Ok(self.handle_leader_key(key));
        }
        if key.code == KeyCode::Char(' ') && key.modifiers == KeyModifiers::NONE {
            self.which_key.open();
            return Ok(Effect::None);
        }
        if key.modifiers.intersects(
            crossterm::event::KeyModifiers::ALT | crossterm::event::KeyModifiers::CONTROL,
        ) {
            let effect = self.state.handle_key(key);
            return self.apply_local_effect(effect);
        }
        if self.state.active_panel == crate::entities::Panel::FileExplorer
            && let crossterm::event::KeyCode::Char('/') = key.code
        {
            self.filter_mode = true;
            self.state.target_panel.set_filter(String::new());
            self.detail_scroll = 0;
            self.message = String::from("Filter targets: type text, Enter to apply, Esc to finish");
            return Ok(Effect::None);
        }
        if let crossterm::event::KeyCode::Char('d') = key.code {
            self.load_affected_tests_async();
            return Ok(Effect::None);
        }
        if let crossterm::event::KeyCode::Char('T') = key.code {
            let specs = self
                .affected_tests
                .iter()
                .map(|target| target.spec.clone())
                .collect::<Vec<_>>();
            if specs.is_empty() {
                self.message = String::from("No affected tests loaded; press d first");
            } else {
                for spec in specs {
                    self.mark_running(&spec);
                    self.run_target_async(spec);
                }
                self.message = String::from("Running affected tests");
            }
            return Ok(Effect::None);
        }
        if self.state.active_panel == crate::entities::Panel::Mode
            && matches!(
                key.code,
                crossterm::event::KeyCode::Enter | crossterm::event::KeyCode::Char('g')
            )
        {
            return Ok(self.run_selected_goal());
        }
        if let crossterm::event::KeyCode::Char('R') = key.code {
            self.refresh_targets_async();
            return Ok(Effect::None);
        }
        if let crossterm::event::KeyCode::Char('v') = key.code {
            self.state.target_panel.next_view();
            self.detail_scroll = 0;
            self.message = format!("Targets view: {}", self.state.target_panel.view.title());
            return Ok(Effect::None);
        }
        if let crossterm::event::KeyCode::Char('t') = key.code {
            if let Some(row) = self.state.target_panel.current_row().cloned() {
                let specs = self.state.target_panel.scoped_test_specs();
                if specs.is_empty() {
                    self.message = format!("No tests under {}", row.path);
                } else {
                    let count = specs.len();
                    for spec in specs {
                        self.mark_running(&spec);
                        self.run_target_async(spec);
                    }
                    self.message = if count == 1 {
                        format!("Running test for {}", row.label)
                    } else {
                        format!("Running {count} tests under {}", row.path)
                    };
                }
            }
            return Ok(Effect::None);
        }
        if let crossterm::event::KeyCode::Char('A') = key.code {
            for spec in self.visible_target_specs() {
                self.mark_running(&spec);
                self.run_target_async(spec);
            }
            self.message = String::from("Running all visible targets");
            return Ok(Effect::None);
        }
        if self.state.active_panel == crate::entities::Panel::FileExplorer
            && let crossterm::event::KeyCode::Enter = key.code
        {
            self.state.target_panel.toggle_tree();
            self.message = String::from("Toggled target tree");
            return Ok(Effect::None);
        }
        if self.state.active_panel == crate::entities::Panel::FileExplorer {
            match key.code {
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                    self.state.target_panel.move_cursor_up();
                    self.detail_scroll = 0;
                    return Ok(Effect::None);
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                    self.state.target_panel.move_cursor_down();
                    self.detail_scroll = 0;
                    return Ok(Effect::None);
                }
                _ => {}
            }
        }
        if self.state.active_panel == crate::entities::Panel::Main {
            match key.code {
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                    return Ok(Effect::None);
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                    self.detail_scroll = self.detail_scroll.saturating_add(1);
                    return Ok(Effect::None);
                }
                _ => {}
            }
        }
        if self.state.active_panel == crate::entities::Panel::Mode {
            match key.code {
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                    self.goal_selected = self.goal_selected.saturating_sub(1);
                    return Ok(Effect::None);
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                    self.goal_selected = self
                        .goal_selected
                        .saturating_add(1)
                        .min(self.pants_client.goals().len().saturating_sub(1));
                    return Ok(Effect::None);
                }
                _ => {}
            }
        }
        if self.state.active_panel == crate::entities::Panel::Runs {
            match key.code {
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                    self.runs_scroll = self.runs_scroll.saturating_sub(1);
                    return Ok(Effect::None);
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                    self.runs_scroll = self.runs_scroll.saturating_add(1);
                    return Ok(Effect::None);
                }
                _ => {}
            }
        }
        let effect = self.state.handle_key(key);
        self.apply_local_effect(effect)
    }

    pub fn has_pending_goal(&self) -> bool {
        self.pending_goal.is_some()
    }

    fn handle_leader_key(&mut self, key: KeyEvent) -> Effect {
        let outcome = match key.code {
            KeyCode::Esc => {
                self.which_key.cancel();
                return Effect::None;
            }
            KeyCode::Backspace => self.which_key.back(),
            KeyCode::Char(character) => self.which_key.press(character),
            _ => {
                self.which_key.cancel();
                return Effect::None;
            }
        };
        match outcome {
            Outcome::Action(action) => self.run_leader_action(action),
            Outcome::Unknown => {
                self.message = String::from("Unknown Space key");
                Effect::None
            }
            Outcome::Pending | Outcome::Cancelled => Effect::None,
        }
    }

    fn press(&mut self, character: char) -> Effect {
        self.handle_key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE))
            .unwrap_or(Effect::None)
    }

    fn run_leader_action(&mut self, action: &'static str) -> Effect {
        use crate::entities::Panel;
        match action {
            "help" => Effect::ToggleHelp,
            "quit" => Effect::Quit,
            "filter" => {
                self.state.set_active_panel(Panel::FileExplorer);
                self.press('/')
            }
            "view" => self.press('v'),
            "refresh" => self.press('R'),
            "run_tests" => self.press('t'),
            "run_all" => self.press('A'),
            "run_goal" => {
                self.state.set_active_panel(Panel::Mode);
                self.press('g')
            }
            "affected_load" => self.press('d'),
            "affected_run" => self.press('T'),
            "panel1" => self.state.set_active_panel(Panel::Version),
            "panel2" => self.state.set_active_panel(Panel::Mode),
            "panel3" => self.state.set_active_panel(Panel::FileExplorer),
            "panel4" => self.state.set_active_panel(Panel::Main),
            "panel5" => self.state.set_active_panel(Panel::Runs),
            _ => Effect::None,
        }
    }

    pub fn drain_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::TargetRunStarted { spec } => {
                    self.apply_running_state(&spec);
                    self.message = format!("Running {}", spec);
                }
                AppEvent::TargetRunFinished {
                    spec,
                    success,
                    stdout,
                    stderr,
                } => {
                    self.last_run_output = format!("{}\n{}", stdout, stderr).trim().to_string();
                    if let Some(run) = self.run_history.iter_mut().rev().find(|run| {
                        run.spec == spec
                            && run.status == crate::features::run::model::RunStatus::Running
                    }) {
                        run.finish(success, stdout.clone(), stderr.clone());
                    }
                    if let Ok(targets) = self.pants_client.load_targets() {
                        self.state.target_panel.replace_items(targets);
                    }
                    self.message = if success {
                        format!("{} passed", spec)
                    } else {
                        format!("{} failed", spec)
                    };
                }
                AppEvent::TargetsRefreshed => {
                    if let Ok(targets) = self.pants_client.load_targets() {
                        self.state.target_panel.replace_items(targets);
                    }
                    self.message = String::from("Refreshed target cache");
                }
                AppEvent::AffectedTestsLoaded { scope, result } => {
                    self.affected_loading = false;
                    self.affected_scope = Some(scope.clone());
                    match result {
                        Ok(tests) => {
                            let count = tests.len();
                            self.affected_tests = tests;
                            self.message = format!("Found {count} affected tests for {scope}");
                        }
                        Err(error) => {
                            self.affected_tests.clear();
                            self.message = format!("Affected tests failed: {error}");
                        }
                    }
                }
                AppEvent::GoalRunFinished {
                    command,
                    success,
                    stdout,
                    stderr,
                } => {
                    if let Some(run) = self.run_history.iter_mut().rev().find(|run| {
                        run.spec == command
                            && run.status == crate::features::run::model::RunStatus::Running
                    }) {
                        run.finish(success, stdout.clone(), stderr.clone());
                    }
                    self.last_run_output = format!("{stdout}\n{stderr}").trim().to_string();
                    self.message = if success {
                        format!("{command} completed")
                    } else {
                        format!("{command} failed")
                    };
                }
            }
        }
    }

    pub fn frame_tick(&mut self) {
        self.frame_tick = self.frame_tick.wrapping_add(1);
    }

    pub fn running_count(&self) -> usize {
        self.run_history
            .iter()
            .filter(|run| run.status == crate::features::run::model::RunStatus::Running)
            .count()
    }

    pub fn spinner(&self) -> char {
        const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
        FRAMES[self.frame_tick % FRAMES.len()]
    }

    pub fn goals(&self) -> &[crate::shared::pants::models::PantsGoal] {
        self.pants_client.goals()
    }

    pub fn finish_interactive_goal(&mut self, goal: &str, specs: &[String], success: bool) {
        let command = command_label(goal, specs);
        if let Some(run) = self.run_history.iter_mut().rev().find(|run| {
            run.spec == command && run.status == crate::features::run::model::RunStatus::Running
        }) {
            run.finish(success, String::new(), String::new());
        }
        self.message = if success {
            format!("{command} completed")
        } else {
            format!("{command} failed")
        };
    }

    pub fn execute_interactive_goal(&self, goal: &str, specs: &[String]) -> bool {
        self.pants_client
            .run_interactive_goal(goal, specs)
            .unwrap_or(false)
    }

    fn refresh_targets_async(&self) {
        let tx = self.event_tx.clone();
        let cache_client = self.pants_client.clone();
        std::thread::spawn(move || {
            let _ = cache_client.refresh_targets();
            let _ = tx.send(AppEvent::TargetsRefreshed);
        });
    }

    fn run_target_async(&self, spec: String) {
        let tx = self.event_tx.clone();
        let client = self.pants_client.clone();
        std::thread::spawn(move || {
            let _ = tx.send(AppEvent::TargetRunStarted { spec: spec.clone() });
            match client.run_python_test_target_with_output(&spec) {
                Ok((success, stdout, stderr)) => {
                    let _ = tx.send(AppEvent::TargetRunFinished {
                        spec,
                        success,
                        stdout,
                        stderr,
                    });
                }
                Err(err) => {
                    let _ = tx.send(AppEvent::TargetRunFinished {
                        spec,
                        success: false,
                        stdout: String::new(),
                        stderr: err.to_string(),
                    });
                }
            }
        });
    }

    fn load_affected_tests_async(&mut self) {
        let Some(scope) = self.current_scope_spec() else {
            self.message = String::from("Select a target or directory first");
            return;
        };
        self.affected_loading = true;
        self.affected_scope = Some(scope.clone());
        self.affected_tests.clear();
        self.message = format!("Loading affected tests for {scope}");
        let tx = self.event_tx.clone();
        let client = self.pants_client.clone();
        std::thread::spawn(move || {
            let result = client
                .affected_python_tests(&scope)
                .map_err(|error| error.to_string());
            let _ = tx.send(AppEvent::AffectedTestsLoaded { scope, result });
        });
    }

    fn current_scope_spec(&self) -> Option<String> {
        self.state.target_panel.current_pants_spec()
    }

    fn run_selected_goal(&mut self) -> Effect {
        use crate::features::goal::model::GoalKind;

        let Some(goal) = self.pants_client.goals().get(self.goal_selected) else {
            self.message = String::from("No Pants goal selected");
            return Effect::None;
        };
        let goal_name = goal.name.clone();
        let specs = if goal_accepts_specs(&goal_name) {
            self.current_scope_spec().into_iter().collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        if GoalKind::for_name(&goal_name) == GoalKind::Mutating {
            self.pending_goal = Some((goal_name.clone(), specs));
            self.message = format!("Run mutating goal {goal_name}? y confirm / n cancel");
            Effect::None
        } else {
            self.start_goal(goal_name, specs)
        }
    }

    fn start_goal(&mut self, goal: String, specs: Vec<String>) -> Effect {
        use crate::features::goal::model::GoalKind;

        let command = command_label(&goal, &specs);
        self.run_history.push(RunItem::running(command.clone()));
        self.runs_scroll = 0;
        self.message = format!("Running {command}");
        if GoalKind::for_name(&goal) == GoalKind::Interactive {
            return Effect::RunInteractive { goal, specs };
        }

        let tx = self.event_tx.clone();
        let client = self.pants_client.clone();
        std::thread::spawn(move || {
            let result = client.run_goal_with_output(&goal, &specs);
            let event = match result {
                Ok((success, stdout, stderr)) => AppEvent::GoalRunFinished {
                    command,
                    success,
                    stdout,
                    stderr,
                },
                Err(error) => AppEvent::GoalRunFinished {
                    command,
                    success: false,
                    stdout: String::new(),
                    stderr: error.to_string(),
                },
            };
            let _ = tx.send(event);
        });
        Effect::None
    }

    fn visible_target_specs(&self) -> Vec<String> {
        self.state.target_panel.visible_test_specs()
    }

    fn mark_running(&mut self, spec: &str) {
        if !self.run_history.iter().any(|run| {
            run.spec == spec && run.status == crate::features::run::model::RunStatus::Running
        }) {
            self.run_history.push(RunItem::running(spec));
            self.runs_scroll = 0;
        }
        if let Some(target) = self
            .state
            .target_panel
            .items
            .iter_mut()
            .find(|item| item.spec == spec)
        {
            *target = target.clone().running();
        }
        self.state.target_panel.sync_selection();
    }

    fn apply_running_state(&mut self, spec: &str) {
        self.mark_running(spec);
    }

    fn apply_local_effect(&mut self, effect: Effect) -> AppResult<Effect> {
        Ok(match effect {
            Effect::Message(message) => {
                self.message = message;
                Effect::None
            }
            Effect::ToggleHelp => {
                self.show_help = !self.show_help;
                Effect::None
            }
            passthrough => passthrough,
        })
    }
}

fn command_label(goal: &str, specs: &[String]) -> String {
    if specs.is_empty() {
        format!("pants {goal}")
    } else {
        format!("pants {goal} {}", specs.join(" "))
    }
}

fn goal_accepts_specs(goal: &str) -> bool {
    !matches!(
        goal,
        "version" | "help" | "help-all" | "roots" | "generate-lockfiles" | "experimental-explorer"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_goal_command_labels() {
        assert_eq!(
            command_label("lint", &[String::from("src::")]),
            "pants lint src::"
        );
        assert_eq!(command_label("version", &[]), "pants version");
    }

    #[test]
    fn global_goals_do_not_receive_the_current_scope() {
        assert!(!goal_accepts_specs("version"));
        assert!(!goal_accepts_specs("generate-lockfiles"));
        assert!(goal_accepts_specs("test"));
    }
}
