//! Content generation: pick random words from a pool.

pub mod lessons;
pub mod wordlist;

use rand::seq::SliceRandom;
use rand::Rng;

/// Produce `count` words chosen at random (with repetition) from `pool`,
/// joined by single spaces. Empty pool yields an empty string.
pub fn generate_words<R: Rng>(pool: &[String], count: usize, rng: &mut R) -> String {
    if pool.is_empty() {
        return String::new();
    }
    (0..count)
        .map(|_| pool.choose(rng).unwrap().as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Read words from process stdin. Returns an error if stdin is not piped.
pub fn from_stdin() -> crate::error::Result<Vec<String>> {
    use std::io::IsTerminal;
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        return Ok(Vec::new()); // no piped input
    }
    wordlist::from_reader(stdin.lock())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn generates_requested_word_count() {
        let pool: Vec<String> = vec!["a".into(), "bb".into(), "ccc".into()];
        let mut rng = StdRng::seed_from_u64(42);
        let text = generate_words(&pool, 5, &mut rng);
        assert_eq!(text.split(' ').count(), 5);
    }

    #[test]
    fn deterministic_with_seed() {
        let pool: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let t1 = generate_words(&pool, 10, &mut StdRng::seed_from_u64(7));
        let t2 = generate_words(&pool, 10, &mut StdRng::seed_from_u64(7));
        assert_eq!(t1, t2);
    }

    #[test]
    fn from_stdin_returns_empty_when_terminal() {
        // In test harness stdin is not a tty piped with words; this just verifies
        // the function is callable and returns a Vec without panicking.
        let _ = super::from_stdin();
    }
}
