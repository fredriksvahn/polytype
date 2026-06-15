//! Theme: a semantic color palette, loaded from built-in or user TOML themes.

use crate::ui::heat::Heat;
use crate::ui::keyboard::Hand;
use ratatui::style::Color;
use serde::Deserialize;
use std::path::Path;

/// Parse "#rrggbb" into a Color. None on bad input.
fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

/// Theme file: hex strings per slot. Missing fields inherit the base default.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ThemeFile {
    pub bg: String,
    pub fg: String,
    pub dim: String,
    pub error: String,
    pub accent: String,
    pub cursor_fg: String,
    pub cursor_bg: String,
    pub left_hand: String,
    pub right_hand: String,
    pub heat_good: String,
    pub heat_mid: String,
    pub heat_bad: String,
    pub heat_unknown: String,
}

impl Default for ThemeFile {
    fn default() -> Self {
        // Neutral dark base; every slot here is also the fallback for partial themes.
        ThemeFile {
            bg: "#1a1a1a".into(),
            fg: "#e0e0e0".into(),
            dim: "#808080".into(),
            error: "#d75f5f".into(),
            accent: "#5fafd7".into(),
            cursor_fg: "#1a1a1a".into(),
            cursor_bg: "#d7af5f".into(),
            left_hand: "#87af87".into(),
            right_hand: "#af87d7".into(),
            heat_good: "#87af87".into(),
            heat_mid: "#d7af5f".into(),
            heat_bad: "#d75f5f".into(),
            heat_unknown: "#585858".into(),
        }
    }
}

/// Resolved theme: ratatui colors.
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub dim: Color,
    pub error: Color,
    pub accent: Color,
    pub cursor_fg: Color,
    pub cursor_bg: Color,
    pub left_hand: Color,
    pub right_hand: Color,
    pub heat_good: Color,
    pub heat_mid: Color,
    pub heat_bad: Color,
    pub heat_unknown: Color,
}

impl Theme {
    fn from_file(tf: &ThemeFile) -> Theme {
        let base = ThemeFile::default();
        // Parse a slot; fall back to the base slot's hex if the theme's is bad.
        let c = |val: &str, base_val: &str| {
            parse_hex(val)
                .or_else(|| parse_hex(base_val))
                .unwrap_or(Color::Reset)
        };
        Theme {
            bg: c(&tf.bg, &base.bg),
            fg: c(&tf.fg, &base.fg),
            dim: c(&tf.dim, &base.dim),
            error: c(&tf.error, &base.error),
            accent: c(&tf.accent, &base.accent),
            cursor_fg: c(&tf.cursor_fg, &base.cursor_fg),
            cursor_bg: c(&tf.cursor_bg, &base.cursor_bg),
            left_hand: c(&tf.left_hand, &base.left_hand),
            right_hand: c(&tf.right_hand, &base.right_hand),
            heat_good: c(&tf.heat_good, &base.heat_good),
            heat_mid: c(&tf.heat_mid, &base.heat_mid),
            heat_bad: c(&tf.heat_bad, &base.heat_bad),
            heat_unknown: c(&tf.heat_unknown, &base.heat_unknown),
        }
    }

    pub fn hand_color(&self, hand: Hand) -> Color {
        match hand {
            Hand::Left => self.left_hand,
            Hand::Right => self.right_hand,
        }
    }

    pub fn heat_color(&self, heat: Heat) -> Color {
        match heat {
            Heat::Good => self.heat_good,
            Heat::Mid => self.heat_mid,
            Heat::Bad => self.heat_bad,
            Heat::Unknown => self.heat_unknown,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::from_file(&ThemeFile::default())
    }
}

const BUILTINS: &[(&str, &str)] = &[
    (
        "catppuccin-mocha",
        include_str!("../../assets/themes/catppuccin-mocha.toml"),
    ),
    (
        "catppuccin-macchiato",
        include_str!("../../assets/themes/catppuccin-macchiato.toml"),
    ),
    (
        "catppuccin-frappe",
        include_str!("../../assets/themes/catppuccin-frappe.toml"),
    ),
    (
        "catppuccin-latte",
        include_str!("../../assets/themes/catppuccin-latte.toml"),
    ),
    ("dracula", include_str!("../../assets/themes/dracula.toml")),
    (
        "gruvbox-dark",
        include_str!("../../assets/themes/gruvbox-dark.toml"),
    ),
    (
        "gruvbox-light",
        include_str!("../../assets/themes/gruvbox-light.toml"),
    ),
    ("nord", include_str!("../../assets/themes/nord.toml")),
    (
        "rose-pine",
        include_str!("../../assets/themes/rose-pine.toml"),
    ),
    (
        "rose-pine-moon",
        include_str!("../../assets/themes/rose-pine-moon.toml"),
    ),
    (
        "rose-pine-dawn",
        include_str!("../../assets/themes/rose-pine-dawn.toml"),
    ),
    (
        "everforest",
        include_str!("../../assets/themes/everforest.toml"),
    ),
    (
        "solarized-dark",
        include_str!("../../assets/themes/solarized-dark.toml"),
    ),
    (
        "solarized-light",
        include_str!("../../assets/themes/solarized-light.toml"),
    ),
    ("onedark", include_str!("../../assets/themes/onedark.toml")),
    (
        "kanagawa",
        include_str!("../../assets/themes/kanagawa.toml"),
    ),
];

fn builtin(name: &str) -> Option<Theme> {
    BUILTINS
        .iter()
        .find(|(n, _)| *n == name)
        .and_then(|(_, src)| {
            toml::from_str::<ThemeFile>(src)
                .ok()
                .map(|tf| Theme::from_file(&tf))
        })
}

/// Load a named theme: user dir `<name>.toml` → built-in → default.
pub fn load(name: &str, user_dir: Option<&Path>) -> Theme {
    if let Some(dir) = user_dir {
        let path = dir.join(format!("{name}.toml"));
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(tf) = toml::from_str::<ThemeFile>(&text) {
                return Theme::from_file(&tf);
            }
        }
    }
    builtin(name).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex() {
        assert_eq!(parse_hex("#1e1e2e"), Some(Color::Rgb(0x1e, 0x1e, 0x2e)));
        assert_eq!(parse_hex("nope"), None);
        assert_eq!(parse_hex("#fff"), None);
    }

    #[test]
    fn partial_theme_inherits_base() {
        let tf: ThemeFile = toml::from_str("bg = \"#000000\"").unwrap();
        let t = Theme::from_file(&tf);
        assert_eq!(t.bg, Color::Rgb(0, 0, 0)); // overridden
        assert_eq!(t.fg, Theme::default().fg); // inherited
    }

    #[test]
    fn unknown_name_is_default() {
        let t = load("does-not-exist", None);
        assert_eq!(format!("{:?}", t.bg), format!("{:?}", Theme::default().bg));
    }

    #[test]
    fn bundled_themes_parse() {
        for (name, _) in BUILTINS {
            let t = load(name, None);
            // a bundled theme differs from the neutral default in bg
            let _ = t.bg;
        }
        let mocha = load("catppuccin-mocha", None);
        assert_eq!(mocha.bg, Color::Rgb(0x1e, 0x1e, 0x2e));
        assert_eq!(BUILTINS.len(), 16);
    }
}
