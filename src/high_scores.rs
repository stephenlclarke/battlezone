use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MAX_HIGH_SCORES: usize = 10;
const DEFAULT_HIGH_SCORES: [(&str, u32); MAX_HIGH_SCORES] = [
    ("ACE", 120_000),
    ("DVG", 105_000),
    ("AVG", 70_000),
    ("ROM", 60_000),
    ("CPU", 50_000),
    ("TNK", 40_000),
    ("RAD", 30_000),
    ("GUN", 20_000),
    ("VEC", 10_000),
    ("COM", 5_000),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HighScoreEntry {
    pub initials: String,
    pub score: u32,
}

pub fn default_high_scores() -> Vec<HighScoreEntry> {
    DEFAULT_HIGH_SCORES
        .into_iter()
        .map(|(initials, score)| HighScoreEntry {
            initials: initials.to_string(),
            score,
        })
        .collect()
}

pub fn load_default() -> Vec<HighScoreEntry> {
    load(&default_storage_path()).unwrap_or_else(|_| default_high_scores())
}

pub fn load(path: &Path) -> io::Result<Vec<HighScoreEntry>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(parse(&text).unwrap_or_else(default_high_scores)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(default_high_scores()),
        Err(error) => Err(error),
    }
}

pub fn save_default(entries: &[HighScoreEntry]) -> io::Result<()> {
    save(&default_storage_path(), entries)
}

pub fn save(path: &Path, entries: &[HighScoreEntry]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serialize(entries))
}

pub fn top_score(entries: &[HighScoreEntry]) -> u32 {
    normalize_entries(entries)
        .first()
        .map_or(0, |entry| entry.score)
}

pub fn qualifies(entries: &[HighScoreEntry], score: u32) -> bool {
    score > 0
        && (entries.len() < MAX_HIGH_SCORES
            || normalize_entries(entries)
                .last()
                .is_some_and(|entry| score > entry.score))
}

pub fn insert_entry(entries: &mut Vec<HighScoreEntry>, initials: &str, score: u32) {
    entries.push(HighScoreEntry {
        initials: sanitize_initials(initials),
        score,
    });
    *entries = normalize_entries(entries);
}

pub fn default_storage_path() -> PathBuf {
    if let Some(path) = env::var_os("BATTLEZONE_DATA_DIR") {
        return PathBuf::from(path).join("high_scores.txt");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".battlezone")
            .join("high_scores.txt");
    }

    PathBuf::from(".battlezone-high-scores.txt")
}

pub fn sanitize_initials(initials: &str) -> String {
    let mut cleaned: String = initials
        .chars()
        .filter_map(|character| match character {
            'a'..='z' | 'A'..='Z' => Some(character.to_ascii_uppercase()),
            ' ' | '-' => Some(character),
            _ => None,
        })
        .take(3)
        .collect();

    while cleaned.len() < 3 {
        cleaned.push(' ');
    }

    cleaned
}

fn normalize_entries(entries: &[HighScoreEntry]) -> Vec<HighScoreEntry> {
    let mut normalized: Vec<HighScoreEntry> = entries
        .iter()
        .map(|entry| HighScoreEntry {
            initials: sanitize_initials(&entry.initials),
            score: entry.score,
        })
        .collect();
    normalized.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.initials.cmp(&right.initials))
    });
    normalized.truncate(MAX_HIGH_SCORES);
    normalized
}

fn parse(text: &str) -> Option<Vec<HighScoreEntry>> {
    let mut entries = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let (initials, score) = trimmed.split_once('|')?;
        entries.push(HighScoreEntry {
            initials: sanitize_initials(initials),
            score: score.parse().ok()?,
        });
    }

    Some(normalize_entries(&entries))
}

fn serialize(entries: &[HighScoreEntry]) -> String {
    let mut text = String::new();
    for entry in normalize_entries(entries) {
        text.push_str(&entry.initials);
        text.push('|');
        text.push_str(&entry.score.to_string());
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{
        HighScoreEntry, default_high_scores, default_storage_path, insert_entry, load, qualifies,
        sanitize_initials, save, top_score,
    };

    static NEXT_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "battlezone-high-scores-test-{}-{}",
                std::process::id(),
                NEXT_DIR_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn default_table_exposes_seeded_top_score() {
        let entries = default_high_scores();

        assert_eq!(entries.len(), 10);
        assert_eq!(top_score(&entries), 120_000);
    }

    #[test]
    fn insert_entry_sorts_descending_and_truncates() {
        let mut entries = default_high_scores();

        insert_entry(&mut entries, "zap", 130_000);

        assert_eq!(entries[0].initials, "ZAP");
        assert_eq!(entries[0].score, 130_000);
        assert_eq!(entries.len(), 10);
        assert_eq!(
            entries.last().map(|entry| entry.initials.as_str()),
            Some("VEC")
        );
    }

    #[test]
    fn qualify_requires_beating_the_last_seeded_score() {
        let entries = default_high_scores();

        assert!(qualifies(&entries, 130_000));
        assert!(!qualifies(&entries, 5_000));
    }

    #[test]
    fn save_and_load_round_trip_entries() {
        let dir = TempDir::new();
        let path = dir.path().join("scores.txt");
        let entries = vec![
            HighScoreEntry {
                initials: "A C".to_string(),
                score: 12_000,
            },
            HighScoreEntry {
                initials: "BBB".to_string(),
                score: 15_000,
            },
        ];

        save(&path, &entries).expect("save scores");
        let loaded = load(&path).expect("load scores");

        assert_eq!(loaded[0].initials, "BBB");
        assert_eq!(loaded[0].score, 15_000);
        assert_eq!(loaded[1].initials, "A C");
    }

    #[test]
    fn load_defaults_when_file_is_missing() {
        let dir = TempDir::new();
        let path = dir.path().join("missing.txt");

        let loaded = load(&path).expect("missing file should default");

        assert_eq!(loaded, default_high_scores());
    }

    #[test]
    fn sanitize_initials_preserves_spaces_and_dashes() {
        assert_eq!(sanitize_initials("a- "), "A- ");
        assert_eq!(sanitize_initials("ab"), "AB ");
    }

    #[test]
    fn default_storage_path_uses_override_or_home() {
        let original_override = std::env::var_os("BATTLEZONE_DATA_DIR");
        let original_home = std::env::var_os("HOME");

        let override_path = std::env::temp_dir().join("battlezone-score-override");
        // SAFETY: This test sets process environment variables and restores them before exit.
        unsafe {
            std::env::set_var("BATTLEZONE_DATA_DIR", &override_path);
        }
        let path = default_storage_path();
        assert_eq!(path, override_path.join("high_scores.txt"));

        // SAFETY: This test sets process environment variables and restores them before exit.
        unsafe {
            std::env::remove_var("BATTLEZONE_DATA_DIR");
            std::env::set_var("HOME", "/tmp/battlezone-home");
        }
        let path = default_storage_path();
        assert_eq!(
            path,
            PathBuf::from("/tmp/battlezone-home/.battlezone/high_scores.txt")
        );

        // SAFETY: This test restores process environment variables modified above.
        unsafe {
            if let Some(value) = original_override {
                std::env::set_var("BATTLEZONE_DATA_DIR", value);
            } else {
                std::env::remove_var("BATTLEZONE_DATA_DIR");
            }
            if let Some(value) = original_home {
                std::env::set_var("HOME", value);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }
}
