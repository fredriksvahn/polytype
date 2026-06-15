//! Opening the config file in the user's $EDITOR.

use crate::config::Config;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Choose an editor: EDITOR, then VISUAL, then "vi". Pure for testability.
pub fn pick_editor(editor: Option<String>, visual: Option<String>) -> String {
    editor
        .filter(|s| !s.trim().is_empty())
        .or(visual.filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "vi".to_string())
}

fn resolve_editor() -> String {
    pick_editor(std::env::var("EDITOR").ok(), std::env::var("VISUAL").ok())
}

/// Ensure `dir/config.toml` exists (write a default template if missing); return it.
pub fn ensure_file_in(dir: &Path) -> std::io::Result<PathBuf> {
    let path = dir.join("config.toml");
    if !path.exists() {
        std::fs::create_dir_all(dir)?;
        let template = toml::to_string_pretty(&Config::default()).unwrap_or_default();
        std::fs::write(&path, template)?;
    }
    Ok(path)
}

fn ensure_config_file() -> std::io::Result<Option<PathBuf>> {
    match Config::config_dir() {
        Some(dir) => ensure_file_in(&dir).map(Some),
        None => Ok(None),
    }
}

/// Open the config file in the user's editor (blocking). The TUI must be
/// suspended (ratatui::restore) before calling this and re-inited after.
pub fn edit_config() {
    let Ok(Some(path)) = ensure_config_file() else {
        return;
    };
    let editor = resolve_editor();
    let mut parts = editor.split_whitespace();
    if let Some(cmd) = parts.next() {
        let args: Vec<&str> = parts.collect();
        let _ = Command::new(cmd).args(args).arg(&path).status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_editor_prefers_editor_then_visual_then_vi() {
        assert_eq!(
            pick_editor(Some("nvim".into()), Some("nano".into())),
            "nvim"
        );
        assert_eq!(pick_editor(None, Some("nano".into())), "nano");
        assert_eq!(pick_editor(Some("  ".into()), None), "vi");
        assert_eq!(pick_editor(None, None), "vi");
    }

    #[test]
    fn ensure_file_writes_default_when_missing() {
        let dir = std::env::temp_dir().join("polytype-test-editcfg");
        std::fs::remove_dir_all(&dir).ok();
        let path = ensure_file_in(&dir).unwrap();
        assert!(path.exists());
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("target_layout"), "wrote a default template");
        // Existing file is left untouched.
        std::fs::write(&path, "wordlist = \"swedish\"\n").unwrap();
        let path2 = ensure_file_in(&dir).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path2).unwrap(),
            "wordlist = \"swedish\"\n"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
