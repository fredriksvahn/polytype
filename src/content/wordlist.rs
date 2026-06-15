//! Word sources: bundled English, custom files, stdin.

use crate::error::Result;
use std::io::Read;
use std::path::Path;

const ENGLISH: &str = include_str!("../../assets/wordlists/english.txt");
const SWEDISH: &str = include_str!("../../assets/wordlists/swedish.txt");

/// Load the bundled English wordlist.
pub fn english() -> Vec<String> {
    parse_words(ENGLISH)
}

/// Load a named wordlist: a user file `<user_dir>/<name>.txt` wins, else a
/// bundled language ("english"/"swedish"), else fall back to English.
pub fn load_named(name: &str, user_dir: Option<&std::path::Path>) -> Vec<String> {
    if let Some(dir) = user_dir {
        let path = dir.join(format!("{name}.txt"));
        if path.is_file() {
            if let Ok(words) = from_file(&path) {
                if !words.is_empty() {
                    return words;
                }
            }
        }
    }
    match name {
        "swedish" => parse_words(SWEDISH),
        _ => english(),
    }
}

/// Load a custom wordlist file (one word per line; blank lines ignored).
pub fn from_file(path: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(parse_words(&text))
}

/// Read words from any reader (e.g. stdin): whitespace-separated.
pub fn from_reader<R: Read>(mut r: R) -> Result<Vec<String>> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    Ok(buf.split_whitespace().map(|s| s.to_string()).collect())
}

fn parse_words(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_nonempty() {
        assert!(english().len() >= 40);
        assert!(english().contains(&"the".to_string()));
    }

    #[test]
    fn reader_splits_on_whitespace() {
        let words = from_reader("hello world\nfoo  bar".as_bytes()).unwrap();
        assert_eq!(words, vec!["hello", "world", "foo", "bar"]);
    }

    #[test]
    fn file_one_word_per_line() {
        let p = std::env::temp_dir().join("polytype-test-wl.txt");
        std::fs::write(&p, "alpha\n\nbeta\n").unwrap();
        let words = from_file(&p).unwrap();
        assert_eq!(words, vec!["alpha", "beta"]);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn load_named_swedish_has_accented_words() {
        let w = load_named("swedish", None);
        assert!(!w.is_empty());
        assert!(
            w.iter().any(|word| word.chars().any(|c| "åäö".contains(c))),
            "swedish list contains å/ä/ö"
        );
    }

    #[test]
    fn load_named_unknown_falls_back_to_english() {
        assert_eq!(load_named("klingon", None), english());
    }

    #[test]
    fn load_named_user_dir_overrides() {
        let dir = std::env::temp_dir().join("polytype-test-wl-named");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("mylang.txt"), "alpha\nbeta\n").unwrap();
        let w = load_named("mylang", Some(&dir));
        assert_eq!(w, vec!["alpha".to_string(), "beta".to_string()]);
        std::fs::remove_dir_all(&dir).ok();
    }
}
