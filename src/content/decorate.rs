//! Inject punctuation and numbers into a word list, staying within what the
//! target layout can actually type (only on-grid punctuation; digits pass
//! through the remapper unchanged so they work on any layout).

use crate::layout::Layout;
use rand::Rng;

/// Curated punctuation candidates.
const PUNCT: &[char] = &['.', ',', ';', '\'', '/', '-'];

/// Punctuation chars that exist on this layout's key grid.
pub fn available_punct(layout: &Layout) -> Vec<char> {
    PUNCT
        .iter()
        .copied()
        .filter(|c| layout.position_of(*c).is_some())
        .collect()
}

/// Decorate words in place: replace some with number tokens, append punctuation
/// to some. No-op when both toggles are false.
pub fn apply<R: Rng>(
    words: &mut [String],
    layout: &Layout,
    punctuation: bool,
    numbers: bool,
    rng: &mut R,
) {
    let punct = if punctuation {
        available_punct(layout)
    } else {
        Vec::new()
    };
    for w in words.iter_mut() {
        if numbers && rng.gen_bool(0.1) {
            *w = rng.gen_range(0..1000).to_string();
            continue; // a number token is not also punctuated
        }
        if !punct.is_empty() && rng.gen_bool(0.15) {
            let p = punct[rng.gen_range(0..punct.len())];
            w.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::builtin::load_registry;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashSet;

    fn layout(name: &str) -> Layout {
        load_registry(None).unwrap()[name].clone()
    }

    #[test]
    fn no_toggles_leaves_words_unchanged() {
        let mut w = vec!["the".to_string(), "fox".to_string()];
        let orig = w.clone();
        apply(
            &mut w,
            &layout("qwerty"),
            false,
            false,
            &mut StdRng::seed_from_u64(1),
        );
        assert_eq!(w, orig);
    }

    #[test]
    fn punctuation_only_uses_grid_chars() {
        let lay = layout("graphite");
        let avail: HashSet<char> = available_punct(&lay).into_iter().collect();
        assert!(!avail.contains(&','), "graphite grid indeed lacks a comma");
        let mut w: Vec<String> = (0..300).map(|_| "word".to_string()).collect();
        apply(&mut w, &lay, true, false, &mut StdRng::seed_from_u64(7));
        for token in &w {
            for c in token.chars().filter(|c| !c.is_alphabetic()) {
                assert!(avail.contains(&c), "punct {c:?} not on graphite grid");
            }
        }
    }

    #[test]
    fn numbers_inject_digit_tokens() {
        let mut w: Vec<String> = (0..300).map(|_| "word".to_string()).collect();
        apply(
            &mut w,
            &layout("qwerty"),
            false,
            true,
            &mut StdRng::seed_from_u64(3),
        );
        assert!(
            w.iter()
                .any(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_digit())),
            "some number tokens injected"
        );
    }
}
