//! Menu state machine: pick mode + layout + (optional) lesson, then start.

use crate::app::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    ModeKind,
    Layout,
    LessonLevel,
    Punctuation,
    Numbers,
    Start,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeKind {
    Words,
    Timed,
    Lesson,
}

/// What the menu emits when the user activates "Start".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartRequest {
    pub mode: Mode,
    pub layout: String,
    pub punctuation: bool,
    pub numbers: bool,
}

impl StartRequest {
    /// A request with symbols off (for simple/test construction).
    pub fn new(mode: Mode, layout: String) -> Self {
        StartRequest {
            mode,
            layout,
            punctuation: false,
            numbers: false,
        }
    }
}

pub struct MenuState {
    pub fields: Vec<Field>,
    pub cursor: usize,
    pub mode_kind: ModeKind,
    pub words: usize,
    pub time: u64,
    pub lesson_level: usize,
    pub layouts: Vec<String>,
    pub layout_idx: usize,
    pub punctuation: bool,
    pub numbers: bool,
}

impl MenuState {
    pub fn new(layouts: Vec<String>, default_layout: &str) -> Self {
        let layout_idx = layouts
            .iter()
            .position(|l| l == default_layout)
            .unwrap_or(0);
        Self {
            fields: vec![
                Field::ModeKind,
                Field::Layout,
                Field::LessonLevel,
                Field::Punctuation,
                Field::Numbers,
                Field::Start,
            ],
            cursor: 0,
            mode_kind: ModeKind::Words,
            words: 50,
            time: 30,
            lesson_level: 1,
            layouts,
            layout_idx,
            punctuation: false,
            numbers: false,
        }
    }

    pub fn focused(&self) -> Field {
        self.fields[self.cursor]
    }

    pub fn move_down(&mut self) {
        self.cursor = (self.cursor + 1) % self.fields.len();
    }

    pub fn move_up(&mut self) {
        self.cursor = (self.cursor + self.fields.len() - 1) % self.fields.len();
    }

    /// Adjust the focused field's value. `delta` is +1 (right) or -1 (left).
    pub fn adjust(&mut self, delta: i32) {
        match self.focused() {
            Field::ModeKind => {
                self.mode_kind = match (self.mode_kind, delta >= 0) {
                    (ModeKind::Words, true) => ModeKind::Timed,
                    (ModeKind::Timed, true) => ModeKind::Lesson,
                    (ModeKind::Lesson, true) => ModeKind::Words,
                    (ModeKind::Words, false) => ModeKind::Lesson,
                    (ModeKind::Timed, false) => ModeKind::Words,
                    (ModeKind::Lesson, false) => ModeKind::Timed,
                };
            }
            Field::Layout => {
                let n = self.layouts.len() as i32;
                self.layout_idx = (((self.layout_idx as i32 + delta) % n + n) % n) as usize;
            }
            Field::LessonLevel => {
                let next = self.lesson_level as i32 + delta;
                self.lesson_level = next.max(1) as usize;
            }
            Field::Punctuation => self.punctuation = !self.punctuation,
            Field::Numbers => self.numbers = !self.numbers,
            Field::Start => {}
        }
    }

    /// Build a StartRequest from the current selection (any field).
    pub fn request(&self) -> StartRequest {
        let mode = match self.mode_kind {
            ModeKind::Words => Mode::Words(self.words),
            ModeKind::Timed => Mode::Timed(self.time),
            ModeKind::Lesson => Mode::Lesson(self.lesson_level),
        };
        StartRequest {
            mode,
            layout: self.layouts[self.layout_idx].clone(),
            punctuation: self.punctuation,
            numbers: self.numbers,
        }
    }

    /// Activate the focused field. Returns a StartRequest only on Start.
    pub fn activate(&self) -> Option<StartRequest> {
        if self.focused() != Field::Start {
            return None;
        }
        Some(self.request())
    }

    /// Preselect the mode kind + value (used when seeding the quick-panel).
    pub fn seed_mode(&mut self, mode: &Mode) {
        match mode {
            Mode::Words(n) => {
                self.mode_kind = ModeKind::Words;
                self.words = *n;
            }
            Mode::Timed(s) => {
                self.mode_kind = ModeKind::Timed;
                self.time = *s;
            }
            Mode::Lesson(l) => {
                self.mode_kind = ModeKind::Lesson;
                self.lesson_level = *l;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> MenuState {
        MenuState::new(
            vec!["qwerty".into(), "colemak-dhm".into(), "graphite".into()],
            "colemak-dhm",
        )
    }

    #[test]
    fn defaults_to_named_layout() {
        let m = menu();
        assert_eq!(m.layouts[m.layout_idx], "colemak-dhm");
    }

    #[test]
    fn cursor_wraps() {
        let mut m = menu();
        m.move_up();
        assert_eq!(m.focused(), Field::Start);
        m.move_down();
        assert_eq!(m.focused(), Field::ModeKind);
    }

    #[test]
    fn adjust_cycles_layout_and_mode() {
        let mut m = menu();
        m.adjust(1); // ModeKind Words -> Timed
        assert_eq!(m.mode_kind, ModeKind::Timed);
        m.cursor = 1; // Layout
        let before = m.layout_idx;
        m.adjust(1);
        assert_ne!(m.layout_idx, before);
    }

    #[test]
    fn request_builds_from_selection_any_field() {
        let mut m = menu();
        m.adjust(1); // Words -> Timed (on ModeKind field)
        let req = m.request(); // not on Start, still works
        assert_eq!(req.mode, Mode::Timed(30));
    }

    #[test]
    fn seed_mode_sets_kind_and_value() {
        let mut m = menu();
        m.seed_mode(&Mode::Lesson(4));
        assert_eq!(m.mode_kind, ModeKind::Lesson);
        assert_eq!(m.lesson_level, 4);
    }

    #[test]
    fn start_emits_request_only_on_start_field() {
        let mut m = menu();
        assert!(m.activate().is_none()); // on ModeKind
        m.cursor = 5; // Start
        let req = m.activate().unwrap();
        assert_eq!(req.mode, Mode::Words(50));
        assert_eq!(req.layout, "colemak-dhm");
    }

    #[test]
    fn toggles_flip_and_flow_into_request() {
        let mut m = menu();
        m.cursor = 3; // Punctuation
        m.adjust(1);
        assert!(m.punctuation);
        m.cursor = 4; // Numbers
        m.adjust(1);
        assert!(m.numbers);
        m.cursor = 5; // Start
        let req = m.activate().unwrap();
        assert!(req.punctuation && req.numbers);
    }
}
