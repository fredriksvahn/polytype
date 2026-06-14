//! Color palette for the UI.

use crate::ui::heat::Heat;
use crate::ui::keyboard::Hand;
use ratatui::style::Color;

pub const CURSOR_BG: Color = Color::Yellow;
pub const CURSOR_FG: Color = Color::Black;
pub const TODO: Color = Color::Gray;
pub const CORRECT: Color = Color::White;
pub const WRONG: Color = Color::Red;
pub const STATUS: Color = Color::Cyan;

pub fn hand_color(hand: Hand) -> Color {
    match hand {
        Hand::Left => Color::Green,
        Hand::Right => Color::Magenta,
    }
}

pub fn heat_color(heat: Heat) -> Color {
    match heat {
        Heat::Unknown => Color::DarkGray,
        Heat::Bad => Color::Red,
        Heat::Mid => Color::Yellow,
        Heat::Good => Color::Green,
    }
}
