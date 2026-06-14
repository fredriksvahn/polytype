//! Maps per-key accuracy to a heat level for the keyboard heatmap.

use crate::stats::KeyStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heat {
    Unknown, // never typed
    Bad,     // < 0.85
    Mid,     // 0.85 .. 0.95
    Good,    // >= 0.95
}

/// Heat level for a key given accumulated stats.
pub fn heat_for(stats: &KeyStats, key: char) -> Heat {
    match stats.accuracy(key) {
        None => Heat::Unknown,
        Some(a) if a >= 0.95 => Heat::Good,
        Some(a) if a >= 0.85 => Heat::Mid,
        Some(_) => Heat::Bad,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats_with(key: char, hits: usize, misses: usize) -> KeyStats {
        let mut s = KeyStats::default();
        s.keys.insert(key, (hits, misses));
        s
    }

    #[test]
    fn unknown_when_never_typed() {
        assert_eq!(heat_for(&KeyStats::default(), 'e'), Heat::Unknown);
    }

    #[test]
    fn buckets_by_accuracy() {
        assert_eq!(heat_for(&stats_with('e', 100, 0), 'e'), Heat::Good); // 1.0
        assert_eq!(heat_for(&stats_with('e', 90, 10), 'e'), Heat::Mid); // 0.90
        assert_eq!(heat_for(&stats_with('e', 70, 30), 'e'), Heat::Bad); // 0.70
    }
}
