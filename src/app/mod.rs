//! Application state shell shared by the UI.

pub mod menu;
pub mod runner;

/// Which test mode is being run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Words(usize),
    Timed(u64),
    Lesson(usize),
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
use crate::content::{generate_words, lessons};
use crate::engine::Score;
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
    pub last_score: Option<Score>,
    pub session_stats: KeyStats,
    pub should_quit: bool,
    pub word_pool: Vec<String>,
}

impl App {
    pub fn new(
        settings: Settings,
        registry: HashMap<String, Layout>,
        stats: KeyStats,
        pool: Vec<String>,
    ) -> Self {
        let layouts: Vec<String> = {
            let mut v: Vec<String> = registry.keys().cloned().collect();
            v.sort();
            v
        };
        let menu = MenuState::new(layouts, &settings.target_layout);
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
            last_score: None,
            session_stats: KeyStats::default(),
            should_quit: false,
            word_pool: pool,
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

        let text = match &req.mode {
            Mode::Words(n) => generate_words(&self.word_pool, *n, rng),
            Mode::Timed(_) => generate_words(&self.word_pool, 200, rng),
            Mode::Lesson(level) => {
                let prog = lessons::progression(&layout);
                let lesson = prog
                    .get(level.saturating_sub(1))
                    .cloned()
                    .unwrap_or_else(|| prog[0].clone());
                lessons::drill_text(&lesson, &self.word_pool, 30, rng)
            }
        };

        let remapper = Remapper::new(source, layout);
        self.active_mode = Some(req.mode.clone());
        self.runner = Some(SessionRunner::new(&text, remapper, req.mode));
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
}
