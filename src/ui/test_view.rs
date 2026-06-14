//! Renders the test screen: status line, the target text with per-char
//! coloring, and (optionally) the on-screen keyboard.

use crate::app::runner::SessionRunner;
use crate::engine::Cell;
use crate::layout::Layout;
use crate::stats::KeyStats;
use crate::ui::keyboard::{hand_of, highlight_pos};
use crate::ui::{heat, theme};
use ratatui::layout::{Constraint, Direction, Layout as LLayout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub struct TestView<'a> {
    pub runner: &'a SessionRunner,
    pub target_text: &'a str,
    pub target_layout: &'a Layout,
    pub stats: &'a KeyStats,
    pub show_keyboard: bool,
    pub show_heatmap: bool,
}

impl TestView<'_> {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = LLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status
                Constraint::Min(3),    // text
                Constraint::Length(5), // keyboard
            ])
            .split(area);

        // Status line
        let score = self.runner.score();
        let status = format!(
            "wpm {:.0}  acc {:.0}%  {}",
            score.wpm,
            score.accuracy * 100.0,
            self.target_layout.name
        );
        f.render_widget(
            Paragraph::new(Line::from(status).style(Style::new().fg(theme::STATUS))),
            chunks[0],
        );

        // Target text colored by per-position outcome.
        let cursor = self.runner.cursor();
        let cells = self.runner.cells();
        let chars: Vec<char> = self.target_text.chars().collect();
        let word_err = word_has_error(&chars, cells);

        let spans: Vec<Span> = chars
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                let mut style = if i == cursor {
                    Style::new().fg(theme::CURSOR_FG).bg(theme::CURSOR_BG)
                } else {
                    match cells.get(i) {
                        Some(Cell::Correct) => Style::new().fg(theme::CORRECT),
                        Some(Cell::Wrong) => Style::new().fg(theme::WRONG),
                        _ => Style::new().fg(theme::TODO),
                    }
                };
                if word_err.get(i).copied().unwrap_or(false) {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                Span::styled(c.to_string(), style)
            })
            .collect();
        f.render_widget(Paragraph::new(Line::from(spans)), chunks[1]);

        // Keyboard
        if self.show_keyboard {
            let next = self.target_text.chars().nth(cursor);
            let hl = highlight_pos(self.target_layout, next);
            let kb = self.keyboard_lines(hl);
            f.render_widget(Paragraph::new(kb), chunks[2]);
        }
    }

    fn keyboard_lines(&self, highlight: Option<usize>) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for row in 0..3 {
            let mut spans = Vec::new();
            for col in 0..10 {
                let pos = row * 10 + col;
                let ch = self.target_layout.char_at(pos).unwrap_or(' ');
                let mut style = if self.show_heatmap {
                    Style::new().fg(theme::heat_color(heat::heat_for(self.stats, ch)))
                } else {
                    Style::new().fg(theme::hand_color(hand_of(pos)))
                };
                if Some(pos) == highlight {
                    style = style.bg(theme::CURSOR_BG).fg(theme::CURSOR_FG).bold();
                }
                spans.push(Span::styled(format!(" {ch}"), style));
            }
            lines.push(Line::from(spans));
        }
        lines
    }
}

/// For each char index, true if its word (run between spaces) contains a Wrong cell.
fn word_has_error(chars: &[char], cells: &[Cell]) -> Vec<bool> {
    let mut flags = vec![false; chars.len()];
    let mut start = 0;
    for i in 0..=chars.len() {
        let boundary = i == chars.len() || chars[i] == ' ';
        if boundary {
            let has_err = (start..i).any(|j| cells.get(j) == Some(&Cell::Wrong));
            if has_err {
                for flag in flags.iter_mut().take(i).skip(start) {
                    *flag = true;
                }
            }
            start = i + 1;
        }
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Mode;
    use crate::layout::builtin::load_registry;
    use crate::layout::remap::Remapper;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_target_text_and_status() {
        let reg = load_registry(None).unwrap();
        let target = reg["colemak-dhm"].clone();
        let remapper = Remapper::new(reg["qwerty"].clone(), target.clone());
        let runner = SessionRunner::new("the", remapper, Mode::Words(1));
        let stats = KeyStats::default();

        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| {
            TestView {
                runner: &runner,
                target_text: "the",
                target_layout: &target,
                stats: &stats,
                show_keyboard: true,
                show_heatmap: false,
            }
            .render(f, f.area());
        })
        .unwrap();

        let content = buffer_text(&term);
        assert!(content.contains("the"), "target text rendered");
        assert!(content.contains("colemak-dhm"), "status shows layout");
    }

    #[test]
    fn wrong_char_is_red_and_word_underlined() {
        let reg = load_registry(None).unwrap();
        let target = reg["qwerty"].clone();
        let remapper = Remapper::new(reg["qwerty"].clone(), target.clone());
        let mut runner = SessionRunner::new("ab cd", remapper, Mode::Words(2));
        runner.type_char('x'); // wrong for 'a' (free mode advances)
        let stats = KeyStats::default();

        let mut term = Terminal::new(TestBackend::new(40, 8)).unwrap();
        term.draw(|f| {
            TestView {
                runner: &runner,
                target_text: "ab cd",
                target_layout: &target,
                stats: &stats,
                show_keyboard: false,
                show_heatmap: false,
            }
            .render(f, f.area());
        })
        .unwrap();

        // Find the cell for 'a' (index 0) in the text row (row 1).
        let buf = term.backend().buffer();
        let cell = buf.cell((0, 1)).expect("cell at (0,1)");
        assert_eq!(cell.fg, theme::WRONG, "wrong char rendered red");
        assert!(
            cell.modifier.contains(ratatui::style::Modifier::UNDERLINED),
            "errored word underlined"
        );
    }

    fn buffer_text(term: &Terminal<TestBackend>) -> String {
        let buf = term.backend().buffer();
        buf.content().iter().map(|c| c.symbol()).collect()
    }
}
