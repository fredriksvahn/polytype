//! Pure view model for the on-screen keyboard: which physical position to
//! highlight next, and which hand a position belongs to.

use crate::layout::Layout;

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
    next_char.and_then(|c| target.position_of(c))
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
    fn space_and_unknown_have_no_highlight() {
        let layout = dhm();
        assert_eq!(highlight_pos(&layout, Some(' ')), None);
        assert_eq!(highlight_pos(&layout, None), None);
    }
}
