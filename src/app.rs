use crossterm::event::KeyEvent;
use std::sync::mpsc;
use std::collections::BTreeSet;

use crate::{
    error::AppResult,
    shared::{pants::client::PantsClient, ui::UiTheme},
    state::AppState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    None,
    Quit,
    Message(String),
    ToggleHelp,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    TargetRunStarted { spec: String },
    TargetRunFinished {
        spec: String,
        success: bool,
        stdout: String,
        stderr: String,
    },
    TargetsRefreshed,
}

pub struct App {
    pub state: AppState,
    pub theme: UiTheme,
    pub message: String,
    pub show_help: bool,
    pub last_run_output: String,
    pants_client: PantsClient,
    event_tx: mpsc::Sender<AppEvent>,
    event_rx: mpsc::Receiver<AppEvent>,
    running_targets: BTreeSet<String>,
    frame_tick: usize,
}

impl App {
    pub fn new() -> Self {
        let pants_client = PantsClient::new();
        let (event_tx, event_rx) = mpsc::channel();
        let mut state = AppState::new();
        if let Ok(targets) = pants_client.load_python_test_targets() {
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
            running_targets: BTreeSet::new(),
            frame_tick: 0,
            last_run_output: String::new(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppResult<Effect> {
        if let crossterm::event::KeyCode::Char('R') = key.code {
            self.refresh_targets_async();
            return Ok(Effect::None);
        }
        if let crossterm::event::KeyCode::Char('v') = key.code {
            self.state.target_panel.next_view();
            self.message = format!("Targets view: {}", self.state.target_panel.view.title());
            return Ok(Effect::None);
        }
        if let crossterm::event::KeyCode::Char('t') = key.code {
            if let Some(row) = self.state.target_panel.current_row().cloned() {
                if row.has_children {
                    let specs = self.specs_under_path(&row.path);
                    for spec in specs {
                        self.mark_running(&spec);
                        self.run_target_async(spec);
                    }
                    self.message = format!("Running branch {}", row.label);
                } else if let Some(spec) = self
                    .state
                    .target_panel
                    .items
                    .get(self.state.target_panel.selected)
                    .map(|item| item.spec.clone())
                {
                    self.mark_running(&spec);
                    self.message = format!("Running {}", spec);
                    self.run_target_async(spec);
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
        if let crossterm::event::KeyCode::Enter = key.code {
            self.state.target_panel.toggle_tree();
            self.message = String::from("Toggled target tree");
            return Ok(Effect::None);
        }
        if self.state.active_panel == crate::entities::Panel::FileExplorer {
            match key.code {
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                    self.state.target_panel.move_cursor_up();
                    return Ok(Effect::None);
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                    self.state.target_panel.move_cursor_down();
                    return Ok(Effect::None);
                }
                crossterm::event::KeyCode::Char(' ') => {
                    self.state.target_panel.toggle_tree();
                    self.message = String::from("Toggled selected node");
                    return Ok(Effect::None);
                }
                _ => {}
            }
        }
        let effect = self.state.handle_key(key);
        self.apply_local_effect(effect)
    }

    pub fn drain_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::TargetRunStarted { spec } => {
                    self.running_targets.insert(spec.clone());
                    self.apply_running_state(&spec);
                    self.message = format!("Running {}", spec);
                }
                AppEvent::TargetRunFinished { spec, success, stdout, stderr } => {
                    self.last_run_output = format!("{}\n{}", stdout, stderr).trim().to_string();
                    self.running_targets.remove(&spec);
                    if let Ok(targets) = self.pants_client.refresh_python_test_targets() {
                        self.state.target_panel = crate::features::target::ui::TargetPanel::with_items(targets);
                    }
                    self.message = if success {
                        format!("{} passed", spec)
                    } else {
                        format!("{} failed", spec)
                    };
                }
                AppEvent::TargetsRefreshed => {
                    if let Ok(targets) = self.pants_client.refresh_python_test_targets() {
                        self.state.target_panel = crate::features::target::ui::TargetPanel::with_items(targets);
                    }
                    self.message = String::from("Refreshed target cache");
                }
            }
        }
    }

    pub fn frame_tick(&mut self) {
        self.frame_tick = self.frame_tick.wrapping_add(1);
    }

    pub fn running_count(&self) -> usize {
        self.running_targets.len()
    }

    pub fn spinner(&self) -> char {
        const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
        FRAMES[self.frame_tick % FRAMES.len()]
    }

    pub fn available_goal_lines(&self) -> Vec<String> {
        self.pants_client
            .goals()
            .iter()
            .map(|goal| format!("{} — {}", goal.name, goal.description))
            .collect()
    }

    fn refresh_targets_async(&self) {
        let tx = self.event_tx.clone();
        let cache_client = self.pants_client.clone();
        std::thread::spawn(move || {
            let _ = cache_client.refresh_python_test_targets();
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

    fn visible_target_specs(&self) -> Vec<String> {
        self.state
            .target_panel
            .items
            .iter()
            .filter(|item| self.state.target_panel.view.matches(item))
            .map(|item| item.spec.clone())
            .collect()
    }

    fn specs_under_path(&self, path: &str) -> Vec<String> {
        self.state
            .target_panel
            .rows
            .iter()
            .filter(|row| row.path.starts_with(path) && !row.has_children)
            .filter_map(|row| row.spec.clone())
            .collect()
    }

    fn mark_running(&mut self, spec: &str) {
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
