use polytype::config::Config;
use polytype::content::{generate_words, wordlist};
use polytype::engine::TestSession;
use polytype::layout::{builtin::load_registry, remap::Remapper};
use polytype::stats::KeyStats;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[test]
fn full_words_test_perfect_run() {
    let cfg = Config::default();
    let reg = load_registry(None).unwrap();
    let target = reg[&cfg.target_layout].clone(); // colemak-dhm
    let source = reg[&cfg.source_layout].clone(); // qwerty
    let remap = Remapper::new(source.clone(), target.clone());

    // Generate target text (normal English words).
    let pool = wordlist::english();
    let mut rng = StdRng::seed_from_u64(123);
    let text = generate_words(&pool, 10, &mut rng);

    // Simulate a perfect typist: for each expected target char, find which
    // source-layout key produces it, "press" that key, and remap it back.
    let mut session = TestSession::new(&text);
    for expected in text.chars() {
        if expected == ' ' {
            session.input(remap.remap(' '));
            continue;
        }
        // Which physical position produces `expected` on the target layout?
        let pos = target.position_of(expected).expect("char on target grid");
        // The key the user physically presses produces this char from the source layout.
        let pressed = source.char_at(pos).unwrap();
        session.input(remap.remap(pressed));
    }

    assert!(session.is_finished());
    let score = session.score(30.0);
    assert_eq!(score.accuracy, 1.0);

    // Persist stats.
    let mut stats = KeyStats::default();
    stats.merge(session.per_key());
    assert!(stats.keys.values().all(|(_, m)| *m == 0));
}
