//! Drives a single typing test: remaps incoming keystrokes through the chosen
//! layout, feeds the core TestSession, and tracks mode/elapsed completion.

use crate::app::Mode;
use crate::engine::{Score, TestSession};
use crate::layout::remap::Remapper;
use std::collections::HashMap;

pub struct SessionRunner {
    remapper: Remapper,
    session: TestSession,
    mode: Mode,
    elapsed_secs: f64,
}

impl SessionRunner {
    pub fn new(target_text: &str, remapper: Remapper, mode: Mode) -> Self {
        Self {
            remapper,
            session: TestSession::new(target_text),
            mode,
            elapsed_secs: 0.0,
        }
    }

    /// Feed a raw character produced by the source layout (what the OS sends).
    /// It is remapped through the target layout before scoring.
    pub fn type_char(&mut self, raw: char) {
        let mapped = self.remapper.remap(raw);
        self.session.input(mapped);
    }

    /// Update elapsed time (seconds since the test started).
    pub fn set_elapsed(&mut self, secs: f64) {
        self.elapsed_secs = secs;
    }

    pub fn elapsed(&self) -> f64 {
        self.elapsed_secs
    }

    pub fn is_finished(&self) -> bool {
        match self.mode {
            Mode::Timed(limit) => self.session.is_finished() || self.elapsed_secs >= limit as f64,
            _ => self.session.is_finished(),
        }
    }

    pub fn set_strict(&mut self, strict: bool) {
        self.session.set_strict(strict);
    }

    pub fn backspace(&mut self) {
        self.session.backspace();
    }

    pub fn cells(&self) -> &[crate::engine::Cell] {
        self.session.cells()
    }

    pub fn cursor(&self) -> usize {
        self.session.cursor()
    }

    pub fn score(&self) -> Score {
        self.session.score(self.elapsed_secs)
    }

    pub fn per_key(&self) -> &HashMap<char, crate::engine::KeyStat> {
        self.session.per_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::builtin::load_registry;

    fn remapper(src: &str, tgt: &str) -> Remapper {
        let reg = load_registry(None).unwrap();
        Remapper::new(reg[src].clone(), reg[tgt].clone())
    }

    #[test]
    fn remaps_keystrokes_before_scoring() {
        // Target colemak: to produce 'f' the user presses the QWERTY 'e' key.
        let mut r = SessionRunner::new("f", remapper("qwerty", "colemak"), Mode::Words(1));
        r.type_char('e'); // QWERTY 'e' -> colemak 'f'
        assert!(r.is_finished());
        assert_eq!(r.score().accuracy, 1.0);
    }

    #[test]
    fn timed_mode_finishes_when_time_is_up() {
        let mut r = SessionRunner::new(
            "the quick brown fox",
            remapper("qwerty", "qwerty"),
            Mode::Timed(30),
        );
        r.type_char('t');
        assert!(!r.is_finished());
        r.set_elapsed(30.0);
        assert!(r.is_finished());
    }

    #[test]
    fn words_mode_finishes_when_text_consumed() {
        let mut r = SessionRunner::new("hi", remapper("qwerty", "qwerty"), Mode::Words(1));
        r.type_char('h');
        r.type_char('i');
        assert!(r.is_finished());
    }

    #[test]
    fn strict_runner_blocks_on_wrong() {
        let mut r = SessionRunner::new("ab", remapper("qwerty", "qwerty"), Mode::Words(1));
        r.set_strict(true);
        r.type_char('x'); // wrong for 'a'
        assert_eq!(r.cursor(), 0);
        r.type_char('a');
        assert_eq!(r.cursor(), 1);
    }

    #[test]
    fn elapsed_reflects_set_value() {
        let mut r = SessionRunner::new("hi", remapper("qwerty", "qwerty"), Mode::Words(1));
        r.set_elapsed(12.5);
        assert!((r.elapsed() - 12.5).abs() < 1e-9);
    }

    #[test]
    fn runner_backspace_reverts() {
        let mut r = SessionRunner::new("ab", remapper("qwerty", "qwerty"), Mode::Words(1));
        r.type_char('a');
        assert_eq!(r.cursor(), 1);
        r.backspace();
        assert_eq!(r.cursor(), 0);
    }
}
