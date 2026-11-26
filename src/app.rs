use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::mpsc;
use tuikit::{Nav, NavKeys, NavStep, ScrollPopup};
use whichkey::{Outcome, WhichKey};

use crate::{
    entities::Panel,
    error::AppResult,
    features::{
        run::model::RunItem,
        version::{
            data::outside_scope,
            ui::{DependencyRequest, SubTab},
        },
    },
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
    DependenciesLoaded {
        request: DependencyRequest,
        result: Result<Vec<String>, String>,
    },
    GoalRunFinished {
        command: String,
        success: bool,
        stdout: String,
        stderr: String,
    },
}

/// Full output of one run, opened from the Runs panel.
pub struct OutputView {
    pub run: usize,
    pub scroll: usize,
    /// Written while rendering, because only the frame knows the wrapped height.
    pub max_scroll: Cell<usize>,
    pub page: Cell<usize>,
    keys: NavKeys,
}

pub struct App {
    pub text_area: std::cell::Cell<ratatui::layout::Rect>,
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
    filter_before: Option<String>,
    nav: NavKeys,
    pub help_popup: ScrollPopup,
    /// Inner height of the focused panel, written while rendering.
    pub panel_rows: Cell<usize>,
    pub detail_max: Cell<usize>,
    pub version_max: Cell<usize>,
    pub which_key: WhichKey<&'static str>,
    pub project: String,
    pub detail_scroll: u16,
    /// Index into `run_history`; `None` follows the newest run.
    pub runs_selected: Option<usize>,
    pub output_view: Option<OutputView>,
    pub pending_quit: bool,
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
        let mut state = AppState::new();
        if let Ok(version_string) = pants_client.version_string() {
            state.version_panel.version_string = version_string;
        }
        if let Ok(targets) = pants_client.load_targets() {
            state.target_panel = crate::features::target::ui::TargetPanel::with_items(targets);
        }
        Self::with_client(pants_client, state, project)
    }

    fn with_client(pants_client: PantsClient, mut state: AppState, project: String) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        state.sync_sub_projects();
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
            filter_before: None,
            nav: NavKeys::default(),
            text_area: Default::default(),
            help_popup: ScrollPopup::default(),
            panel_rows: Cell::new(10),
            detail_max: Cell::new(0),
            version_max: Cell::new(0),
            which_key: crate::leader::which_key(),
            project,
            detail_scroll: 0,
            runs_selected: None,
            output_view: None,
            pending_quit: false,
            goal_selected: 0,
            pending_goal: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppResult<Effect> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.cancel_pending();
            return Ok(Effect::None);
        }
        if self.output_view.is_some() {
            self.handle_output_key(key);
            return Ok(Effect::None);
        }
        if self.show_help {
            if self.help_popup.key(key) {
                self.show_help = false;
            }
            return Ok(Effect::None);
        }
        if self.pending_quit {
            return Ok(match key.code {
                KeyCode::Char('y') => Effect::Quit,
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.pending_quit = false;
                    self.message = String::from("Quit cancelled");
                    Effect::None
                }
                _ => Effect::None,
            });
        }
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
                KeyCode::Esc => self.cancel_filter(),
                KeyCode::Enter => {
                    self.filter_mode = false;
                    self.filter_before = None;
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
            let effect = self.handle_leader_key(key);
            return self.apply_local_effect(effect);
        }
        if key.code == KeyCode::Char(' ') && key.modifiers == KeyModifiers::NONE {
            self.which_key.open();
            return Ok(Effect::None);
        }
        match self.panel_nav(key) {
            NavStep::Move(_) | NavStep::Pending => return Ok(Effect::None),
            NavStep::Other => {}
        }
        if key
            .modifiers
            .intersects(KeyModifiers::ALT | KeyModifiers::CONTROL)
        {
            let effect = self.state.handle_key(key);
            return self.apply_local_effect(effect);
        }
        let panel = self.state.active_panel;
        match (panel, key.code) {
            (Panel::FileExplorer, KeyCode::Char('/')) => {
                self.filter_before = Some(self.state.target_panel.filter.clone());
                self.filter_mode = true;
                self.state.target_panel.set_filter(String::new());
                self.detail_scroll = 0;
                self.message = String::from("Filter targets • [Enter] apply • [Esc] cancel");
                Ok(Effect::None)
            }
            (Panel::FileExplorer, KeyCode::Enter) => {
                self.state.target_panel.toggle_tree();
                Ok(Effect::None)
            }
            (Panel::FileExplorer, KeyCode::Char('h') | KeyCode::Left) => {
                self.state.target_panel.collapse_or_parent();
                self.detail_scroll = 0;
                Ok(Effect::None)
            }
            (Panel::FileExplorer, KeyCode::Char('l') | KeyCode::Right) => {
                self.state.target_panel.expand();
                Ok(Effect::None)
            }
            (Panel::FileExplorer, KeyCode::Char('o')) => {
                self.open_output_for_row();
                Ok(Effect::None)
            }
            (Panel::Runs, KeyCode::Enter | KeyCode::Char('o')) => {
                self.open_output();
                Ok(Effect::None)
            }
            (Panel::Mode, KeyCode::Enter) => Ok(self.run_selected_goal()),
            (_, KeyCode::Char('a')) => {
                self.load_affected_tests_async();
                Ok(Effect::None)
            }
            (_, KeyCode::Char('T')) => {
                self.run_affected_tests();
                Ok(Effect::None)
            }
            (_, KeyCode::Char('R')) => {
                self.refresh_targets_async();
                Ok(Effect::None)
            }
            (_, KeyCode::Char('v')) => {
                self.state.target_panel.next_view();
                self.detail_scroll = 0;
                self.message = format!("Targets view: {}", self.state.target_panel.view.title());
                Ok(Effect::None)
            }
            (_, KeyCode::Char('r' | 't')) => {
                self.run_scoped_tests();
                Ok(Effect::None)
            }
            (_, KeyCode::Char('A')) => {
                for spec in self.visible_target_specs() {
                    self.mark_running(&spec);
                    self.run_target_async(spec);
                }
                self.message = String::from("Running all visible targets");
                Ok(Effect::None)
            }
            _ => {
                let effect = self.state.handle_key(key);
                self.apply_local_effect(effect)
            }
        }
    }

    /// `j/k`, `gg/G` and paging act on the focused panel; the Version section is one line.
    fn panel_nav(&mut self, key: KeyEvent) -> NavStep {
        let panel = self.state.active_panel;
        if panel == Panel::Version && self.state.version_panel.active_tab == SubTab::Version {
            self.nav = NavKeys::default();
            return NavStep::Other;
        }
        let step = self.nav.press(key);
        let NavStep::Move(nav) = step else {
            return step;
        };
        let page = self.panel_rows.get();
        match panel {
            Panel::FileExplorer => {
                let target = &mut self.state.target_panel;
                target.move_cursor_to(nav.apply(target.cursor, target.rows.len(), page));
                self.detail_scroll = 0;
            }
            Panel::Main => {
                let max = self.detail_max.get();
                let scroll = usize::from(self.detail_scroll).min(max);
                self.detail_scroll = nav.apply(scroll, max + 1, page).min(u16::MAX.into()) as u16;
            }
            Panel::Mode => {
                self.goal_selected =
                    nav.apply(self.goal_selected, self.pants_client.goals().len(), page);
            }
            Panel::Runs => self.move_run_selection(nav, page),
            Panel::Version => {
                let max = self.version_max.get();
                let version = &mut self.state.version_panel;
                version.scroll = nav.apply(version.scroll.min(max), max + 1, page);
            }
        }
        step
    }

    pub fn nav_pending(&self) -> bool {
        self.nav.is_pending()
    }

    fn run_scoped_tests(&mut self) {
        let Some(row) = self.state.target_panel.current_row().cloned() else {
            return;
        };
        let specs = self.state.target_panel.scoped_test_specs();
        if specs.is_empty() {
            self.message = format!("No tests under {}", row.path);
            return;
        }
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

    fn run_affected_tests(&mut self) {
        let specs = self
            .affected_tests
            .iter()
            .map(|target| target.spec.clone())
            .collect::<Vec<_>>();
        if specs.is_empty() {
            self.message = String::from("No affected tests loaded; press [a] first");
            return;
        }
        for spec in specs {
            self.mark_running(&spec);
            self.run_target_async(spec);
        }
        self.message = String::from("Running affected tests");
    }

    /// Ctrl+c cancels whatever is open or pending. It never quits.
    fn cancel_pending(&mut self) {
        self.output_view = None;
        self.show_help = false;
        self.pending_quit = false;
        self.pending_goal = None;
        self.which_key.cancel();
        self.nav = NavKeys::default();
        if self.filter_mode {
            self.cancel_filter();
        }
    }

    fn cancel_filter(&mut self) {
        self.filter_mode = false;
        let previous = self.filter_before.take().unwrap_or_default();
        self.state.target_panel.set_filter(previous);
        self.message = String::from("Filter cancelled");
    }

    pub fn has_pending_confirm(&self) -> bool {
        self.pending_goal.is_some() || self.pending_quit
    }

    pub fn selected_run_index(&self) -> Option<usize> {
        let last = self.run_history.len().checked_sub(1)?;
        Some(self.runs_selected.unwrap_or(last).min(last))
    }

    /// The panel lists newest first, so moving down goes to an older run.
    fn move_run_selection(&mut self, nav: Nav, page: usize) {
        let Some(current) = self.selected_run_index() else {
            return;
        };
        let last = self.run_history.len() - 1;
        let position = nav.apply(last - current, last + 1, page);
        self.runs_selected = (position != 0).then_some(last - position);
    }

    fn open_output_for_row(&mut self) {
        let Some(spec) = self
            .state
            .target_panel
            .current_row()
            .and_then(|row| row.spec.clone())
        else {
            self.message = String::from("Select a target to see its last output");
            return;
        };
        match self.run_history.iter().rposition(|run| run.spec == spec) {
            Some(run) => {
                self.runs_selected = (run + 1 != self.run_history.len()).then_some(run);
                self.open_output();
            }
            None => self.message = format!("No run yet for {spec}; press [r] to run it"),
        }
    }

    fn open_output(&mut self) {
        use crate::features::run::model::RunStatus;

        let Some(run) = self.selected_run_index() else {
            self.message = String::from("No runs yet");
            return;
        };
        // A failure is reported at the end of the output.
        let scroll = if self.run_history[run].status == RunStatus::Failed {
            usize::MAX
        } else {
            0
        };
        self.output_view = Some(OutputView {
            run,
            scroll,
            max_scroll: Cell::new(0),
            page: Cell::new(1),
            keys: NavKeys::default(),
        });
    }

    fn handle_output_key(&mut self, key: KeyEvent) {
        let Some(view) = self.output_view.as_mut() else {
            return;
        };
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q' | 'o')) {
            self.output_view = None;
            return;
        }
        if let NavStep::Move(nav) = view.keys.press(key) {
            let max = view.max_scroll.get();
            view.scroll = nav.apply(view.scroll.min(max), max + 1, view.page.get());
        }
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
        match action {
            "help" => Effect::ToggleHelp,
            "quit" => Effect::Quit,
            "filter" => {
                self.state.set_active_panel(Panel::FileExplorer);
                self.press('/')
            }
            "view" => self.press('v'),
            "refresh" => self.press('R'),
            "run_tests" => self.press('r'),
            "run_all" => self.press('A'),
            "run_goal" => {
                self.state.set_active_panel(Panel::Mode);
                self.run_selected_goal()
            }
            "affected_load" => self.press('a'),
            "affected_run" => self.press('T'),
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
                        self.state.sync_sub_projects();
                    }
                    self.state.version_panel.invalidate_dependencies();
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
                AppEvent::DependenciesLoaded { request, result } => {
                    self.state
                        .version_panel
                        .finish_dependencies(request, result);
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

    /// Starts the lookup the open Dependencies section is waiting for, if any.
    pub fn load_open_section(&mut self) {
        let scope = self.dependency_scope();
        let Some(request) = self.state.version_panel.request_dependencies(scope) else {
            return;
        };
        let tx = self.event_tx.clone();
        let client = self.pants_client.clone();
        std::thread::spawn(move || {
            let result = client
                .dependencies(&request.scope)
                .map(|addresses| outside_scope(&request.scope, addresses))
                .map_err(|error| error.to_string());
            let _ = tx.send(AppEvent::DependenciesLoaded { request, result });
        });
    }

    /// The repository root is skipped: it would run for a long time, and everything it
    /// depends on is the whole lockfile, which answers no question about a selection.
    fn dependency_scope(&self) -> Option<String> {
        self.current_scope_spec().filter(|scope| scope != "::")
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
            let command = command_label(&goal_name, &specs);
            self.pending_goal = Some((goal_name, specs));
            self.message = format!("[y]es / [n]o • Run mutating {command}?");
            Effect::None
        } else {
            self.start_goal(goal_name, specs)
        }
    }

    fn start_goal(&mut self, goal: String, specs: Vec<String>) -> Effect {
        use crate::features::goal::model::GoalKind;

        let command = command_label(&goal, &specs);
        self.run_history.push(RunItem::running(command.clone()));
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
            Effect::Quit if self.running_count() > 0 => {
                self.pending_quit = true;
                self.message = format!(
                    "[y]es / [n]o • {} run(s) in progress. Quit without waiting?",
                    self.running_count()
                );
                Effect::None
            }
            Effect::Message(message) => {
                self.message = message;
                Effect::None
            }
            Effect::ToggleHelp => {
                self.help_popup.reset();
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
    use crate::features::run::model::RunStatus;
    use crate::features::target::{model::TargetItem, ui::TargetPanel};
    use crate::shared::pants::models::PantsGoal;

    fn app() -> App {
        let goal = |name: &str| PantsGoal {
            name: name.into(),
            description: String::new(),
            flags: Vec::new(),
        };
        let mut state = AppState::new();
        state.target_panel = TargetPanel::with_items(vec![TargetItem::new("src/lib:lib")]);
        App::with_client(
            PantsClient::for_tests("/nonexistent".into(), vec![goal("lint"), goal("fmt")]),
            state,
            "project".into(),
        )
    }

    fn press(app: &mut App, code: KeyCode) -> Effect {
        app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
            .unwrap()
    }

    #[test]
    fn help_is_modal_so_hidden_shortcuts_do_not_run_and_esc_only_closes_it() {
        let mut app = app();
        press(&mut app, KeyCode::Char('?'));
        assert!(app.show_help);
        press(&mut app, KeyCode::Char('A'));
        assert!(app.run_history.is_empty());
        assert_eq!(press(&mut app, KeyCode::Esc), Effect::None);
        assert!(!app.show_help);
    }

    #[test]
    fn leader_help_opens_help() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('?'));
        assert!(app.show_help);
    }

    fn control(app: &mut App, character: char) {
        app.handle_key(KeyEvent::new(
            KeyCode::Char(character),
            KeyModifiers::CONTROL,
        ))
        .unwrap();
    }

    fn tree_app() -> App {
        let mut app = app();
        app.state.target_panel = TargetPanel::with_items(
            ["src/a:a", "src/b:b", "src/c:c", "tests/d:d"]
                .map(TargetItem::new)
                .to_vec(),
        );
        app
    }

    #[test]
    fn starts_on_the_source_tree_and_j_k_never_change_focus() {
        let mut app = tree_app();
        assert_eq!(app.state.active_panel, Panel::FileExplorer);
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.state.active_panel, Panel::FileExplorer);
        assert_eq!(app.state.target_panel.cursor, 1);
        app.state.set_active_panel(Panel::Version);
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.state.active_panel, Panel::Version);
    }

    #[test]
    fn j_and_k_scroll_an_open_section_and_stay_in_bounds() {
        let mut app = tree_app();
        app.state.set_active_panel(Panel::Version);
        press(&mut app, KeyCode::Char('l'));
        app.version_max.set(2);
        for _ in 0..5 {
            press(&mut app, KeyCode::Char('j'));
        }
        assert_eq!(app.state.version_panel.scroll, 2);
        press(&mut app, KeyCode::Char('k'));
        assert_eq!(app.state.version_panel.scroll, 1);
        press(&mut app, KeyCode::Char('g'));
        press(&mut app, KeyCode::Char('g'));
        assert_eq!(app.state.version_panel.scroll, 0);
        assert_eq!(app.state.active_panel, Panel::Version);
    }

    #[test]
    fn the_repository_root_is_never_looked_up() {
        let mut app = tree_app();
        assert_eq!(
            app.state.target_panel.current_pants_spec().as_deref(),
            Some("::")
        );
        assert_eq!(app.dependency_scope(), None);
        press(&mut app, KeyCode::Char('j'));
        assert!(app.dependency_scope().is_some());
    }

    #[test]
    fn the_answer_reaches_the_panel_following_the_tree_selection() {
        let mut app = tree_app();
        app.state.version_panel.next_tab();
        press(&mut app, KeyCode::Char('j'));
        app.load_open_section();
        // The test client's root does not exist, so this fails at once without running Pants.
        let scope = app.dependency_scope().unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !app.state.version_panel.has_answer(&scope) {
            assert!(std::time::Instant::now() < deadline, "no answer arrived");
            std::thread::sleep(std::time::Duration::from_millis(10));
            app.drain_events();
        }
    }

    #[test]
    fn tree_uses_gg_g_and_paging() {
        let mut app = tree_app();
        press(&mut app, KeyCode::Char('G'));
        let last = app.state.target_panel.rows.len() - 1;
        assert_eq!(app.state.target_panel.cursor, last);
        press(&mut app, KeyCode::Char('g'));
        assert!(app.nav_pending());
        assert_eq!(app.state.target_panel.cursor, last);
        press(&mut app, KeyCode::Char('g'));
        assert_eq!(app.state.target_panel.cursor, 0);
        app.panel_rows.set(2);
        control(&mut app, 'd');
        assert_eq!(app.state.target_panel.cursor, 1);
        assert_eq!(app.state.active_panel, Panel::FileExplorer);
    }

    #[test]
    fn h_and_l_change_tree_depth_not_focus() {
        let mut app = tree_app();
        press(&mut app, KeyCode::Char('h'));
        assert!(!app.state.target_panel.tree.expanded);
        assert_eq!(app.state.active_panel, Panel::FileExplorer);
        press(&mut app, KeyCode::Char('l'));
        assert!(app.state.target_panel.tree.expanded);
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Char('j'));
        let child = app.state.target_panel.cursor;
        assert!(child > 0);
        let depth = app.state.target_panel.rows[child].depth;
        if !app.state.target_panel.rows[child].has_children {
            press(&mut app, KeyCode::Char('h'));
            assert!(app.state.target_panel.rows[app.state.target_panel.cursor].depth < depth);
        }
    }

    #[test]
    fn a_is_affected_and_d_does_nothing() {
        let mut app = tree_app();
        press(&mut app, KeyCode::Char('d'));
        assert!(!app.affected_loading);
        press(&mut app, KeyCode::Char('T'));
        assert!(app.message.contains("press [a]"), "{}", app.message);
    }

    #[test]
    fn esc_cancels_the_filter_and_restores_the_previous_one() {
        let mut app = tree_app();
        press(&mut app, KeyCode::Char('/'));
        press(&mut app, KeyCode::Char('s'));
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.state.target_panel.filter, "s");
        press(&mut app, KeyCode::Char('/'));
        press(&mut app, KeyCode::Char('t'));
        assert_eq!(app.state.target_panel.filter, "t");
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.state.target_panel.filter, "s");
        assert!(!app.filter_mode);
    }

    #[test]
    fn filter_uses_smartcase_and_shortcut_letters_are_text() {
        let mut app = tree_app();
        press(&mut app, KeyCode::Char('/'));
        press(&mut app, KeyCode::Char('q'));
        assert_eq!(app.state.target_panel.filter, "q");
        press(&mut app, KeyCode::Esc);
        app.state.target_panel.set_filter("TESTS");
        assert!(app.state.target_panel.visible_test_specs().is_empty());
        assert!(app.state.target_panel.rows.len() <= 1);
        app.state.target_panel.set_filter("tests");
        assert!(app.state.target_panel.rows.len() > 1);
    }

    #[test]
    fn ctrl_c_cancels_and_never_quits() {
        let mut app = tree_app();
        control(&mut app, 'c');
        press(&mut app, KeyCode::Char('?'));
        control(&mut app, 'c');
        assert!(!app.show_help);
        press(&mut app, KeyCode::Char('/'));
        control(&mut app, 'c');
        assert!(!app.filter_mode);
        app.run_history.push(RunItem::running("x"));
        press(&mut app, KeyCode::Char('q'));
        assert!(app.pending_quit);
        control(&mut app, 'c');
        assert!(!app.pending_quit);
    }

    #[test]
    fn leader_tree_has_tests_and_no_panel_group() {
        let mut app = tree_app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('p'));
        assert!(!app.which_key.is_open());
        assert_eq!(app.message, "Unknown Space key");
    }

    #[test]
    fn o_on_a_target_opens_its_last_run_or_explains() {
        let mut app = tree_app();
        press(&mut app, KeyCode::Char('o'));
        assert!(app.output_view.is_none());
        let spec = app
            .state
            .target_panel
            .rows
            .iter()
            .find_map(|row| row.spec.clone());
        let Some(spec) = spec else { return };
        let row = app
            .state
            .target_panel
            .rows
            .iter()
            .position(|row| row.spec.as_deref() == Some(&spec))
            .unwrap();
        app.state.target_panel.move_cursor_to(row);
        press(&mut app, KeyCode::Char('o'));
        assert!(app.message.contains("press [r]"), "{}", app.message);
        app.run_history.push(RunItem::running(spec));
        app.run_history.push(RunItem::running("other"));
        press(&mut app, KeyCode::Char('o'));
        assert_eq!(app.output_view.as_ref().unwrap().run, 0);
    }

    #[test]
    fn g_never_runs_a_goal() {
        let mut app = app();
        app.state.set_active_panel(Panel::Mode);
        press(&mut app, KeyCode::Char('g'));
        press(&mut app, KeyCode::Char('g'));
        assert!(app.run_history.is_empty());
    }

    #[test]
    fn mutating_goal_confirmation_names_the_scope() {
        let mut app = app();
        app.state.set_active_panel(Panel::Mode);
        app.goal_selected = 1;
        press(&mut app, KeyCode::Enter);
        assert!(app.has_pending_confirm());
        assert!(app.message.contains("pants fmt ::"), "{}", app.message);
        press(&mut app, KeyCode::Char('n'));
        assert!(!app.has_pending_confirm());
    }

    #[test]
    fn esc_never_quits_and_q_asks_only_while_runs_are_active() {
        let mut app = app();
        assert_eq!(press(&mut app, KeyCode::Esc), Effect::None);
        assert_eq!(app.state.active_panel, Panel::FileExplorer);
        app.run_history.push(RunItem::running("src:test"));
        assert_eq!(press(&mut app, KeyCode::Char('q')), Effect::None);
        assert!(app.pending_quit);
        press(&mut app, KeyCode::Char('n'));
        assert!(!app.pending_quit);
        press(&mut app, KeyCode::Char('q'));
        assert_eq!(press(&mut app, KeyCode::Char('y')), Effect::Quit);
        app.run_history[0].status = RunStatus::Passed;
        app.pending_quit = false;
        assert_eq!(press(&mut app, KeyCode::Char('q')), Effect::Quit);
    }

    #[test]
    fn control_keys_do_not_move_focus_unless_they_are_ctrl_hjkl() {
        let mut app = app();
        app.state.set_active_panel(Panel::FileExplorer);
        for code in ['d', 'a', 'c'] {
            app.handle_key(KeyEvent::new(KeyCode::Char(code), KeyModifiers::CONTROL))
                .unwrap();
            assert_eq!(app.state.active_panel, Panel::FileExplorer);
        }
        app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::ALT))
            .unwrap();
        assert_eq!(app.state.active_panel, Panel::FileExplorer);
        app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL))
            .unwrap();
        assert_eq!(app.state.active_panel, Panel::Runs);
    }

    #[test]
    fn output_of_a_failed_run_opens_at_the_end_and_scrolls_within_bounds() {
        let mut app = app();
        let mut failed = RunItem::running("src:failed");
        failed.finish(false, String::from("first\nlast"), String::new());
        app.run_history.push(failed);
        app.run_history.push(RunItem::running("src:newest"));
        app.state.set_active_panel(Panel::Runs);
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.selected_run_index(), Some(0));
        press(&mut app, KeyCode::Enter);
        let view = app.output_view.as_ref().unwrap();
        assert_eq!(view.run, 0);
        assert_eq!(view.scroll, usize::MAX);
        view.max_scroll.set(10);
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.output_view.as_ref().unwrap().scroll, 10);
        press(&mut app, KeyCode::Char('k'));
        assert_eq!(app.output_view.as_ref().unwrap().scroll, 9);
        press(&mut app, KeyCode::Char('g'));
        press(&mut app, KeyCode::Char('g'));
        assert_eq!(app.output_view.as_ref().unwrap().scroll, 0);
        assert_eq!(press(&mut app, KeyCode::Esc), Effect::None);
        assert!(app.output_view.is_none());
    }

    #[test]
    fn new_runs_do_not_move_a_selected_older_run() {
        let mut app = app();
        app.run_history.push(RunItem::running("a"));
        app.run_history.push(RunItem::running("b"));
        app.state.set_active_panel(Panel::Runs);
        press(&mut app, KeyCode::Char('j'));
        app.run_history.push(RunItem::running("c"));
        assert_eq!(app.selected_run_index(), Some(0));
        press(&mut app, KeyCode::Char('k'));
        assert_eq!(app.selected_run_index(), Some(1));
    }

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
