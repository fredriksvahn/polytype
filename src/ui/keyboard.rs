//! Pure view model for the on-screen keyboard: which physical position to
//! highlight next, and which hand a position belongs to.

use crate::layout::Layout;
use crate::stats::KeyStats;
use crate::ui::{heat, theme::Theme};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyColoring {
    Hands,
    Heat,
}

/// Render the on-screen keyboard for `layout` as 3 lines of styled spans.
/// `coloring` picks hand colors vs accuracy heat; `split` inserts a gap between
/// the hands; `highlight` reverse-highlights one grid position.
pub fn keyboard_lines(
    layout: &Layout,
    theme: &Theme,
    stats: &KeyStats,
    coloring: KeyColoring,
    split: bool,
    highlight: Option<usize>,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for row in 0..3 {
        let mut spans = Vec::new();
        for col in 0..10 {
            if split && col == 5 {
                spans.push(Span::raw("     "));
            }
            let pos = row * 10 + col;
            let ch = layout.char_at(pos).unwrap_or(' ');
            let mut style = match coloring {
                KeyColoring::Heat => Style::new().fg(theme.heat_color(heat::heat_for(stats, ch))),
                KeyColoring::Hands => Style::new().fg(theme.hand_color(hand_of(pos))),
            };
            if Some(pos) == highlight {
                style = style.bg(theme.cursor_bg).fg(theme.cursor_fg).bold();
            }
            spans.push(Span::styled(format!(" {ch}"), style));
        }
        lines.push(Line::from(spans));
    }
    lines
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

/// Hand for a grid position (row-major 30-key grid: cols 0-4 left, 5-9 right).
pub fn hand_of(pos: usize) -> Hand {
    if pos % 10 < 5 {
        Hand::Left
    } else {
        Hand::Right
    }
}

/// The grid position to highlight for the next expected char, if it is a key
/// on the target layout (spaces and unknown chars return None).
pub fn highlight_pos(target: &Layout, next_char: Option<char>) -> Option<usize> {
    next_char.and_then(|c| target.position_of(c.to_ascii_lowercase()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Finger {
    LPinky,
    LRing,
    LMiddle,
    LIndex,
    RIndex,
    RMiddle,
    RRing,
    RPinky,
}

impl Finger {
    pub fn label(self) -> &'static str {
        match self {
            Finger::LPinky => "L-pinky",
            Finger::LRing => "L-ring",
            Finger::LMiddle => "L-middle",
            Finger::LIndex => "L-index",
            Finger::RIndex => "R-index",
            Finger::RMiddle => "R-middle",
            Finger::RRing => "R-ring",
            Finger::RPinky => "R-pinky",
        }
    }
}

/// The finger that types a grid position (8-finger touch typing; index covers 2 columns).
pub fn finger_of(pos: usize) -> Finger {
    use Finger::*;
    match pos % 10 {
        0 => LPinky,
        1 => LRing,
        2 => LMiddle,
        3 => LIndex,
        4 => LIndex,
        5 => RIndex,
        6 => RIndex,
        7 => RMiddle,
        8 => RRing,
        _ => RPinky,
    }
}

/// Per-finger accuracy for `layout`, from accumulated per-key stats. Only fingers
/// with at least one typed key, sorted weakest first.
pub fn per_finger_accuracy(layout: &Layout, stats: &KeyStats) -> Vec<(Finger, f64)> {
    use std::collections::HashMap;
    let mut agg: HashMap<Finger, (usize, usize)> = HashMap::new();
    for pos in 0..crate::layout::GRID_LEN {
        if let Some(ch) = layout.char_at(pos) {
            if let Some((h, m)) = stats.keys.get(&ch) {
                let e = agg.entry(finger_of(pos)).or_insert((0, 0));
                e.0 += h;
                e.1 += m;
            }
        }
    }
    let mut out: Vec<(Finger, f64)> = agg
        .into_iter()
        .filter(|(_, (h, m))| h + m > 0)
        .map(|(f, (h, m))| (f, h as f64 / (h + m) as f64))
        .collect();
    out.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::builtin::load_registry;

    fn dhm() -> Layout {
        load_registry(None).unwrap()["colemak-dhm"].clone()
    }

    #[test]
    fn hand_split_is_columns() {
        assert_eq!(hand_of(0), Hand::Left); // top-left
        assert_eq!(hand_of(9), Hand::Right); // top-right
        assert_eq!(hand_of(13), Hand::Left); // home col 3
        assert_eq!(hand_of(16), Hand::Right); // home col 6
    }

    #[test]
    fn highlights_next_key_position() {
        let layout = dhm();
        // 'n' is on the colemak-dhm home row (right hand). Just assert it resolves.
        let pos = highlight_pos(&layout, Some('n')).unwrap();
        assert_eq!(layout.char_at(pos), Some('n'));
    }

    #[test]
    fn uppercase_next_char_highlights_lowercase_key() {
        let layout = dhm();
        let pos = highlight_pos(&layout, Some('N')).unwrap();
        assert_eq!(layout.char_at(pos), Some('n'));
    }

    #[test]
    fn space_and_unknown_have_no_highlight() {
        let layout = dhm();
        assert_eq!(highlight_pos(&layout, Some(' ')), None);
        assert_eq!(highlight_pos(&layout, None), None);
    }

    #[test]
    fn finger_of_maps_columns() {
        assert_eq!(finger_of(0), Finger::LPinky);
        assert_eq!(finger_of(4), Finger::LIndex);
        assert_eq!(finger_of(5), Finger::RIndex);
        assert_eq!(finger_of(9), Finger::RPinky);
        assert_eq!(finger_of(19), Finger::RPinky); // home-row ';' column
    }

    #[test]
    fn per_finger_aggregates_and_sorts() {
        let layout = dhm();
        let mut stats = KeyStats::default();
        // 'a' is home-row left pinky on colemak-dhm (pos 10); make it weak.
        stats.keys.insert('a', (6, 4)); // 0.60
                                        // 'o' right pinky-ish, strong
        stats.keys.insert('o', (10, 0)); // 1.0
        let pf = per_finger_accuracy(&layout, &stats);
        assert!(!pf.is_empty());
        assert!(pf[0].1 <= pf[pf.len() - 1].1, "sorted weakest first");
        assert!(pf.iter().any(|(f, _)| *f == Finger::LPinky));
    }

    #[test]
    fn keyboard_lines_split_widens_and_coloring_differs() {
        use crate::stats::KeyStats;
        use crate::ui::theme::Theme;
        let layout = dhm();
        let theme = Theme::default();
        let stats = KeyStats::default();
        let normal = keyboard_lines(&layout, &theme, &stats, KeyColoring::Hands, false, None);
        let split = keyboard_lines(&layout, &theme, &stats, KeyColoring::Hands, true, None);
        assert!(split[0].width() > normal[0].width(), "split adds a gap");
        // With empty stats, Heat colors untyped keys (unknown) — differs from hand color.
        let heat = keyboard_lines(&layout, &theme, &stats, KeyColoring::Heat, false, None);
        assert_ne!(
            heat[0].spans[0].style.fg, normal[0].spans[0].style.fg,
            "heat vs hands color differs"
        );
    }
}
