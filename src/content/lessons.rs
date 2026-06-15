//! Lessons: a generated key progression plus user-defined lessons (keys or text).

use crate::layout::Layout;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LessonContent {
    /// Allowed letters; drill words are generated from them.
    Keys(Vec<char>),
    /// A fixed passage to type verbatim.
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lesson {
    pub name: String,
    pub content: LessonContent,
}

impl Lesson {
    pub fn allowed_keys(&self) -> Option<&[char]> {
        match &self.content {
            LessonContent::Keys(k) => Some(k),
            LessonContent::Text(_) => None,
        }
    }
}

/// Order in which physical positions are introduced (home row first, then top, bottom).
const INTRO_ORDER: &[usize] = &[
    13, 16, 12, 17, 11, 18, 10, 19, 14, 15, // home
    3, 6, 2, 7, 1, 8, 0, 9, 4, 5, // top
    23, 26, 22, 27, 21, 28, 20, 29, 24, 25, // bottom
];

/// Generated progression for a layout: each lesson adds keys (5 first, then 2).
pub fn progression(target: &Layout) -> Vec<Lesson> {
    let mut lessons = Vec::new();
    let mut acc: Vec<char> = Vec::new();
    let mut level = 0usize;
    let mut i = 0usize;
    while i < INTRO_ORDER.len() {
        let take = if level == 0 { 5 } else { 2 };
        for _ in 0..take {
            if let Some(&pos) = INTRO_ORDER.get(i) {
                if let Some(c) = target.char_at(pos) {
                    if c.is_alphabetic() {
                        acc.push(c);
                    }
                }
                i += 1;
            }
        }
        level += 1;
        lessons.push(Lesson {
            name: format!("Lesson {level}"),
            content: LessonContent::Keys(acc.clone()),
        });
    }
    lessons
}

/// Text for a lesson: literal for Text, generated drill for Keys.
pub fn lesson_text<R: Rng>(
    lesson: &Lesson,
    pool: &[String],
    word_count: usize,
    rng: &mut R,
) -> String {
    let keys = match &lesson.content {
        LessonContent::Text(t) => return t.clone(),
        LessonContent::Keys(k) => k,
    };
    let allowed: std::collections::HashSet<char> = keys.iter().copied().collect();
    let usable: Vec<&String> = pool
        .iter()
        .filter(|w| !w.is_empty() && w.chars().all(|c| allowed.contains(&c)))
        .collect();
    if usable.len() >= 5 {
        (0..word_count)
            .map(|_| usable.choose(rng).unwrap().as_str())
            .collect::<Vec<_>>()
            .join(" ")
    } else if keys.is_empty() {
        String::new()
    } else {
        (0..word_count)
            .map(|_| {
                let len = rng.gen_range(3..=5);
                (0..len)
                    .map(|_| *keys.choose(rng).unwrap())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Deserialize)]
struct LessonFile {
    name: String,
    #[serde(default)]
    keys: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

/// Load user lessons from `dir/*.toml` (sorted). `text` wins over `keys`; a file
/// with neither is skipped.
pub fn user_lessons(dir: Option<&Path>) -> Vec<Lesson> {
    let mut out = Vec::new();
    let Some(dir) = dir else { return out };
    if !dir.is_dir() {
        return out;
    }
    let mut paths: Vec<std::path::PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("toml"))
            .collect(),
        Err(_) => return out,
    };
    paths.sort();
    for path in paths {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(lf) = toml::from_str::<LessonFile>(&text) else {
            continue;
        };
        let content = if let Some(t) = lf.text.filter(|s| !s.trim().is_empty()) {
            LessonContent::Text(t)
        } else if let Some(k) = lf.keys.filter(|s| !s.trim().is_empty()) {
            LessonContent::Keys(k.chars().filter(|c| !c.is_whitespace()).collect())
        } else {
            continue;
        };
        out.push(Lesson {
            name: lf.name,
            content,
        });
    }
    out
}

/// Generated progression for `layout` followed by the user lessons.
pub fn all_lessons(layout: &Layout, user: &[Lesson]) -> Vec<Lesson> {
    let mut v = progression(layout);
    v.extend(user.iter().cloned());
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::builtin::builtin_registry;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn dhm() -> Layout {
        builtin_registry().unwrap()["colemak-dhm"].clone()
    }

    #[test]
    fn first_lesson_has_five_keys() {
        let p = progression(&dhm());
        assert_eq!(p[0].allowed_keys().unwrap().len(), 5);
    }

    #[test]
    fn lessons_are_cumulative() {
        let p = progression(&dhm());
        for w in p.windows(2) {
            let a = w[0].allowed_keys().unwrap();
            let b = w[1].allowed_keys().unwrap();
            assert!(b.len() >= a.len());
            assert!(a.iter().all(|c| b.contains(c)));
        }
    }

    #[test]
    fn keys_lesson_only_uses_allowed() {
        let p = progression(&dhm());
        let mut rng = StdRng::seed_from_u64(1);
        let text = lesson_text(&p[0], &[], 10, &mut rng); // empty pool → fallback
        let allowed: std::collections::HashSet<char> =
            p[0].allowed_keys().unwrap().iter().copied().collect();
        for c in text.chars().filter(|c| *c != ' ') {
            assert!(allowed.contains(&c), "char {c} not allowed");
        }
    }

    #[test]
    fn text_lesson_is_verbatim() {
        let l = Lesson {
            name: "p".into(),
            content: LessonContent::Text("hello world".into()),
        };
        let mut rng = StdRng::seed_from_u64(1);
        assert_eq!(lesson_text(&l, &[], 5, &mut rng), "hello world");
    }

    #[test]
    fn user_lessons_parse_keys_text_and_skip() {
        let dir = std::env::temp_dir().join("polytype-test-lessons");
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.toml"), "name = \"keys one\"\nkeys = \"abc\"\n").unwrap();
        std::fs::write(
            dir.join("b.toml"),
            "name = \"text one\"\ntext = \"hi there\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("c.toml"),
            "name = \"both\"\nkeys = \"xyz\"\ntext = \"wins\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("d.toml"), "name = \"empty\"\n").unwrap();
        let ls = user_lessons(Some(&dir));
        assert_eq!(ls.len(), 3); // d skipped
        assert_eq!(ls[0].content, LessonContent::Keys(vec!['a', 'b', 'c']));
        assert_eq!(ls[1].content, LessonContent::Text("hi there".into()));
        assert_eq!(ls[2].content, LessonContent::Text("wins".into())); // text wins
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn all_lessons_appends_user() {
        let user = vec![Lesson {
            name: "mine".into(),
            content: LessonContent::Text("t".into()),
        }];
        let all = all_lessons(&dhm(), &user);
        assert_eq!(all.last().unwrap().name, "mine");
        assert!(all.len() == progression(&dhm()).len() + 1);
    }
}
