//! User configuration loaded from `~/.config/polytype/config.toml`.

use crate::error::Result;
use crate::keys::KeySpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub target_layout: String,
    pub source_layout: String,
    pub mode: String, // "words" | "timed" | "lesson"
    pub words: usize,
    pub time: u64,
    pub wordlist: String,
    pub theme: String,
    pub show_keyboard: bool,
    pub show_heatmap: bool,
    pub stop_on_error: bool,
    pub punctuation: bool,
    pub numbers: bool,
    pub split_keyboard: bool,
    pub keys: HashMap<String, KeySpec>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            target_layout: "colemak-dhm".into(),
            source_layout: "qwerty".into(),
            mode: "words".into(),
            words: 50,
            time: 30,
            wordlist: "english".into(),
            theme: "default".into(),
            show_keyboard: true,
            show_heatmap: false,
            stop_on_error: false,
            punctuation: false,
            numbers: false,
            split_keyboard: false,
            keys: HashMap::new(),
        }
    }
}

impl Config {
    /// Default config directory: `~/.config/polytype`.
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("polytype"))
    }

    /// Load config from a given path, falling back to defaults for missing fields.
    /// If the file does not exist, returns `Config::default()`.
    pub fn load_from(path: &Path) -> Result<Config> {
        if !path.exists() {
            return Ok(Config::default());
        }
        let text = std::fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&text)?;
        Ok(cfg)
    }

    /// Load from the default config dir.
    pub fn load() -> Result<Config> {
        match Self::config_dir() {
            Some(dir) => Self::load_from(&dir.join("config.toml")),
            None => Ok(Config::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_target_is_colemak_dhm() {
        assert_eq!(Config::default().target_layout, "colemak-dhm");
        assert_eq!(Config::default().source_layout, "qwerty");
    }

    #[test]
    fn missing_file_yields_defaults() {
        let cfg = Config::load_from(Path::new("/nonexistent/polytype/config.toml")).unwrap();
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn partial_file_merges_over_defaults() {
        let mut tmp = tempfile_path("partial.toml");
        let mut f = std::fs::File::create(&tmp).unwrap();
        writeln!(f, "target_layout = \"dvorak\"\nwords = 25").unwrap();
        let cfg = Config::load_from(&tmp).unwrap();
        assert_eq!(cfg.target_layout, "dvorak"); // overridden
        assert_eq!(cfg.words, 25); // overridden
        assert_eq!(cfg.source_layout, "qwerty"); // default kept
        assert!(cfg.show_keyboard); // default kept
        std::fs::remove_file(&tmp).ok();
        let _ = &mut tmp;
    }

    #[test]
    fn stop_on_error_defaults_false() {
        assert!(!Config::default().stop_on_error);
    }

    #[test]
    fn symbols_default_false() {
        let c = Config::default();
        assert!(!c.punctuation);
        assert!(!c.numbers);
    }

    #[test]
    fn keys_table_parses() {
        let p = std::env::temp_dir().join("polytype-test-keys.toml");
        std::fs::write(
            &p,
            "[keys]\ntest_restart = \"esc\"\nnav_down = [\"down\", \"j\"]\n",
        )
        .unwrap();
        let cfg = Config::load_from(&p).unwrap();
        assert!(cfg.keys.contains_key("test_restart"));
        assert!(cfg.keys.contains_key("nav_down"));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn keys_default_empty() {
        assert!(Config::default().keys.is_empty());
    }

    fn tempfile_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("polytype-test-{name}"))
    }
}
