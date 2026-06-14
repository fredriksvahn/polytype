//! Configurable keymap: actions bound to key chords, with built-in defaults
//! overridable from config.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    NavUp,
    NavDown,
    NavPrev,
    NavNext,
    Confirm,
    Quit,
    TestRestart,
    TestPanel,
    ResultsRestart,
    ResultsMenu,
    PanelCancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chord {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

/// Config value: a single key string or a list of them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeySpec {
    One(String),
    Many(Vec<String>),
}

impl KeySpec {
    fn keys(&self) -> Vec<String> {
        match self {
            KeySpec::One(s) => vec![s.clone()],
            KeySpec::Many(v) => v.clone(),
        }
    }
}

/// Parse a chord like "ctrl-c", "esc", "up", "j". Returns None if unparseable.
pub fn parse_chord(s: &str) -> Option<Chord> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    let mut parts: Vec<&str> = s.split('-').collect();
    let key = parts.pop()?;
    if key.is_empty() {
        return None;
    }
    let mut mods = KeyModifiers::NONE;
    for m in parts {
        match m {
            "ctrl" => mods |= KeyModifiers::CONTROL,
            "alt" => mods |= KeyModifiers::ALT,
            "shift" => mods |= KeyModifiers::SHIFT,
            _ => return None,
        }
    }
    let code = match key {
        "esc" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "enter" | "return" => KeyCode::Enter,
        "space" => KeyCode::Char(' '),
        "backspace" => KeyCode::Backspace,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        other => {
            let mut chars = other.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => KeyCode::Char(c),
                _ => return None,
            }
        }
    };
    Some(Chord { code, mods })
}

pub fn action_from_name(name: &str) -> Option<Action> {
    use Action::*;
    Some(match name {
        "nav_up" => NavUp,
        "nav_down" => NavDown,
        "nav_prev" => NavPrev,
        "nav_next" => NavNext,
        "confirm" => Confirm,
        "quit" => Quit,
        "test_restart" => TestRestart,
        "test_panel" => TestPanel,
        "results_restart" => ResultsRestart,
        "results_menu" => ResultsMenu,
        "panel_cancel" => PanelCancel,
        _ => return None,
    })
}

fn chords(specs: &[&str]) -> Vec<Chord> {
    specs.iter().filter_map(|s| parse_chord(s)).collect()
}

pub struct Keymap {
    map: HashMap<Action, Vec<Chord>>,
}

impl Keymap {
    pub fn defaults() -> Self {
        let mut map = HashMap::new();
        map.insert(Action::NavUp, chords(&["up", "k"]));
        map.insert(Action::NavDown, chords(&["down", "j"]));
        map.insert(Action::NavPrev, chords(&["left", "h"]));
        map.insert(Action::NavNext, chords(&["right", "l"]));
        map.insert(Action::Confirm, chords(&["enter"]));
        map.insert(Action::Quit, chords(&["ctrl-c"]));
        map.insert(Action::TestRestart, chords(&["esc"]));
        map.insert(Action::TestPanel, chords(&["tab"]));
        map.insert(Action::ResultsRestart, chords(&["tab", "enter"]));
        map.insert(Action::ResultsMenu, chords(&["esc"]));
        map.insert(Action::PanelCancel, chords(&["esc"]));
        Keymap { map }
    }

    pub fn with_overrides(overrides: &HashMap<String, KeySpec>) -> Self {
        let mut km = Keymap::defaults();
        for (name, spec) in overrides {
            if let Some(action) = action_from_name(name) {
                km.map.insert(
                    action,
                    spec.keys().iter().filter_map(|s| parse_chord(s)).collect(),
                );
            }
        }
        km
    }

    pub fn matches(&self, action: Action, key: &KeyEvent) -> bool {
        self.map.get(&action).is_some_and(|cs| {
            cs.iter()
                .any(|c| c.code == key.code && c.mods == key.modifiers)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_and_char_and_ctrl() {
        assert_eq!(
            parse_chord("esc"),
            Some(Chord {
                code: KeyCode::Esc,
                mods: KeyModifiers::NONE
            })
        );
        assert_eq!(
            parse_chord("j"),
            Some(Chord {
                code: KeyCode::Char('j'),
                mods: KeyModifiers::NONE
            })
        );
        assert_eq!(
            parse_chord("ctrl-c"),
            Some(Chord {
                code: KeyCode::Char('c'),
                mods: KeyModifiers::CONTROL
            })
        );
        assert_eq!(parse_chord(""), None);
        assert_eq!(parse_chord("bogusmod-x"), None);
    }

    #[test]
    fn defaults_have_vim_and_arrows() {
        let km = Keymap::defaults();
        let down = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let down_arrow = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(km.matches(Action::NavDown, &down));
        assert!(km.matches(Action::NavDown, &down_arrow));
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(km.matches(Action::TestRestart, &esc));
    }

    #[test]
    fn override_replaces_action_keys() {
        let mut ov = HashMap::new();
        ov.insert(
            "test_restart".to_string(),
            KeySpec::One("ctrl-r".to_string()),
        );
        let km = Keymap::with_overrides(&ov);
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert!(
            !km.matches(Action::TestRestart, &esc),
            "old binding replaced"
        );
        assert!(km.matches(Action::TestRestart, &ctrl_r));
        // other actions still default
        let down = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(km.matches(Action::NavDown, &down));
    }

    #[test]
    fn unknown_action_name_ignored() {
        let mut ov = HashMap::new();
        ov.insert("bogus".to_string(), KeySpec::One("x".to_string()));
        let _ = Keymap::with_overrides(&ov); // must not panic
    }
}
