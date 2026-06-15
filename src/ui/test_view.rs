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
                Constraint::Length(3), // text window
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

        // Target text colored by per-position outcome, wrapped into lines, then
        // scrolled so the cursor's line stays visible (middle of a 3-line window).
        let cursor = self.runner.cursor();
        let cells = self.runner.cells();
        let chars: Vec<char> = self.target_text.chars().collect();
        let word_err = word_has_error(&chars, cells);

        let width = chunks[1].width.max(1) as usize;
        let line_idx = wrap_line_index(&chars, width);
        let total_lines = line_idx.last().map(|l| l + 1).unwrap_or(1);
        let cursor_line = if chars.is_empty() {
            0
        } else {
            line_idx[cursor.min(chars.len() - 1)]
        };
        let start = window_start(cursor_line, total_lines, WINDOW_HEIGHT);

        let mut line_spans: Vec<Vec<Span>> = vec![Vec::new(); total_lines];
        for (i, &c) in chars.iter().enumerate() {
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
            line_spans[line_idx[i]].push(Span::styled(c.to_string(), style));
        }

        let visible: Vec<Line> = line_spans
            .into_iter()
            .skip(start)
            .take(WINDOW_HEIGHT)
            .map(Line::from)
            .collect();
        f.render_widget(Paragraph::new(visible), chunks[1]);

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

const WINDOW_HEIGHT: usize = 3;

/// Greedy word-wrap: the line index for each char at the given width. Words
/// break at spaces; a word wider than `width` is hard-split.
fn wrap_line_index(chars: &[char], width: usize) -> Vec<usize> {
    let width = width.max(1);
    let mut idx = vec![0usize; chars.len()];
    let mut line = 0usize;
    let mut col = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == ' ' {
            if col >= width {
                line += 1;
                col = 0;
            }
            idx[i] = line;
            col += 1;
            i += 1;
        } else {
            let mut j = i;
            while j < chars.len() && chars[j] != ' ' {
                j += 1;
            }
            let word_len = j - i;
            if word_len <= width && col + word_len > width {
                line += 1;
                col = 0;
            }
            for idx_k in idx.iter_mut().take(j).skip(i) {
                if col >= width {
                    line += 1;
                    col = 0;
                }
                *idx_k = line;
                col += 1;
            }
            i = j;
        }
    }
    idx
}

/// First visible line so the cursor's line sits on the middle row (top at the
/// start, clamped near the end).
fn window_start(cursor_line: usize, total_lines: usize, height: usize) -> usize {
    if total_lines <= height {
        return 0;
    }
    cursor_line.saturating_sub(1).min(total_lines - height)
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

    #[test]
    fn long_text_wraps_instead_of_truncating() {
        let reg = load_registry(None).unwrap();
        let target = reg["qwerty"].clone();
        let remapper = Remapper::new(reg["qwerty"].clone(), target.clone());
        let text = "alpha bravo charlie delta"; // wider than a 12-col window
        let runner = SessionRunner::new(text, remapper, Mode::Words(4));
        let stats = KeyStats::default();

        let mut term = Terminal::new(TestBackend::new(12, 8)).unwrap();
        term.draw(|f| {
            TestView {
                runner: &runner,
                target_text: text,
                target_layout: &target,
                stats: &stats,
                show_keyboard: false,
                show_heatmap: false,
            }
            .render(f, f.area());
        })
        .unwrap();

        // Without wrapping, "delta" would be truncated off the first row.
        assert!(
            buffer_text(&term).contains("delta"),
            "later word wrapped into view"
        );
    }

    #[test]
    fn cursor_stays_visible_when_scrolled() {
        let reg = load_registry(None).unwrap();
        let target = reg["qwerty"].clone();
        let remapper = Remapper::new(reg["qwerty"].clone(), target.clone());
        let text = "alpha bravo charlie delta echo foxtrot golf hotel india";
        let mut runner = SessionRunner::new(text, remapper, Mode::Words(9));
        // Advance the cursor far into the text (free mode advances on any key).
        for _ in 0..40 {
            runner.type_char('x');
        }
        let stats = KeyStats::default();

        let mut term = Terminal::new(TestBackend::new(10, 8)).unwrap();
        term.draw(|f| {
            TestView {
                runner: &runner,
                target_text: text,
                target_layout: &target,
                stats: &stats,
                show_keyboard: false,
                show_heatmap: false,
            }
            .render(f, f.area());
        })
        .unwrap();

        let content: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(!content.contains("alpha"), "early word scrolled off");
        assert!(content.contains("india"), "cursor's region is visible");
    }

    #[test]
    fn wrap_breaks_at_spaces() {
        let chars: Vec<char> = "alpha bravo charlie delta".chars().collect();
        let idx = wrap_line_index(&chars, 12);
        // "alpha bravo " fits on line 0; "charlie" line 1; "delta" line 2
        assert_eq!(idx[0], 0); // 'a' of alpha
        assert_eq!(idx[chars.iter().position(|&c| c == 'c').unwrap()], 1); // 'c' of charlie
        assert_eq!(idx[chars.len() - 1], 2); // 'a' of delta
    }

    #[test]
    fn wrap_hard_splits_overlong_word() {
        let chars: Vec<char> = "abcdefghij".chars().collect(); // 10 chars
        let idx = wrap_line_index(&chars, 4);
        assert_eq!(idx[0], 0);
        assert_eq!(idx[3], 0);
        assert_eq!(idx[4], 1);
        assert_eq!(idx[9], 2);
    }

    #[test]
    fn window_start_centers_then_clamps() {
        assert_eq!(window_start(0, 10, 3), 0); // start: cursor on top
        assert_eq!(window_start(5, 10, 3), 4); // middle
        assert_eq!(window_start(9, 10, 3), 7); // clamp to last 3
        assert_eq!(window_start(2, 3, 3), 0); // fits entirely
    }

    fn buffer_text(term: &Terminal<TestBackend>) -> String {
        let buf = term.backend().buffer();
        buf.content().iter().map(|c| c.symbol()).collect()
    }
}
