use crate::tweaks::{Category, TweakStatus, build_categories};
use crossterm::event::{KeyCode, KeyEvent};

// ─── Screen / Focus ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,   // Category list
    Category,    // Tweak list within a category
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Categories,     // Dashboard: category list
    Tweaks,         // Category: tweak list
}

// ─── Confirm Dialog ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    ApplyAll,
    RevertAll,
    ApplyTweak(usize),    // tweak index
    RevertTweak(usize),   // tweak index
}

// ─── Message ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StatusMsg {
    pub text: String,
    pub is_error: bool,
}

impl StatusMsg {
    pub fn ok(text: impl Into<String>) -> Self {
        Self { text: text.into(), is_error: false }
    }
    pub fn err(text: impl Into<String>) -> Self {
        Self { text: text.into(), is_error: true }
    }
}

// ─── App State ───────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,
    pub focus: Focus,
    pub categories: Vec<Category>,
    pub selected_category: usize,
    pub selected_tweak: usize,
    pub scroll_offset: usize,
    pub confirm: Option<ConfirmAction>,
    pub status: Option<StatusMsg>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Dashboard,
            focus: Focus::Categories,
            categories: build_categories(),
            selected_category: 0,
            selected_tweak: 0,
            scroll_offset: 0,
            confirm: None,
            status: None,
            should_quit: false,
        }
    }

    pub fn current_category(&self) -> &Category {
        &self.categories[self.selected_category]
    }

    pub fn total_tweaks(&self) -> usize {
        self.categories.iter().map(|c| c.tweaks.len()).sum()
    }

    pub fn total_applied(&self) -> usize {
        self.categories.iter().map(|c| c.applied_count()).sum()
    }

    // ─── Key Handling ────────────────────────────────────────────────────────

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Clear status on any keypress
        self.status = None;

        // Confirm dialog mode
        if let Some(action) = self.confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.confirm = None;
                    self.execute_confirm(action);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm = None;
                    self.status = Some(StatusMsg::ok("Cancelled"));
                }
                _ => {}
            }
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                return;
            }
            _ => {}
        }

        match self.screen {
            Screen::Dashboard => self.handle_dashboard(key),
            Screen::Category => self.handle_category(key),
        }
    }

    fn handle_dashboard(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_category > 0 {
                    self.selected_category -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_category < self.categories.len() - 1 {
                    self.selected_category += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.screen = Screen::Category;
                self.focus = Focus::Tweaks;
                self.selected_tweak = 0;
                self.scroll_offset = 0;
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.confirm = Some(ConfirmAction::ApplyAll);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.confirm = Some(ConfirmAction::RevertAll);
            }
            _ => {}
        }
    }

    fn handle_category(&mut self, key: KeyEvent) {
        let tweak_count = self.current_category().tweaks.len();

        match key.code {
            KeyCode::Esc | KeyCode::Char('b') => {
                self.screen = Screen::Dashboard;
                self.focus = Focus::Categories;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_tweak > 0 {
                    self.selected_tweak -= 1;
                    if self.selected_tweak < self.scroll_offset {
                        self.scroll_offset = self.selected_tweak;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_tweak < tweak_count - 1 {
                    self.selected_tweak += 1;
                }
            }
            KeyCode::Char('g') => {
                self.selected_tweak = 0;
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                self.selected_tweak = tweak_count.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_selected_tweak();
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.confirm = Some(ConfirmAction::ApplyAll);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.confirm = Some(ConfirmAction::RevertAll);
            }
            _ => {}
        }
    }

    fn toggle_selected_tweak(&mut self) {
        let cat = &self.categories[self.selected_category];
        let tweak = &cat.tweaks[self.selected_tweak];
        let status = tweak.get_status();

        if status == TweakStatus::Applied {
            if tweak.revert().is_ok() {
                self.status = Some(StatusMsg::ok(format!("Reverted: {}", tweak.name)));
            } else {
                self.status = Some(StatusMsg::err(format!("Failed to revert: {}", tweak.name)));
            }
        } else {
            if tweak.apply().is_ok() {
                self.status = Some(StatusMsg::ok(format!("Applied: {}", tweak.name)));
            } else {
                self.status = Some(StatusMsg::err(format!("Failed to apply: {}", tweak.name)));
            }
        }
    }

    fn execute_confirm(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::ApplyAll => {
                let mut total_ok = 0;
                let mut total = 0;
                for cat in &self.categories {
                    let (ok, count) = cat.apply_all();
                    total_ok += ok;
                    total += count;
                }
                self.status = Some(StatusMsg::ok(format!(
                    "Applied {}/{} tweaks", total_ok, total
                )));
            }
            ConfirmAction::RevertAll => {
                let mut total_ok = 0;
                let mut total = 0;
                for cat in &self.categories {
                    let (ok, count) = cat.revert_all();
                    total_ok += ok;
                    total += count;
                }
                self.status = Some(StatusMsg::ok(format!(
                    "Reverted {}/{} tweaks to defaults", total_ok, total
                )));
            }
            ConfirmAction::ApplyTweak(idx) => {
                let cat = &self.categories[self.selected_category];
                if let Some(tweak) = cat.tweaks.get(idx) {
                    if tweak.apply().is_ok() {
                        self.status = Some(StatusMsg::ok(format!("Applied: {}", tweak.name)));
                    } else {
                        self.status = Some(StatusMsg::err(format!("Failed: {}", tweak.name)));
                    }
                }
            }
            ConfirmAction::RevertTweak(idx) => {
                let cat = &self.categories[self.selected_category];
                if let Some(tweak) = cat.tweaks.get(idx) {
                    if tweak.revert().is_ok() {
                        self.status = Some(StatusMsg::ok(format!("Reverted: {}", tweak.name)));
                    } else {
                        self.status = Some(StatusMsg::err(format!("Failed: {}", tweak.name)));
                    }
                }
            }
        }
    }
}
