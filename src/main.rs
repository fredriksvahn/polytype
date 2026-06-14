use clap::Parser;
use polytype::app::App;
use polytype::cli::{Args, LaunchMode, Settings};
use polytype::config::Config;
use polytype::content::wordlist;
use polytype::layout::builtin::load_registry;
use polytype::stats::KeyStats;

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let config = Config::load().unwrap_or_default();
    let settings = Settings::resolve(&args, &config);

    let user_layouts = Config::config_dir().map(|d| d.join("layouts"));
    let registry = load_registry(user_layouts.as_deref()).unwrap_or_default();

    let stats_path = KeyStats::data_dir().map(|d| d.join("keystats.toml"));
    let stats = stats_path
        .as_ref()
        .and_then(|p| KeyStats::load_from(p).ok())
        .unwrap_or_default();

    // Word pool: piped stdin wins, else bundled english.
    let pool = {
        use std::io::IsTerminal;
        if std::io::stdin().is_terminal() {
            wordlist::english()
        } else {
            polytype::content::from_stdin().unwrap_or_else(|_| wordlist::english())
        }
    };

    let mut app = App::new(settings.clone(), registry, stats, pool);

    // CLI launch shortcuts skip the menu.
    let mut rng = rand::thread_rng();
    match settings.launch {
        LaunchMode::Menu => {}
        LaunchMode::Words(n) => {
            app.start(menu_req(&settings, polytype::app::Mode::Words(n)), &mut rng)
        }
        LaunchMode::Timed(s) => {
            app.start(menu_req(&settings, polytype::app::Mode::Timed(s)), &mut rng)
        }
        LaunchMode::Lesson(n) => app.start(
            menu_req(&settings, polytype::app::Mode::Lesson(n)),
            &mut rng,
        ),
    }

    polytype::term::run(&mut app)?;

    // Persist stats on exit.
    if let Some(p) = stats_path {
        let _ = app.stats.save_to(&p);
    }
    Ok(())
}

fn menu_req(settings: &Settings, mode: polytype::app::Mode) -> polytype::app::menu::StartRequest {
    polytype::app::menu::StartRequest {
        mode,
        layout: settings.target_layout.clone(),
    }
}
