//! Terminal setup/teardown and the crossterm event loop.

use crate::app::menu;
use crate::app::{App, Screen};
use crate::ui::render;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;
use std::time::Instant;

pub fn run(app: &mut App) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut rng = rand::thread_rng();
    let mut started = Instant::now();

    while !app.should_quit {
        // Keep elapsed fresh for timed mode + live wpm.
        if app.screen == Screen::Test {
            if let Some(runner) = &mut app.runner {
                runner.set_elapsed(started.elapsed().as_secs_f64());
                if runner.is_finished() {
                    app.finish();
                }
            }
        }

        terminal.draw(|f| render::render(f, app))?;

        // Poll so timed tests advance even without keypresses.
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                    continue;
                }
                match app.screen {
                    Screen::Menu => handle_menu_key(app, key, &mut started, &mut rng),
                    Screen::Test => handle_test_key(app, key),
                    Screen::Results => handle_results_key(app, key, &mut started, &mut rng),
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn handle_menu_key(
    app: &mut App,
    key: KeyEvent,
    started: &mut Instant,
    rng: &mut rand::rngs::ThreadRng,
) {
    match key.code {
        KeyCode::Up => app.menu.move_up(),
        KeyCode::Down => app.menu.move_down(),
        KeyCode::Left => app.menu.adjust(-1),
        KeyCode::Right => app.menu.adjust(1),
        KeyCode::Enter => {
            if let Some(req) = app.menu.activate() {
                app.start(req, rng);
                *started = Instant::now();
            }
        }
        KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
}

fn handle_test_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.screen = Screen::Menu,
        KeyCode::Backspace => {
            if let Some(runner) = &mut app.runner {
                runner.backspace();
            }
        }
        KeyCode::Char(c) => {
            if let Some(runner) = &mut app.runner {
                runner.type_char(c);
                if runner.is_finished() {
                    app.finish();
                }
            }
        }
        _ => {}
    }
}

fn handle_results_key(
    app: &mut App,
    key: KeyEvent,
    started: &mut Instant,
    rng: &mut rand::rngs::ThreadRng,
) {
    match key.code {
        KeyCode::Esc => app.screen = Screen::Menu,
        KeyCode::Tab => {
            // Restart the same mode/layout.
            if let (Some(layout), Some(mode)) =
                (app.current_layout.clone(), app.active_mode.clone())
            {
                let req = menu::StartRequest { mode, layout };
                app.start(req, rng);
                *started = Instant::now();
            }
        }
        _ => {}
    }
}
