//! Renders the results screen after a finished test.

use crate::engine::Score;
use crate::stats::KeyStats;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, score: &Score, session_keys: &KeyStats) {
    let cpm = (score.wpm * 5.0).round() as u64;
    let weak = weakest_keys(session_keys, 5);
    let weak_str = if weak.is_empty() {
        "none".to_string()
    } else {
        weak.iter()
            .map(|(k, a)| format!("{k} {:.0}%", a * 100.0))
            .collect::<Vec<_>>()
            .join("  ")
    };
    let lines = vec![
        Line::from(format!("cpm {cpm}")),
        Line::from(format!("wpm {:.0}", score.wpm)),
        Line::from(format!("acc {:.0}%", score.accuracy * 100.0)),
        Line::from(""),
        Line::from(format!("weakest:  {weak_str}")),
        Line::from(""),
        Line::from("tab next   esc menu   ctrl-c quit"),
    ];
    let content_h = lines.len() as u16;
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(content_h),
            Constraint::Fill(1),
        ])
        .split(area);
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), outer[1]);
}

/// The `n` keys with the lowest accuracy (only keys typed at least once).
fn weakest_keys(stats: &KeyStats, n: usize) -> Vec<(char, f64)> {
    let mut v: Vec<(char, f64)> = stats
        .keys
        .iter()
        .map(|(k, (h, m))| {
            (
                *k,
                if h + m == 0 {
                    1.0
                } else {
                    *h as f64 / (h + m) as f64
                },
            )
        })
        .filter(|(_, a)| *a < 1.0)
        .collect();
    v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    v.truncate(n);
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_score() {
        let score = Score {
            wpm: 62.0,
            accuracy: 0.97,
            correct: 50,
            typed: 52,
        };
        let stats = KeyStats::default();
        let mut term = Terminal::new(TestBackend::new(50, 10)).unwrap();
        term.draw(|f| render(f, f.area(), &score, &stats)).unwrap();
        let content: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(content.contains("wpm"));
        assert!(content.contains("62"));
        assert!(content.contains("97%"));
        assert!(content.contains("cpm"));
        assert!(content.contains("310")); // cpm = wpm(62) * 5
    }

    #[test]
    fn weakest_keys_sorted_ascending() {
        let mut stats = KeyStats::default();
        stats.keys.insert('a', (7, 3)); // 0.70
        stats.keys.insert('b', (9, 1)); // 0.90
        stats.keys.insert('c', (10, 0)); // 1.0 -> excluded
        let weak = weakest_keys(&stats, 5);
        assert_eq!(weak[0].0, 'a');
        assert_eq!(weak.len(), 2);
    }
}
