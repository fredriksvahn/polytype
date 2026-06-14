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

#[derive(Debug, Clone)]
pub struct TestSession {
    target: Vec<char>,
    pos: usize,
    correct: usize,
    typed: usize,
    per_key: HashMap<char, KeyStat>,
}

impl TestSession {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.chars().collect(),
            pos: 0,
            correct: 0,
            typed: 0,
            per_key: HashMap::new(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.pos >= self.target.len()
    }

    pub fn cursor(&self) -> usize {
        self.pos
    }

    /// Feed one already-remapped char. Returns true if it matched the expected char.
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
            self.pos += 1;
        } else {
            entry.misses += 1;
            self.pos += 1; // advance (no backspace handling in v1 scoring)
        }
        correct
    }

    pub fn per_key(&self) -> &HashMap<char, KeyStat> {
        &self.per_key
    }

    /// Compute the score given how many seconds elapsed.
    pub fn score(&self, elapsed_secs: f64) -> Score {
        let minutes = (elapsed_secs / 60.0).max(1e-9);
        let wpm = (self.correct as f64 / 5.0) / minutes;
        let accuracy = if self.typed == 0 {
            0.0
        } else {
            self.correct as f64 / self.typed as f64
        };
        Score { wpm, accuracy, correct: self.correct, typed: self.typed }
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
}
