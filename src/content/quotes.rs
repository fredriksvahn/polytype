//! Quote/sentence content: bundled + user quotes, length filtering, and
//! normalization to characters typeable on the target layout.

use crate::content::decorate::available_punct;
use crate::error::Result;
use crate::layout::Layout;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;
use std::path::Path;

const ENGLISH: &str = include_str!("../../assets/quotes/english.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteLength {
    All,
    Short,
    Medium,
    Long,
}

impl QuoteLength {
    pub fn parse(s: &str) -> QuoteLength {
        match s.trim().to_lowercase().as_str() {
            "short" => QuoteLength::Short,
            "medium" => QuoteLength::Medium,
            "long" => QuoteLength::Long,
            _ => QuoteLength::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            QuoteLength::All => "all",
            QuoteLength::Short => "short",
            QuoteLength::Medium => "medium",
            QuoteLength::Long => "long",
        }
    }

    fn matches(self, len: usize) -> bool {
        match self {
            QuoteLength::All => true,
            QuoteLength::Short => len < 120,
            QuoteLength::Medium => (120..=300).contains(&len),
            QuoteLength::Long => len > 300,
        }
    }
}

pub fn bundled() -> Vec<String> {
    parse_lines(ENGLISH)
}

pub fn from_dir(dir: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("txt") {
                out.extend(parse_lines(&std::fs::read_to_string(&path)?));
            }
        }
    }
    Ok(out)
}

fn parse_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

/// Pick a random quote matching the length filter.
pub fn pick<R: Rng>(quotes: &[String], length: QuoteLength, rng: &mut R) -> Option<String> {
    let pool: Vec<&String> = quotes
        .iter()
        .filter(|q| length.matches(q.chars().count()))
        .collect();
    pool.choose(rng).map(|q| (*q).clone())
}

/// Keep letters (any case), digits, spaces, and punctuation on the layout grid;
/// drop everything else; collapse repeated spaces; trim.
pub fn normalize(quote: &str, layout: &Layout) -> String {
    let punct: HashSet<char> = available_punct(layout).into_iter().collect();
    let mut out = String::with_capacity(quote.len());
    let mut prev_space = false;
    for c in quote.chars() {
        let keep = c.is_ascii_alphabetic() || c.is_ascii_digit() || c == ' ' || punct.contains(&c);
        if !keep {
            continue;
        }
        if c == ' ' {
            if prev_space {
                continue;
            }
            prev_space = true;
        } else {
            prev_space = false;
        }
        out.push(c);
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::builtin::load_registry;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn layout(name: &str) -> Layout {
        load_registry(None).unwrap()[name].clone()
    }

    #[test]
    fn parse_defaults_to_all() {
        assert_eq!(QuoteLength::parse("zzz"), QuoteLength::All);
        assert_eq!(QuoteLength::parse("Medium"), QuoteLength::Medium);
    }

    #[test]
    fn pick_respects_length() {
        let quotes = vec![
            "short one".to_string(), // < 120
            "x".repeat(150),         // medium
            "y".repeat(400),         // long
        ];
        let mut rng = StdRng::seed_from_u64(1);
        let s = pick(&quotes, QuoteLength::Short, &mut rng).unwrap();
        assert!(s.chars().count() < 120);
        let l = pick(&quotes, QuoteLength::Long, &mut rng).unwrap();
        assert!(l.chars().count() > 300);
    }

    #[test]
    fn normalize_keeps_caps_and_grid_punct_drops_rest() {
        // qwerty grid has '.' ',' but not '"' or '!'
        let q = "Hello, \u{201c}World\u{201d}! 42.";
        let out = normalize(q, &layout("qwerty"));
        assert!(out.contains('H'), "capitals kept");
        assert!(out.contains("42"), "digits kept");
        assert!(out.contains(','), "grid comma kept");
        assert!(!out.contains('!'), "shifted punctuation dropped");
        assert!(!out.contains('\u{201c}'), "smart quote dropped");
    }

    #[test]
    fn bundled_is_nonempty() {
        assert!(!bundled().is_empty());
    }
}
