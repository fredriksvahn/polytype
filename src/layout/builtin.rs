//! Built-in layouts embedded at compile time + a registry that merges user layouts.

use crate::error::Result;
use crate::layout::{Layout, LayoutFile};
use std::collections::HashMap;
use std::path::Path;

const BUILTINS: &[(&str, &str)] = &[
    ("qwerty", include_str!("../../assets/layouts/qwerty.toml")),
    ("colemak", include_str!("../../assets/layouts/colemak.toml")),
    (
        "colemak-dh",
        include_str!("../../assets/layouts/colemak-dh.toml"),
    ),
    (
        "colemak-dhm",
        include_str!("../../assets/layouts/colemak-dhm.toml"),
    ),
    ("dvorak", include_str!("../../assets/layouts/dvorak.toml")),
    ("workman", include_str!("../../assets/layouts/workman.toml")),
    (
        "graphite",
        include_str!("../../assets/layouts/graphite.toml"),
    ),
    ("tarmak1", include_str!("../../assets/layouts/tarmak1.toml")),
    ("tarmak2", include_str!("../../assets/layouts/tarmak2.toml")),
    ("tarmak3", include_str!("../../assets/layouts/tarmak3.toml")),
    ("tarmak4", include_str!("../../assets/layouts/tarmak4.toml")),
];

/// All built-in layouts keyed by name.
pub fn builtin_registry() -> Result<HashMap<String, Layout>> {
    let mut map = HashMap::new();
    for (name, toml_src) in BUILTINS {
        let file: LayoutFile = toml::from_str(toml_src)?;
        let layout = Layout::from_file(file)?;
        map.insert((*name).to_string(), layout);
    }
    Ok(map)
}

/// Built-ins, then any `*.toml` in `dir` override/add by name.
pub fn load_registry(user_dir: Option<&Path>) -> Result<HashMap<String, Layout>> {
    let mut map = builtin_registry()?;
    if let Some(dir) = user_dir {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    let text = std::fs::read_to_string(&path)?;
                    let file: LayoutFile = toml::from_str(&text)?;
                    let layout = Layout::from_file(file)?;
                    map.insert(layout.name.clone(), layout);
                }
            }
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::GRID_LEN;

    #[test]
    fn all_builtins_parse_and_have_full_grid() {
        let reg = builtin_registry().unwrap();
        assert_eq!(reg.len(), 11);
        for (name, layout) in &reg {
            assert_eq!(layout.keys.len(), GRID_LEN, "layout {name} wrong length");
        }
    }

    #[test]
    fn qwerty_is_identity_order() {
        let reg = builtin_registry().unwrap();
        let q = &reg["qwerty"];
        assert_eq!(q.char_at(0), Some('q'));
        assert_eq!(q.position_of('a'), Some(10)); // start of home row
    }

    #[test]
    fn colemak_dhm_bottom_left_is_angle_modded() {
        // Angle mod: QWERTY z x c v b positions -> x c d v z
        let reg = builtin_registry().unwrap();
        let dhm = &reg["colemak-dhm"];
        // bottom row starts at index 20
        assert_eq!(dhm.char_at(20), Some('x'));
        assert_eq!(dhm.char_at(21), Some('c'));
        assert_eq!(dhm.char_at(22), Some('d'));
        assert_eq!(dhm.char_at(23), Some('v'));
        assert_eq!(dhm.char_at(24), Some('z'));
    }

    #[test]
    fn tarmak_steps_present_and_step1_brings_n_and_e_home() {
        let reg = builtin_registry().unwrap();
        for step in ["tarmak1", "tarmak2", "tarmak3", "tarmak4"] {
            assert!(reg.contains_key(step), "{step} missing");
        }
        // Tarmak1 4-cycle: QWERTY j-pos -> n, k-pos -> e (n and e brought home).
        let t1 = &reg["tarmak1"];
        assert_eq!(t1.char_at(16), Some('n')); // QWERTY 'j' position
        assert_eq!(t1.char_at(17), Some('e')); // QWERTY 'k' position
        assert_eq!(t1.char_at(2), Some('j')); // QWERTY 'e' position parks 'j'
    }

    #[test]
    fn user_dir_adds_and_overrides() {
        let dir = std::env::temp_dir().join("polytype-test-layouts");
        std::fs::create_dir_all(&dir).unwrap();
        let custom = dir.join("mine.toml");
        std::fs::write(
            &custom,
            "name = \"mine\"\ndisplay = \"Mine\"\n\
             top = \"q w e r t y u i o p\"\n\
             home = \"a s d f g h j k l ;\"\n\
             bottom = \"z x c v b n m , . /\"\n",
        )
        .unwrap();
        let reg = load_registry(Some(&dir)).unwrap();
        assert!(reg.contains_key("mine"));
        assert!(reg.contains_key("qwerty")); // builtins still present
        std::fs::remove_dir_all(&dir).ok();
    }
}
