//! Application state shell shared by the UI.

pub mod menu;
pub mod runner;

/// Which test mode is being run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Words(usize),
    Timed(u64),
    Lesson(usize),
    Quote(crate::content::quotes::QuoteLength),
}

/// Which screen is currently shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Test,
    Results,
}

use crate::app::menu::{MenuState, StartRequest};
use crate::app::runner::SessionRunner;
use crate::cli::Settings;
use crate::config::Config;
use crate::content::quotes::{self, QuoteLength};
use crate::content::{decorate, generate_word_list, lessons};
use crate::engine::Score;
use crate::keys::Keymap;
use crate::layout::remap::Remapper;
use crate::layout::Layout;
use crate::stats::KeyStats;
use rand::rngs::ThreadRng;
use std::collections::HashMap;

pub struct App {
    pub settings: Settings,
    pub registry: HashMap<String, Layout>,
    pub stats: KeyStats,
    pub screen: Screen,
    pub menu: MenuState,
    pub runner: Option<SessionRunner>,
    pub target_text: Option<String>,
    pub current_layout: Option<String>,
    pub active_mode: Option<Mode>,
    pub active_punctuation: bool,
    pub active_numbers: bool,
    pub last_score: Option<Score>,
    pub session_stats: KeyStats,
    pub should_quit: bool,
    pub word_pool: Vec<String>,
    pub keymap: Keymap,
    pub overlay: Option<MenuState>,
}

impl App {
    pub fn new(
        settings: Settings,
        registry: HashMap<String, Layout>,
        stats: KeyStats,
        pool: Vec<String>,
        keymap: Keymap,
    ) -> Self {
        let layouts: Vec<String> = {
            let mut v: Vec<String> = registry.keys().cloned().collect();
            v.sort();
            v
        };
        let mut menu = MenuState::new(layouts, &settings.target_layout);
        menu.punctuation = settings.punctuation;
        menu.numbers = settings.numbers;
        App {
            settings,
            registry,
            stats,
            screen: Screen::Menu,
            menu,
            runner: None,
            target_text: None,
            current_layout: None,
            active_mode: None,
            active_punctuation: false,
            active_numbers: false,
            last_score: None,
            session_stats: KeyStats::default(),
            should_quit: false,
            word_pool: pool,
            keymap,
            overlay: None,
        }
    }

    pub fn target_layout(&self) -> Option<&Layout> {
        self.current_layout
            .as_ref()
            .and_then(|n| self.registry.get(n))
    }

    /// Build target text for a start request and enter the Test screen.
    pub fn start(&mut self, req: StartRequest, rng: &mut ThreadRng) {
        let layout = match self.registry.get(&req.layout) {
            Some(l) => l.clone(),
            None => return,
        };
        let source = self
            .registry
            .get(&self.settings.source_layout)
            .cloned()
            .unwrap_or_else(|| layout.clone());

        self.active_mode = Some(req.mode.clone());
        self.active_punctuation = req.punctuation;
        self.active_numbers = req.numbers;

        let text = match &req.mode {
            Mode::Words(n) => {
                let mut words = generate_word_list(&self.word_pool, *n, rng);
                decorate::apply(&mut words, &layout, req.punctuation, req.numbers, rng);
                words.join(" ")
            }
            Mode::Timed(_) => {
                let mut words = generate_word_list(&self.word_pool, 200, rng);
                decorate::apply(&mut words, &layout, req.punctuation, req.numbers, rng);
                words.join(" ")
            }
            Mode::Lesson(level) => {
                let prog = lessons::progression(&layout);
                let lesson = prog
                    .get(level.saturating_sub(1))
                    .cloned()
                    .unwrap_or_else(|| prog[0].clone());
                lessons::drill_text(&lesson, &self.word_pool, 30, rng)
            }
            Mode::Quote(length) => {
                let mut all = quotes::bundled();
                if let Some(dir) = Config::config_dir().map(|d| d.join("quotes")) {
                    if let Ok(user) = quotes::from_dir(&dir) {
                        all.extend(user);
                    }
                }
                let raw = quotes::pick(&all, *length, rng)
                    .or_else(|| quotes::pick(&all, QuoteLength::All, rng))
                    .unwrap_or_default();
                quotes::normalize(&raw, &layout)
            }
        };

        let remapper = Remapper::new(source, layout);
        let mut runner = SessionRunner::new(&text, remapper, req.mode);
        runner.set_strict(self.settings.strict);
        self.runner = Some(runner);
        self.target_text = Some(text);
        self.current_layout = Some(req.layout);
        self.session_stats = KeyStats::default();
        self.screen = Screen::Test;
    }

    /// Finish the current test: record stats and show results.
    pub fn finish(&mut self) {
        if let Some(runner) = &self.runner {
            self.last_score = Some(runner.score());
            let mut sess = KeyStats::default();
            sess.merge(runner.per_key());
            self.stats.merge(runner.per_key());
            self.session_stats = sess;
        }
        self.screen = Screen::Results;
    }

    /// Open the quick-panel, seeded with the current layout + mode.
    pub fn open_panel(&mut self) {
        let layouts: Vec<String> = {
            let mut v: Vec<String> = self.registry.keys().cloned().collect();
            v.sort();
            v
        };
        let layout = self
            .current_layout
            .clone()
            .unwrap_or_else(|| self.settings.target_layout.clone());
        let mut menu = MenuState::new(layouts, &layout);
        if let Some(mode) = &self.active_mode {
            menu.seed_mode(mode);
        }
        menu.punctuation = self.active_punctuation;
        menu.numbers = self.active_numbers;
        self.overlay = Some(menu);
    }

    /// Apply the panel selection: start a new test, close the panel.
    pub fn confirm_panel(&mut self, rng: &mut ThreadRng) {
        if let Some(req) = self.overlay.as_ref().map(|m| m.request()) {
            self.start(req, rng);
        }
        self.overlay = None;
    }

    /// Close the panel without changing anything.
    pub fn cancel_panel(&mut self) {
        self.overlay = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::Keymap;
    use crate::layout::builtin::load_registry;

    fn app() -> App {
        let registry = load_registry(None).unwrap();
        let settings = crate::cli::Settings::resolve(
            &crate::cli::Args::default(),
            &crate::config::Config::default(),
        );
        App::new(
            settings,
            registry,
            KeyStats::default(),
            vec!["the".into(), "fox".into()],
            Keymap::defaults(),
        )
    }

    #[test]
    fn open_panel_seeds_overlay() {
        let mut a = app();
        let mut rng = rand::thread_rng();
        a.start(
            StartRequest::new(Mode::Words(5), "colemak-dhm".into()),
            &mut rng,
        );
        a.open_panel();
        let ov = a.overlay.as_ref().unwrap();
        assert_eq!(ov.layouts[ov.layout_idx], "colemak-dhm");
    }

    #[test]
    fn confirm_panel_starts_and_closes() {
        let mut a = app();
        let mut rng = rand::thread_rng();
        a.start(StartRequest::new(Mode::Words(5), "qwerty".into()), &mut rng);
        a.open_panel();
        a.confirm_panel(&mut rng);
        assert!(a.overlay.is_none());
        assert_eq!(a.screen, Screen::Test);
        assert!(a.target_text.is_some());
    }

    #[test]
    fn cancel_panel_closes_only() {
        let mut a = app();
        let mut rng = rand::thread_rng();
        a.start(StartRequest::new(Mode::Words(5), "qwerty".into()), &mut rng);
        a.open_panel();
        a.cancel_panel();
        assert!(a.overlay.is_none());
    }

    #[test]
    fn quote_start_produces_typeable_text() {
        let mut a = app();
        let mut rng = rand::thread_rng();
        a.start(
            StartRequest::new(Mode::Quote(QuoteLength::All), "colemak-dhm".into()),
            &mut rng,
        );
        let layout = a.target_layout().unwrap().clone();
        let text = a.target_text.clone().unwrap();
        assert!(!text.is_empty());
        for c in text.chars().filter(|c| *c != ' ') {
            // letters (any case) map via case-aware remap; others must be on-grid
            let typeable = c.is_ascii_alphabetic()
                || c.is_ascii_digit()
                || layout.position_of(c.to_ascii_lowercase()).is_some();
            assert!(typeable, "char {c:?} not typeable");
        }
    }

    #[test]
    fn start_with_punctuation_only_typeable_chars() {
        let mut a = app();
        let mut rng = rand::thread_rng();
        let mut req = StartRequest::new(Mode::Words(60), "colemak-dhm".into());
        req.punctuation = true;
        a.start(req, &mut rng);
        let layout = a.target_layout().unwrap().clone();
        let text = a.target_text.clone().unwrap();
        for c in text.chars().filter(|c| !c.is_alphabetic() && *c != ' ') {
            assert!(
                layout.position_of(c).is_some(),
                "char {c:?} typeable on layout"
            );
        }
    }
}
