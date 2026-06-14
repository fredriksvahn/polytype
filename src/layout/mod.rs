//! Layout type: a positional char table aligned to the QWERTY 30-key grid.

use crate::error::{PolytypeError, Result};
use serde::Deserialize;

pub const GRID_LEN: usize = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub name: String,
    pub display: String,
    /// Produced char at each of the 30 physical positions, row-major.
    pub keys: Vec<char>,
}

/// TOML shape for a layout file.
#[derive(Debug, Deserialize)]
pub struct LayoutFile {
    pub name: String,
    pub display: String,
    pub top: String,
    pub home: String,
    pub bottom: String,
}

impl Layout {
    pub fn from_file(f: LayoutFile) -> Result<Layout> {
        let keys = parse_rows(&f.name, &[&f.top, &f.home, &f.bottom])?;
        Ok(Layout { name: f.name, display: f.display, keys })
    }

    pub fn position_of(&self, ch: char) -> Option<usize> {
        self.keys.iter().position(|&c| c == ch)
    }

    pub fn char_at(&self, pos: usize) -> Option<char> {
        self.keys.get(pos).copied()
    }
}

fn parse_rows(name: &str, rows: &[&str]) -> Result<Vec<char>> {
    let mut keys = Vec::with_capacity(GRID_LEN);
    for row in rows {
        let tokens: Vec<&str> = row.split_whitespace().collect();
        if tokens.len() != 10 {
            return Err(PolytypeError::InvalidLayout {
                name: name.to_string(),
                reason: format!("expected 10 keys per row, got {}", tokens.len()),
            });
        }
        for t in tokens {
            let mut chars = t.chars();
            let c = chars.next().ok_or_else(|| PolytypeError::InvalidLayout {
                name: name.to_string(),
                reason: "empty key token".into(),
            })?;
            keys.push(c);
        }
    }
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qwerty() -> Layout {
        Layout::from_file(LayoutFile {
            name: "qwerty".into(),
            display: "QWERTY".into(),
            top: "q w e r t y u i o p".into(),
            home: "a s d f g h j k l ;".into(),
            bottom: "z x c v b n m , . /".into(),
        })
        .unwrap()
    }

    #[test]
    fn parses_thirty_keys() {
        assert_eq!(qwerty().keys.len(), GRID_LEN);
    }

    #[test]
    fn position_and_char_roundtrip() {
        let q = qwerty();
        let pos = q.position_of('f').unwrap();
        assert_eq!(q.char_at(pos), Some('f'));
    }

    #[test]
    fn rejects_wrong_row_length() {
        let err = Layout::from_file(LayoutFile {
            name: "broken".into(),
            display: "Broken".into(),
            top: "q w e".into(),
            home: "a s d f g h j k l ;".into(),
            bottom: "z x c v b n m , . /".into(),
        });
        assert!(err.is_err());
    }
}
