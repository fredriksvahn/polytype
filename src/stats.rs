//! Persisted per-key accuracy stats (for the heatmap and progress).

use crate::engine::KeyStat;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyStats {
    /// key char -> (hits, misses)
    #[serde(with = "char_key_map")]
    pub keys: HashMap<char, (usize, usize)>,
}

/// TOML table keys must be strings, so serialize the `char`-keyed map with
/// single-character string keys and parse them back on load.
mod char_key_map {
    use serde::de::{Deserialize, Deserializer, Error as _};
    use serde::ser::{Serialize, Serializer};
    use std::collections::HashMap;

    pub fn serialize<S>(
        map: &HashMap<char, (usize, usize)>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let stringified: HashMap<String, (usize, usize)> =
            map.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        stringified.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<char, (usize, usize)>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let stringified = HashMap::<String, (usize, usize)>::deserialize(deserializer)?;
        stringified
            .into_iter()
            .map(|(k, v)| {
                let mut chars = k.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok((c, v)),
                    _ => Err(D::Error::custom(format!("invalid single-char key: {k:?}"))),
                }
            })
            .collect()
    }
}

impl KeyStats {
    pub fn data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("polytype"))
    }

    pub fn load_from(path: &Path) -> Result<KeyStats> {
        if !path.exists() {
            return Ok(KeyStats::default());
        }
        let text = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Merge a finished session's per-key counts into the aggregate.
    pub fn merge(&mut self, per_key: &HashMap<char, KeyStat>) {
        for (k, stat) in per_key {
            let entry = self.keys.entry(*k).or_insert((0, 0));
            entry.0 += stat.hits;
            entry.1 += stat.misses;
        }
    }

    /// Accuracy for a key, or None if never typed.
    pub fn accuracy(&self, key: char) -> Option<f64> {
        self.keys.get(&key).map(|(h, m)| {
            let total = h + m;
            if total == 0 {
                1.0
            } else {
                *h as f64 / total as f64
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_accumulates() {
        let mut s = KeyStats::default();
        let mut pk = HashMap::new();
        pk.insert('a', KeyStat { hits: 3, misses: 1 });
        s.merge(&pk);
        s.merge(&pk);
        assert_eq!(s.keys[&'a'], (6, 2));
        assert!((s.accuracy('a').unwrap() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let p = std::env::temp_dir().join("polytype-test-stats.toml");
        let mut s = KeyStats::default();
        s.keys.insert('e', (10, 2));
        s.save_to(&p).unwrap();
        let loaded = KeyStats::load_from(&p).unwrap();
        assert_eq!(loaded, s);
        std::fs::remove_file(&p).ok();
    }
}
