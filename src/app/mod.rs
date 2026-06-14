//! Application state shell shared by the UI.

pub mod menu;
pub mod runner;

/// Which test mode is being run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Words(usize),
    Timed(u64),
    Lesson(usize),
}

/// Which screen is currently shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Test,
    Results,
}
