//! Terminal setup/teardown and the crossterm event loop.

use crate::app::menu::{Field, StartRequest};
use crate::app::{App, Screen};
use crate::keys::Action;
use crate::ui::render;
use ratatui::crossterm::event::{self, Event, KeyEvent};
use std::io;
use std::time::{Duration, Instant};

pub fn run(app: &mut App) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut rng = rand::thread_rng();
    let mut started = Instant::now();

    while !app.should_quit {
        // Advance the clock only while actually typing (not while the panel is open).
        if app.screen == Screen::Test && app.overlay.is_none() {
            if let Some(runner) = &mut app.runner {
                runner.set_elapsed(started.elapsed().as_secs_f64());
                if runner.is_finished() {
                    app.finish();
                }
            }
        }

        terminal.draw(|f| render::render(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.keymap.matches(Action::Quit, &key) {
                    app.should_quit = true;
                    continue;
                }
                if app.overlay.is_some() {
                    handle_overlay_key(app, key, &mut started, &mut rng);
                } else {
                    match app.screen {
                        Screen::Menu => handle_menu_key(app, key, &mut started, &mut rng),
                        Screen::Test => handle_test_key(app, key, &mut started, &mut rng),
                        Screen::Results => handle_results_key(app, key, &mut started, &mut rng),
                    }
                }
            }
        }

        if app.pending_edit_config {
            app.pending_edit_config = false;
            ratatui::restore();
            crate::editor::edit_config();
            terminal = ratatui::init();
            let _ = terminal.clear();
            app.reload_config();
            started = Instant::now();
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
    if app.keymap.matches(Action::NavUp, &key) {
        app.menu.move_up();
    } else if app.keymap.matches(Action::NavDown, &key) {
        app.menu.move_down();
    } else if app.keymap.matches(Action::NavPrev, &key) {
        app.menu.adjust(-1);
    } else if app.keymap.matches(Action::NavNext, &key) {
        app.menu.adjust(1);
    } else if app.keymap.matches(Action::Confirm, &key) {
        if app.menu.focused() == Field::EditConfig {
            app.pending_edit_config = true;
        } else if let Some(req) = app.menu.activate() {
            app.start(req, rng);
            *started = Instant::now();
        }
    }
}

fn handle_test_key(
    app: &mut App,
    key: KeyEvent,
    started: &mut Instant,
    rng: &mut rand::rngs::ThreadRng,
) {
    if app.keymap.matches(Action::TestRestart, &key) {
        if let (Some(layout), Some(mode)) = (app.current_layout.clone(), app.active_mode.clone()) {
            let punctuation = app.active_punctuation;
            let numbers = app.active_numbers;
            app.start(
                StartRequest {
                    mode,
                    layout,
                    punctuation,
                    numbers,
                },
                rng,
            );
            *started = Instant::now();
        }
        return;
    }
    if app.keymap.matches(Action::TestPanel, &key) {
        app.open_panel(); // clock freezes (overlay open)
        return;
    }
    use ratatui::crossterm::event::KeyCode;
    match key.code {
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
    if app.keymap.matches(Action::ResultsRestart, &key) {
        if let (Some(layout), Some(mode)) = (app.current_layout.clone(), app.active_mode.clone()) {
            let punctuation = app.active_punctuation;
            let numbers = app.active_numbers;
            app.start(
                StartRequest {
                    mode,
                    layout,
                    punctuation,
                    numbers,
                },
                rng,
            );
            *started = Instant::now();
        }
    } else if app.keymap.matches(Action::ResultsMenu, &key) {
        app.screen = Screen::Menu;
    }
}

fn handle_overlay_key(
    app: &mut App,
    key: KeyEvent,
    started: &mut Instant,
    rng: &mut rand::rngs::ThreadRng,
) {
    if app.keymap.matches(Action::NavUp, &key) {
        if let Some(m) = &mut app.overlay {
            m.move_up();
        }
    } else if app.keymap.matches(Action::NavDown, &key) {
        if let Some(m) = &mut app.overlay {
            m.move_down();
        }
    } else if app.keymap.matches(Action::NavPrev, &key) {
        if let Some(m) = &mut app.overlay {
            m.adjust(-1);
        }
    } else if app.keymap.matches(Action::NavNext, &key) {
        if let Some(m) = &mut app.overlay {
            m.adjust(1);
        }
    } else if app.keymap.matches(Action::Confirm, &key) {
        if app.overlay.as_ref().map(|m| m.focused()) == Some(Field::EditConfig) {
            app.pending_edit_config = true;
            app.cancel_panel();
        } else {
            app.confirm_panel(rng);
            *started = Instant::now();
        }
    } else if app.keymap.matches(Action::PanelCancel, &key) {
        // Resume: shift the clock so elapsed continues from where it froze.
        let elapsed = app.runner.as_ref().map(|r| r.elapsed()).unwrap_or(0.0);
        *started = Instant::now() - Duration::from_secs_f64(elapsed);
        app.cancel_panel();
    }
}
