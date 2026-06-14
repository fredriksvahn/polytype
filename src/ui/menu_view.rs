//! Renders the menu screen.

use crate::app::menu::{Field, MenuState, ModeKind};
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, menu: &MenuState) {
    let mode = match menu.mode_kind {
        ModeKind::Words => format!("Words ({})", menu.words),
        ModeKind::Timed => format!("Timed ({}s)", menu.time),
        ModeKind::Lesson => "Lesson".to_string(),
    };
    let rows = [
        (Field::ModeKind, format!("Mode:   {mode}")),
        (
            Field::Layout,
            format!("Layout: {}", menu.layouts[menu.layout_idx]),
        ),
        (Field::LessonLevel, format!("Lesson: {}", menu.lesson_level)),
        (Field::Start, "[ Start ]".to_string()),
    ];
    let lines: Vec<Line> = rows
        .iter()
        .map(|(field, text)| {
            let mut line = Line::from(text.clone());
            if *field == menu.focused() {
                line = line.style(Style::new().reversed());
            }
            line
        })
        .collect();
    f.render_widget(
        Paragraph::new(lines).block(Block::default().title("polytype").borders(Borders::ALL)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_menu_rows() {
        let menu = MenuState::new(vec!["colemak-dhm".into()], "colemak-dhm");
        let mut term = Terminal::new(TestBackend::new(40, 10)).unwrap();
        term.draw(|f| render(f, f.area(), &menu)).unwrap();
        let content: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(content.contains("Mode"));
        assert!(content.contains("colemak-dhm"));
        assert!(content.contains("Start"));
    }
}
