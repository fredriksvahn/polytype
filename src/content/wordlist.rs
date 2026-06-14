//! Word sources: bundled English, custom files, stdin.

use crate::error::Result;
use std::io::Read;
use std::path::Path;

const ENGLISH: &str = include_str!("../../assets/wordlists/english.txt");

/// Load the bundled English wordlist.
pub fn english() -> Vec<String> {
    parse_words(ENGLISH)
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
}
