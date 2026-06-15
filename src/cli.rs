//! CLI arguments, resolved over the on-disk `Config` into effective `Settings`.

use crate::config::Config;
use crate::content::quotes::QuoteLength;
use clap::Parser;

/// polytype — a layout-agnostic terminal typing trainer.
#[derive(Debug, Parser, Default)]
#[command(name = "polytype", version, about)]
pub struct Args {
    /// Target layout to train (e.g. colemak-dhm, graphite).
    #[arg(long)]
    pub layout: Option<String>,
    /// Source layout your OS produces (default: qwerty).
    #[arg(long)]
    pub source: Option<String>,
    /// Number of words for a words test.
    #[arg(long)]
    pub words: Option<usize>,
    /// Run a timed test of N seconds instead of a fixed word count.
    #[arg(long)]
    pub time: Option<u64>,
    /// Jump straight into lesson N.
    #[arg(long)]
    pub lesson: Option<usize>,
    /// Hide the on-screen keyboard.
    #[arg(long)]
    pub no_keyboard: bool,
    /// Show the per-key accuracy heatmap.
    #[arg(long)]
    pub heatmap: bool,
    /// Block until you type the correct letter (stop on error).
    #[arg(long)]
    pub strict: bool,
    /// Sprinkle punctuation into words/timed tests.
    #[arg(long)]
    pub punctuation: bool,
    /// Sprinkle numbers into words/timed tests.
    #[arg(long)]
    pub numbers: bool,
    /// Type a random quote/sentence.
    #[arg(long)]
    pub quotes: bool,
    /// Quote length filter: all, short, medium, long.
    #[arg(long)]
    pub quote_length: Option<String>,
    /// Wordlist/language to use (e.g. english, swedish, or a name in your
    /// wordlists dir).
    #[arg(long)]
    pub wordlist: Option<String>,
    /// Render the on-screen keyboard with the halves spaced apart (split board).
    #[arg(long)]
    pub split: bool,
}

/// Which mode to launch directly (None => show the menu).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchMode {
    Menu,
    Words(usize),
    Timed(u64),
    Lesson(usize),
    Quote(QuoteLength),
}

/// Effective settings after merging CLI args over `Config`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub target_layout: String,
    pub source_layout: String,
    pub wordlist: String,
    pub show_keyboard: bool,
    pub show_heatmap: bool,
    pub strict: bool,
    pub punctuation: bool,
    pub numbers: bool,
    pub split_keyboard: bool,
    pub launch: LaunchMode,
}

impl Settings {
    /// Merge CLI args over a base config. CLI flags win when present.
    pub fn resolve(args: &Args, config: &Config) -> Settings {
        let launch = if let Some(n) = args.lesson {
            LaunchMode::Lesson(n)
        } else if let Some(secs) = args.time {
            LaunchMode::Timed(secs)
        } else if let Some(n) = args.words {
            LaunchMode::Words(n)
        } else if args.quotes {
            LaunchMode::Quote(QuoteLength::parse(
                args.quote_length.as_deref().unwrap_or("all"),
            ))
        } else {
            // No mode flag given: show the menu.
            LaunchMode::Menu
        };
        Settings {
            target_layout: args
                .layout
                .clone()
                .unwrap_or_else(|| config.target_layout.clone()),
            source_layout: args
                .source
                .clone()
                .unwrap_or_else(|| config.source_layout.clone()),
            wordlist: args
                .wordlist
                .clone()
                .unwrap_or_else(|| config.wordlist.clone()),
            show_keyboard: if args.no_keyboard {
                false
            } else {
                config.show_keyboard
            },
            show_heatmap: args.heatmap || config.show_heatmap,
            strict: args.strict || config.stop_on_error,
            punctuation: args.punctuation || config.punctuation,
            numbers: args.numbers || config.numbers,
            split_keyboard: args.split || config.split_keyboard,
            launch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_come_from_config_and_show_menu() {
        let s = Settings::resolve(&Args::default(), &Config::default());
        assert_eq!(s.target_layout, "colemak-dhm");
        assert_eq!(s.source_layout, "qwerty");
        assert_eq!(s.launch, LaunchMode::Menu);
        assert!(s.show_keyboard);
        assert!(!s.show_heatmap);
        assert!(!s.strict);
    }

    #[test]
    fn strict_from_flag_or_config() {
        let args = Args {
            strict: true,
            ..Args::default()
        };
        assert!(Settings::resolve(&args, &Config::default()).strict);

        let cfg = Config {
            stop_on_error: true,
            ..Config::default()
        };
        assert!(Settings::resolve(&Args::default(), &cfg).strict);

        assert!(!Settings::resolve(&Args::default(), &Config::default()).strict);
    }

    #[test]
    fn symbols_from_flag_or_config() {
        let args = Args {
            punctuation: true,
            ..Args::default()
        };
        assert!(Settings::resolve(&args, &Config::default()).punctuation);
        let cfg = Config {
            numbers: true,
            ..Config::default()
        };
        assert!(Settings::resolve(&Args::default(), &cfg).numbers);
    }

    #[test]
    fn cli_overrides_config() {
        let args = Args {
            layout: Some("graphite".into()),
            time: Some(30),
            no_keyboard: true,
            heatmap: true,
            ..Args::default()
        };
        let s = Settings::resolve(&args, &Config::default());
        assert_eq!(s.target_layout, "graphite");
        assert_eq!(s.launch, LaunchMode::Timed(30));
        assert!(!s.show_keyboard);
        assert!(s.show_heatmap);
    }

    #[test]
    fn lesson_takes_priority_over_words() {
        let args = Args {
            lesson: Some(3),
            words: Some(50),
            ..Args::default()
        };
        let s = Settings::resolve(&args, &Config::default());
        assert_eq!(s.launch, LaunchMode::Lesson(3));
    }

    #[test]
    fn wordlist_from_flag_or_config() {
        let args = Args {
            wordlist: Some("swedish".into()),
            ..Args::default()
        };
        assert_eq!(
            Settings::resolve(&args, &Config::default()).wordlist,
            "swedish"
        );
        // default comes from config (english)
        assert_eq!(
            Settings::resolve(&Args::default(), &Config::default()).wordlist,
            "english"
        );
    }

    #[test]
    fn quotes_launch_with_length() {
        use crate::content::quotes::QuoteLength;
        let args = Args {
            quotes: true,
            quote_length: Some("medium".into()),
            ..Args::default()
        };
        assert_eq!(
            Settings::resolve(&args, &Config::default()).launch,
            LaunchMode::Quote(QuoteLength::Medium)
        );
    }
}
