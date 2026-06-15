//! The stats screen: best/avg/run-count, a WPM sparkline, and weak fingers.

use crate::history;
use crate::layout::Layout;
use crate::stats::KeyStats;
use crate::ui::keyboard::per_finger_accuracy;
use crate::ui::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout as LLayout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, layout: &Layout, stats: &KeyStats, theme: &Theme) {
    let sessions = history::load();
    let weak = per_finger_accuracy(layout, stats);
    let weak_str = if weak.is_empty() {
        "no data yet".to_string()
    } else {
        weak.iter()
            .take(4)
            .map(|(fg, a)| format!("{} {:.0}%", fg.label(), a * 100.0))
            .collect::<Vec<_>>()
            .join("  ")
    };

    let summary = if sessions.is_empty() {
        "no runs yet".to_string()
    } else {
        format!(
            "best {:.0}   avg {:.0}   ({} runs)",
            history::best_wpm(&sessions),
            history::avg_wpm(&sessions),
            sessions.len()
        )
    };
    let spark = history::sparkline(&history::recent_wpm(&sessions, 30));

    let lines = vec![
        Line::from("stats").style(Style::new().fg(theme.accent)),
        Line::from(""),
        Line::from(summary).style(Style::new().fg(theme.fg)),
        Line::from(format!("wpm  {spark}")).style(Style::new().fg(theme.fg)),
        Line::from(""),
        Line::from(format!("weak fingers:  {weak_str}")).style(Style::new().fg(theme.dim)),
        Line::from(""),
        Line::from("any key = back").style(Style::new().fg(theme.dim)),
    ];
    let content_h = lines.len() as u16;
    let outer = LLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(content_h),
            Constraint::Fill(1),
        ])
        .split(area);
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), outer[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::builtin::load_registry;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_summary_and_fingers() {
        let layout = load_registry(None).unwrap()["colemak-dhm"].clone();
        let mut stats = KeyStats::default();
        stats.keys.insert('a', (6, 4));
        let theme = Theme::default();
        let mut term = Terminal::new(TestBackend::new(50, 16)).unwrap();
        term.draw(|f| render(f, f.area(), &layout, &stats, &theme))
            .unwrap();
        let content: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(content.contains("stats"));
        assert!(content.contains("weak fingers"));
    }
}
