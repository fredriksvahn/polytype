//! Session history log + derived stats (~/.local/share/polytype/history.csv).

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub ts: u64,
    pub wpm: f64,
    pub accuracy: f64,
    pub mode: String,
    pub layout: String,
}

pub fn history_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("polytype").join("history.csv"))
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn append_to(path: &Path, s: &Session) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let line = format!(
        "{},{:.2},{:.4},{},{}\n",
        s.ts, s.wpm, s.accuracy, s.mode, s.layout
    );
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(line.as_bytes())
}

pub fn load_from(path: &Path) -> Vec<Session> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<Session> {
    let mut p = line.splitn(5, ',');
    let ts = p.next()?.trim().parse().ok()?;
    let wpm = p.next()?.trim().parse().ok()?;
    let accuracy = p.next()?.trim().parse().ok()?;
    let mode = p.next()?.to_string();
    let layout = p.next()?.to_string();
    Some(Session {
        ts,
        wpm,
        accuracy,
        mode,
        layout,
    })
}

pub fn append(s: &Session) {
    if let Some(p) = history_path() {
        let _ = append_to(&p, s);
    }
}

pub fn load() -> Vec<Session> {
    history_path().map(|p| load_from(&p)).unwrap_or_default()
}

pub fn best_wpm(sessions: &[Session]) -> f64 {
    sessions.iter().map(|s| s.wpm).fold(0.0, f64::max)
}

pub fn avg_wpm(sessions: &[Session]) -> f64 {
    if sessions.is_empty() {
        0.0
    } else {
        sessions.iter().map(|s| s.wpm).sum::<f64>() / sessions.len() as f64
    }
}

pub fn recent_wpm(sessions: &[Session], n: usize) -> Vec<f64> {
    let start = sessions.len().saturating_sub(n);
    sessions[start..].iter().map(|s| s.wpm).collect()
}

/// ASCII sparkline of values using 8 block levels.
pub fn sparkline(values: &[f64]) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() {
        return String::new();
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1e-9);
    values
        .iter()
        .map(|v| {
            let t = ((v - min) / span * (BARS.len() as f64 - 1.0)).round() as usize;
            BARS[t.min(BARS.len() - 1)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(ts: u64, wpm: f64) -> Session {
        Session {
            ts,
            wpm,
            accuracy: 0.95,
            mode: "words".into(),
            layout: "colemak-dhm".into(),
        }
    }

    #[test]
    fn append_and_load_roundtrip() {
        let p = std::env::temp_dir().join("polytype-test-history.csv");
        std::fs::remove_file(&p).ok();
        append_to(&p, &s(1, 50.0)).unwrap();
        append_to(&p, &s(2, 60.0)).unwrap();
        let loaded = load_from(&p);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].wpm, 50.0);
        assert_eq!(loaded[1].layout, "colemak-dhm");
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn skips_malformed_lines() {
        let p = std::env::temp_dir().join("polytype-test-history2.csv");
        std::fs::write(&p, "garbage\n3,70.00,0.9000,words,qwerty\n").unwrap();
        let loaded = load_from(&p);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].wpm, 70.0);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn aggregates() {
        let v = vec![s(1, 50.0), s(2, 90.0), s(3, 70.0)];
        assert_eq!(best_wpm(&v), 90.0);
        assert!((avg_wpm(&v) - 70.0).abs() < 1e-9);
        assert_eq!(recent_wpm(&v, 2), vec![90.0, 70.0]);
        assert_eq!(best_wpm(&[]), 0.0);
    }

    #[test]
    fn sparkline_levels() {
        assert_eq!(sparkline(&[]), "");
        assert_eq!(sparkline(&[5.0]).chars().count(), 1);
        let sp = sparkline(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(sp.chars().count(), 4);
        assert_eq!(sp.chars().next().unwrap(), '▁');
        assert_eq!(sp.chars().last().unwrap(), '█');
    }
}
