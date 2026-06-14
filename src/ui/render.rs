//! Top-level render dispatch by screen.

use crate::app::{App, Screen};
use crate::ui::{menu_view, results, test_view::TestView};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    match app.screen {
        Screen::Menu => menu_view::render(f, area, &app.menu),
        Screen::Test => {
            if let (Some(runner), Some(text), Some(layout)) = (
                app.runner.as_ref(),
                app.target_text.as_ref(),
                app.target_layout(),
            ) {
                TestView {
                    runner,
                    target_text: text,
                    target_layout: layout,
                    stats: &app.stats,
                    show_keyboard: app.settings.show_keyboard,
                    show_heatmap: app.settings.show_heatmap,
                }
                .render(f, area);
            }
        }
        Screen::Results => {
            if let Some(score) = &app.last_score {
                results::render(f, area, score, &app.session_stats);
            }
        }
    }
}
