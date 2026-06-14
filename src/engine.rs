//! Typing test session: tracks typed input against a target string, computes
//! WPM/accuracy, and accumulates per-key hit/miss counts.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Score {
    pub wpm: f64,
    pub accuracy: f64,
    pub correct: usize,
    pub typed: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct KeyStat {
    pub hits: usize,
    pub misses: usize,
}

/// Per-position typing outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Untyped,
    Correct,
    Wrong,
}

#[derive(Debug, Clone)]
pub struct TestSession {
    target: Vec<char>,
    cells: Vec<Cell>,
    pos: usize,
    correct: usize,
    typed: usize,
    per_key: HashMap<char, KeyStat>,
    strict: bool,
}

impl TestSession {
    pub fn new(target: &str) -> Self {
        let target: Vec<char> = target.chars().collect();
        let cells = vec![Cell::Untyped; target.len()];
        Self {
            target,
            cells,
            pos: 0,
            correct: 0,
            typed: 0,
            per_key: HashMap::new(),
            strict: false,
        }
    }

    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    pub fn is_finished(&self) -> bool {
        self.pos >= self.target.len()
    }

    pub fn cursor(&self) -> usize {
        self.pos
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Feed one already-remapped char. Returns true if it matched.
    pub fn input(&mut self, ch: char) -> bool {
        if self.is_finished() {
            return false;
        }
        let expected = self.target[self.pos];
        self.typed += 1;
        let correct = ch == expected;
        let entry = self.per_key.entry(expected).or_default();
        if correct {
            self.correct += 1;
            entry.hits += 1;
            self.cells[self.pos] = Cell::Correct;
            self.pos += 1;
        } else {
            entry.misses += 1;
            self.cells[self.pos] = Cell::Wrong;
            if !self.strict {
                self.pos += 1;
            }
        }
        correct
    }

    /// Revert the previous cell (correction). No-op at the start.
    pub fn backspace(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
            self.cells[self.pos] = Cell::Untyped;
        }
    }

    pub fn per_key(&self) -> &HashMap<char, KeyStat> {
        &self.per_key
    }

    pub fn score(&self, elapsed_secs: f64) -> Score {
        let minutes = (elapsed_secs / 60.0).max(1e-9);
        let wpm = (self.correct as f64 / 5.0) / minutes;
        let accuracy = if self.typed == 0 {
            0.0
        } else {
            self.correct as f64 / self.typed as f64
        };
        Score {
            wpm,
            accuracy,
            correct: self.correct,
            typed: self.typed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_str(s: &mut TestSession, input: &str) {
        for c in input.chars() {
            s.input(c);
        }
    }

    #[test]
    fn perfect_run_is_full_accuracy() {
        let mut s = TestSession::new("the");
        type_str(&mut s, "the");
        assert!(s.is_finished());
        let score = s.score(60.0);
        assert_eq!(score.accuracy, 1.0);
        assert_eq!(score.correct, 3);
    }

    #[test]
    fn wpm_uses_chars_over_five() {
        // 25 correct chars in 60s => 25/5 / 1.0 = 5 wpm
        let target: String = "a".repeat(25);
        let mut s = TestSession::new(&target);
        type_str(&mut s, &target);
        let score = s.score(60.0);
        assert!((score.wpm - 5.0).abs() < 1e-6);
    }

    #[test]
    fn mistakes_lower_accuracy_and_record_per_key() {
        let mut s = TestSession::new("ab");
        s.input('x'); // wrong, expected 'a'
        s.input('b'); // right
        let score = s.score(60.0);
        assert_eq!(score.typed, 2);
        assert_eq!(score.correct, 1);
        assert_eq!(s.per_key()[&'a'].misses, 1);
        assert_eq!(s.per_key()[&'b'].hits, 1);
    }

    #[test]
    fn free_mode_advances_on_wrong() {
        let mut s = TestSession::new("ab");
        s.input('x'); // wrong for 'a'
        assert_eq!(s.cursor(), 1);
        assert_eq!(s.cells()[0], Cell::Wrong);
    }

    #[test]
    fn strict_mode_blocks_until_correct() {
        let mut s = TestSession::new("ab");
        s.set_strict(true);
        s.input('x'); // wrong for 'a'
        assert_eq!(s.cursor(), 0, "strict does not advance on wrong");
        assert_eq!(s.cells()[0], Cell::Wrong);
        s.input('a'); // correct now
        assert_eq!(s.cursor(), 1);
        assert_eq!(s.cells()[0], Cell::Correct);
    }

    #[test]
    fn backspace_reverts_cell() {
        let mut s = TestSession::new("ab");
        s.input('a');
        assert_eq!(s.cursor(), 1);
        s.backspace();
        assert_eq!(s.cursor(), 0);
        assert_eq!(s.cells()[0], Cell::Untyped);
    }

    #[test]
    fn correction_lowers_accuracy() {
        // type wrong, backspace, type right -> 1 correct of 2 typed
        let mut s = TestSession::new("a");
        s.set_strict(false);
        s.input('x'); // wrong, advances (free)
        s.backspace();
        s.input('a'); // correct
        let score = s.score(60.0);
        assert_eq!(score.typed, 2);
        assert_eq!(score.correct, 1);
        assert!((score.accuracy - 0.5).abs() < 1e-9);
    }
}
