//! Terminal UI: view models and ratatui rendering.

use ratatui::layout::Rect;

/// A rect of the given size, centered horizontally + vertically in `area`,
/// clamped to `area`'s bounds. Used to lay screens out in a calm centered column.
pub fn centered_column(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

pub mod heat;
pub mod keyboard;
pub mod menu_view;
pub mod render;
pub mod results;
pub mod test_view;
pub mod theme;
