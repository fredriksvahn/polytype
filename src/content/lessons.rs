//! Lesson progression: gradually introduce target-layout keys.

use crate::layout::Layout;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lesson {
    pub level: usize,
    pub name: String,
    /// Allowed target-layout letters for this lesson (cumulative).
    pub keys: Vec<char>,
}

/// Order in which physical positions are introduced, by grid index.
/// Home row first (indices 10..20), starting from the index/middle fingers,
/// then top row (0..10), then bottom row (20..30).
const INTRO_ORDER: &[usize] = &[
    // home row: left index/middle (13,12), right index/middle (16,17),
    13, 16, 12, 17, 11, 18, 10, 19, 14, 15,
    // top row
    3, 6, 2, 7, 1, 8, 0, 9, 4, 5,
    // bottom row
    23, 26, 22, 27, 21, 28, 20, 29, 24, 25,
];

/// Build the lesson progression for a target layout.
/// Each lesson adds 2 new keys (5 keys in the first lesson to make words possible).
pub fn progression(target: &Layout) -> Vec<Lesson> {
    let mut lessons = Vec::new();
    let mut acc: Vec<char> = Vec::new();
    let mut level = 0usize;
    let mut i = 0usize;
    while i < INTRO_ORDER.len() {
        let take = if level == 0 { 5 } else { 2 };
        for _ in 0..take {
            if let Some(&pos) = INTRO_ORDER.get(i) {
                if let Some(c) = target.char_at(pos) {
                    if c.is_alphabetic() {
                        acc.push(c);
                    }
                }
                i += 1;
            }
        }
        level += 1;
        lessons.push(Lesson {
            level,
            name: format!("Lektion {level}"),
            keys: acc.clone(),
        });
    }
    lessons
}

/// Generate drill text for a lesson from a wordlist, filtering to words whose
/// letters are all allowed. Falls back to random letter groups when fewer than
/// `min_words` qualify.
pub fn drill_text<R: Rng>(
    lesson: &Lesson,
    pool: &[String],
    word_count: usize,
    rng: &mut R,
) -> String {
    let allowed: std::collections::HashSet<char> = lesson.keys.iter().copied().collect();
    let usable: Vec<&String> = pool
        .iter()
        .filter(|w| !w.is_empty() && w.chars().all(|c| allowed.contains(&c)))
        .collect();

    if usable.len() >= 5 {
        (0..word_count)
            .map(|_| usable.choose(rng).unwrap().as_str())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        // fallback: random 3-5 letter groups from the allowed set
        let keys: Vec<char> = lesson.keys.clone();
        (0..word_count)
            .map(|_| {
                let len = rng.gen_range(3..=5);
                (0..len).map(|_| *keys.choose(rng).unwrap()).collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::builtin::builtin_registry;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn dhm() -> Layout {
        builtin_registry().unwrap()["colemak-dhm"].clone()
    }

    #[test]
    fn first_lesson_has_five_keys() {
        let p = progression(&dhm());
        assert_eq!(p[0].keys.len(), 5);
    }

    #[test]
    fn lessons_are_cumulative() {
        let p = progression(&dhm());
        for w in p.windows(2) {
            assert!(w[1].keys.len() >= w[0].keys.len());
            assert!(w[0].keys.iter().all(|c| w[1].keys.contains(c)));
        }
    }

    #[test]
    fn drill_only_uses_allowed_keys() {
        let p = progression(&dhm());
        let lesson = &p[0];
        let pool: Vec<String> = vec![]; // force fallback
        let mut rng = StdRng::seed_from_u64(1);
        let text = drill_text(lesson, &pool, 10, &mut rng);
        let allowed: std::collections::HashSet<char> = lesson.keys.iter().copied().collect();
        for c in text.chars().filter(|c| *c != ' ') {
            assert!(allowed.contains(&c), "char {c} not allowed");
        }
    }
}
