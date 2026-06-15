//! Top-level render dispatch by screen.

use crate::app::{App, Screen};
use crate::ui::{menu_view, results, test_view::TestView, theme};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Clear};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    // Solid background so the terminal's own backdrop doesn't show through.
    f.render_widget(Block::default().style(Style::new().bg(theme::BG)), area);
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
                    split_keyboard: app.settings.split_keyboard,
                }
                .render(f, area);
            }
        }
        Screen::Results => {
            if let Some(score) = &app.last_score {
                results::render(f, area, score, &app.session_stats);
            }
        }
        Screen::Stats => {
            let layout = app
                .target_layout()
                .or_else(|| app.registry.get(&app.settings.target_layout));
            if let Some(layout) = layout {
                crate::ui::stats_view::render(f, area, layout, &app.stats);
            }
        }
    }

    // Quick-panel overlay on top of the test.
    if let Some(overlay) = &app.overlay {
        let panel = centered_rect(60, 60, area);
        f.render_widget(Clear, panel);
        f.render_widget(Block::default().style(Style::new().bg(theme::BG)), panel);
        menu_view::render(f, panel, overlay);
    }
}

/// A Rect centered in `area`, sized to the given percentage of width/height.
fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::Keymap;
    use crate::layout::builtin::load_registry;
    use crate::stats::KeyStats;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn overlay_draws_menu_over_test() {
        let registry = load_registry(None).unwrap();
        let settings = crate::cli::Settings::resolve(
            &crate::cli::Args::default(),
            &crate::config::Config::default(),
        );
        let mut app = App::new(
            settings,
            registry,
            KeyStats::default(),
            vec!["the".into(), "fox".into()],
            Keymap::defaults(),
        );
        let mut rng = rand::thread_rng();
        app.start(
            crate::app::menu::StartRequest::new(crate::app::Mode::Words(3), "qwerty".into()),
            &mut rng,
        );
        app.open_panel();

        let mut term = Terminal::new(TestBackend::new(60, 20)).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let content: String = term
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(content.contains("Mode"), "panel shows menu fields");
        assert!(content.contains("Start"), "panel shows Start");
    }
}
