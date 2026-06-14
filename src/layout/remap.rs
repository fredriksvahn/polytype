//! Remaps a received char (produced by the source layout) to the char the
//! target layout produces at the same physical position.

use crate::layout::Layout;

pub struct Remapper {
    source: Layout,
    target: Layout,
}

impl Remapper {
    pub fn new(source: Layout, target: Layout) -> Self {
        Self { source, target }
    }

    /// Returns the target-layout char for a char received from the source layout.
    /// Returns the char unchanged if it is not on the source grid (e.g. space).
    pub fn remap(&self, received: char) -> char {
        match self.source.position_of(received) {
            Some(pos) => self.target.char_at(pos).unwrap_or(received),
            None => received,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::builtin::builtin_registry;

    fn remapper(src: &str, tgt: &str) -> Remapper {
        let reg = builtin_registry().unwrap();
        Remapper::new(reg[src].clone(), reg[tgt].clone())
    }

    #[test]
    fn qwerty_to_colemak() {
        let r = remapper("qwerty", "colemak");
        // QWERTY 'e' physical key produces Colemak 'f'
        assert_eq!(r.remap('e'), 'f');
        // QWERTY 'j' stays 'n' on Colemak
        assert_eq!(r.remap('j'), 'n');
    }

    #[test]
    fn qwerty_to_colemak_dhm() {
        let r = remapper("qwerty", "colemak-dhm");
        // QWERTY 'z' key -> 'x' under the angle mod
        assert_eq!(r.remap('z'), 'x');
        // QWERTY 'b' key -> 'z'
        assert_eq!(r.remap('b'), 'z');
    }

    #[test]
    fn colemak_to_graphite_uses_source_inverse() {
        // User runs Colemak in the OS, trains Graphite.
        let r = remapper("colemak", "graphite");
        // Whatever char Colemak produces at a position maps to Graphite's char there.
        // 'a' is at position 10 in both qwerty-grid and colemak (home start) -> graphite 'n'
        assert_eq!(r.remap('a'), 'n');
    }

    #[test]
    fn passes_through_unknown_chars() {
        let r = remapper("qwerty", "colemak");
        assert_eq!(r.remap(' '), ' ');
    }
}
