//! The history: every finished test with all of its keystrokes, in SQLite.
//! Summaries (daily progression, bests, weak keys) are queries over it, and a
//! single test's graph is recomputed from its keystrokes.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

use crate::stats::{self, Keystroke, Kind, Stats};
use crate::test::{Mode, Test};

pub struct Store {
    conn: Connection,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS tests (
    id INTEGER PRIMARY KEY,
    started_at INTEGER NOT NULL,
    mode TEXT NOT NULL,
    mode_value INTEGER NOT NULL,
    language TEXT NOT NULL,
    punctuation INTEGER NOT NULL,
    numbers INTEGER NOT NULL,
    stop_on_error INTEGER NOT NULL,
    confidence INTEGER NOT NULL,
    duration REAL NOT NULL,
    wpm REAL NOT NULL,
    raw REAL NOT NULL,
    accuracy REAL NOT NULL,
    consistency REAL NOT NULL,
    correct INTEGER NOT NULL,
    incorrect INTEGER NOT NULL,
    extra INTEGER NOT NULL,
    missed INTEGER NOT NULL,
    words TEXT NOT NULL,
    typed TEXT NOT NULL,
    quote_source TEXT
);
CREATE INDEX IF NOT EXISTS tests_started ON tests(started_at);
CREATE TABLE IF NOT EXISTS keystrokes (
    test_id INTEGER NOT NULL REFERENCES tests(id) ON DELETE CASCADE,
    seq INTEGER NOT NULL,
    t REAL NOT NULL,
    kind INTEGER NOT NULL,
    ch TEXT NOT NULL,
    expected TEXT NOT NULL,
    correct INTEGER NOT NULL,
    word INTEGER NOT NULL,
    PRIMARY KEY (test_id, seq)
) WITHOUT ROWID;
";

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// A test's record without keystrokes, as lists and tables show it.
const SUMMARY: &str = "id, started_at, mode, mode_value, language, punctuation, numbers, stop_on_error, confidence, \
     duration, wpm, raw, accuracy, consistency, correct, incorrect, extra, missed, quote_source";

fn summary(row: &rusqlite::Row) -> rusqlite::Result<Value> {
    Ok(json!({
        "id": row.get::<_, i64>(0)?,
        "startedAt": row.get::<_, i64>(1)?,
        "mode": row.get::<_, String>(2)?,
        "value": row.get::<_, i64>(3)?,
        "language": row.get::<_, String>(4)?,
        "punctuation": row.get::<_, bool>(5)?,
        "numbers": row.get::<_, bool>(6)?,
        "stopOnError": row.get::<_, bool>(7)?,
        "confidence": row.get::<_, bool>(8)?,
        "duration": row.get::<_, f64>(9)?,
        "wpm": row.get::<_, f64>(10)?,
        "raw": row.get::<_, f64>(11)?,
        "accuracy": row.get::<_, f64>(12)?,
        "consistency": row.get::<_, f64>(13)?,
        "correct": row.get::<_, i64>(14)?,
        "incorrect": row.get::<_, i64>(15)?,
        "extra": row.get::<_, i64>(16)?,
        "missed": row.get::<_, i64>(17)?,
        "quote": row.get::<_, Option<String>>(18)?,
    }))
}

impl Store {
    pub fn open(path: &Path) -> Result<Store, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let conn = Connection::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Store::init(conn)
    }

    pub fn memory() -> Result<Store, String> {
        Store::init(Connection::open_in_memory().map_err(|e| e.to_string())?)
    }

    fn init(conn: Connection) -> Result<Store, String> {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .and_then(|_| conn.execute_batch(SCHEMA))
            .map_err(|e| e.to_string())?;
        Ok(Store { conn })
    }

    /// Best speed so far for the same kind of test, which is what a personal
    /// best is measured against.
    fn best_for(&self, test: &Test) -> rusqlite::Result<Option<f64>> {
        let c = &test.config;
        self.conn
            .query_row(
                "SELECT MAX(wpm) FROM tests WHERE mode = ?1 AND mode_value = ?2 AND language = ?3 \
                 AND punctuation = ?4 AND numbers = ?5",
                params![c.mode.name(), mode_value(test), c.language, c.punctuation, c.numbers],
                |r| r.get::<_, Option<f64>>(0),
            )
    }

    /// Saves a finished test and returns its result as the window shows it.
    pub fn save(&mut self, test: &Test, stats: &Stats) -> Result<Value, String> {
        self.save_at(test, stats, now_ms() - (test.elapsed as i64))
    }

    pub fn save_at(&mut self, test: &Test, stats: &Stats, started_at: i64) -> Result<Value, String> {
        let previous = self.best_for(test).map_err(|e| e.to_string())?;
        let c = &test.config;
        let typed: Vec<String> = test.typed.iter().map(|w| w.iter().collect()).collect();
        let words: Vec<String> = test.targets().iter().map(|w| w.iter().collect()).collect();
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO tests (started_at, mode, mode_value, language, punctuation, numbers, stop_on_error, \
             confidence, duration, wpm, raw, accuracy, consistency, correct, incorrect, extra, missed, words, typed, \
             quote_source) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            params![
                started_at,
                c.mode.name(),
                mode_value(test),
                c.language,
                c.punctuation,
                c.numbers,
                c.stop_on_error,
                c.confidence,
                stats.duration,
                stats.wpm,
                stats.raw,
                stats.accuracy,
                stats.consistency,
                stats.correct,
                stats.incorrect,
                stats.extra,
                stats.missed,
                serde_json::to_string(&words).unwrap_or_default(),
                serde_json::to_string(&typed).unwrap_or_default(),
                test.quote.as_ref().map(|q| q.1.clone()),
            ],
        )
        .map_err(|e| e.to_string())?;
        let id = tx.last_insert_rowid();
        {
            let mut insert = tx
                .prepare("INSERT INTO keystrokes (test_id, seq, t, kind, ch, expected, correct, word) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
                .map_err(|e| e.to_string())?;
            for (seq, k) in test.keys.iter().enumerate() {
                insert
                    .execute(params![id, seq as i64, k.t, k.kind.code(), k.ch, k.expected, k.correct, k.word as i64])
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        let mut result = self.get(id)?.unwrap_or(Value::Null);
        result["pb"] = json!(previous.is_some_and(|p| stats.wpm > p));
        result["previousBest"] = json!(previous);
        Ok(result)
    }

    /// One test with its words, keystrokes, and per-second graph.
    pub fn get(&self, id: i64) -> Result<Option<Value>, String> {
        let row = self
            .conn
            .query_row(
                &format!("SELECT {SUMMARY}, words, typed FROM tests WHERE id = ?1"),
                params![id],
                |r| Ok((summary(r)?, r.get::<_, String>(19)?, r.get::<_, String>(20)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((mut value, words, typed)) = row else { return Ok(None) };
        let words: Vec<String> = serde_json::from_str(&words).unwrap_or_default();
        let typed: Vec<String> = serde_json::from_str(&typed).unwrap_or_default();
        let keys = self.keystrokes(id)?;
        let targets: Vec<Vec<char>> = words.iter().map(|w| w.chars().collect()).collect();
        let typed_chars: Vec<Vec<char>> = typed.iter().map(|w| w.chars().collect()).collect();
        let duration_ms = value["duration"].as_f64().unwrap_or(0.0) * 1000.0;
        let s = stats::compute(&targets, &typed_chars, &keys, duration_ms);
        value["seconds"] = json!(s.seconds);
        value["words"] = json!(words);
        value["typed"] = json!(typed);
        value["keystrokes"] = json!(keys.len());
        Ok(Some(value))
    }

    pub fn keystrokes(&self, id: i64) -> Result<Vec<Keystroke>, String> {
        let mut q = self
            .conn
            .prepare("SELECT t, kind, ch, expected, correct, word FROM keystrokes WHERE test_id = ?1 ORDER BY seq")
            .map_err(|e| e.to_string())?;
        let rows = q
            .query_map(params![id], |r| {
                Ok(Keystroke {
                    t: r.get(0)?,
                    kind: Kind::from_code(r.get(1)?),
                    ch: r.get(2)?,
                    expected: r.get(3)?,
                    correct: r.get(4)?,
                    word: r.get::<_, i64>(5)? as usize,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn recent(&self, limit: i64, offset: i64) -> Result<Value, String> {
        self.rows(
            &format!("SELECT {SUMMARY} FROM tests ORDER BY started_at DESC LIMIT ?1 OFFSET ?2"),
            params![limit, offset],
            summary,
        )
    }

    pub fn delete(&mut self, id: i64) -> Result<bool, String> {
        self.conn.execute("DELETE FROM tests WHERE id = ?1", params![id]).map(|n| n > 0).map_err(|e| e.to_string())
    }

    /// Per local day over the last `days` days: tests, time typed, average and best speed.
    pub fn daily(&self, days: i64) -> Result<Value, String> {
        let since = now_ms() - days * 86_400_000;
        self.rows(
            "SELECT date(started_at / 1000, 'unixepoch', 'localtime') AS day, COUNT(*), SUM(duration), \
             AVG(wpm), MAX(wpm), AVG(accuracy), AVG(raw) FROM tests WHERE started_at >= ?1 GROUP BY day ORDER BY day",
            params![since],
            |r| {
                Ok(json!({
                    "day": r.get::<_, String>(0)?,
                    "tests": r.get::<_, i64>(1)?,
                    "seconds": r.get::<_, f64>(2)?,
                    "wpm": round(r.get::<_, f64>(3)?),
                    "best": r.get::<_, f64>(4)?,
                    "accuracy": round(r.get::<_, f64>(5)?),
                    "raw": round(r.get::<_, f64>(6)?),
                }))
            },
        )
    }

    /// The best run for each kind of test, with how often it was played.
    pub fn bests(&self) -> Result<Value, String> {
        self.rows(
            &format!(
                "SELECT {SUMMARY}, n FROM (SELECT *, COUNT(*) OVER g AS n, \
                 ROW_NUMBER() OVER (PARTITION BY mode, mode_value, language, punctuation, numbers ORDER BY wpm DESC, started_at) AS rk \
                 FROM tests WINDOW g AS (PARTITION BY mode, mode_value, language, punctuation, numbers)) \
                 WHERE rk = 1 ORDER BY CASE mode WHEN 'time' THEN 0 WHEN 'words' THEN 1 WHEN 'quote' THEN 2 ELSE 3 END, \
                 mode_value, language, punctuation, numbers"
            ),
            [],
            |r| {
                let mut v = summary(r)?;
                v["runs"] = json!(r.get::<_, i64>(19)?);
                Ok(v)
            },
        )
    }

    pub fn totals(&self) -> Result<Value, String> {
        let overall = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(duration), 0), COALESCE(AVG(wpm), 0), COALESCE(MAX(wpm), 0), \
                 COALESCE(AVG(accuracy), 0), COALESCE(AVG(consistency), 0), \
                 (SELECT COUNT(*) FROM keystrokes WHERE kind IN (0, 1)) FROM tests",
                [],
                |r| {
                    Ok(json!({
                        "tests": r.get::<_, i64>(0)?,
                        "seconds": r.get::<_, f64>(1)?,
                        "wpm": round(r.get::<_, f64>(2)?),
                        "best": r.get::<_, f64>(3)?,
                        "accuracy": round(r.get::<_, f64>(4)?),
                        "consistency": round(r.get::<_, f64>(5)?),
                        "keystrokes": r.get::<_, i64>(6)?,
                    }))
                },
            )
            .map_err(|e| e.to_string())?;
        let last10 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(wpm), 0), COALESCE(AVG(accuracy), 0) FROM (SELECT wpm, accuracy FROM tests ORDER BY started_at DESC LIMIT 10)",
                [],
                |r| Ok((r.get::<_, f64>(0)?, r.get::<_, f64>(1)?)),
            )
            .map_err(|e| e.to_string())?;
        let mut v = overall;
        v["last10Wpm"] = json!(round(last10.0));
        v["last10Accuracy"] = json!(round(last10.1));
        Ok(v)
    }

    /// Keys missed most often, as a share of the times the text asked for them.
    pub fn weak_keys(&self, limit: i64, min_seen: i64) -> Result<Value, String> {
        self.rows(
            "SELECT expected, COUNT(*) AS seen, SUM(correct = 0) AS missed FROM keystrokes \
             WHERE kind IN (0, 1) AND expected != '' GROUP BY expected HAVING seen >= ?2 \
             ORDER BY CAST(missed AS REAL) / seen DESC, seen DESC LIMIT ?1",
            params![limit, min_seen],
            |r| {
                let seen = r.get::<_, i64>(1)?;
                let missed = r.get::<_, i64>(2)?;
                Ok(json!({
                    "key": r.get::<_, String>(0)?,
                    "seen": seen,
                    "missed": missed,
                    "rate": round(100.0 * missed as f64 / seen.max(1) as f64),
                }))
            },
        )
    }

    fn rows<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
        map: impl FnMut(&rusqlite::Row) -> rusqlite::Result<Value>,
    ) -> Result<Value, String> {
        let mut q = self.conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = q.query_map(params, map).map_err(|e| e.to_string())?;
        Ok(Value::Array(rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?))
    }
}

fn mode_value(test: &Test) -> i64 {
    if test.config.mode == Mode::Zen { 0 } else { test.config.value as i64 }
}

fn round(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::Config;

    fn finished(seed: u64) -> Test {
        let mut t = Test::new(
            Config {
                mode: Mode::Words,
                value: 5,
                punctuation: false,
                numbers: false,
                stop_on_error: false,
                confidence: false,
                language: "en".into(),
                seed: Some(seed),
            },
            None,
        );
        let text = t.words.join(" ");
        let mut now = 0.0;
        for c in text.chars() {
            t.input(&c.to_string(), now);
            now += 80.0 + seed as f64;
        }
        assert!(t.finished);
        t
    }

    #[test]
    fn saves_and_reloads_with_the_same_graph() {
        let mut store = Store::memory().unwrap();
        let t = finished(1);
        let s = t.stats(0.0);
        let saved = store.save(&t, &s).unwrap();
        let id = saved["id"].as_i64().unwrap();
        let loaded = store.get(id).unwrap().unwrap();
        assert_eq!(loaded["wpm"], json!(s.wpm));
        assert_eq!(loaded["seconds"].as_array().unwrap().len(), s.seconds.len());
        assert_eq!(store.keystrokes(id).unwrap().len(), t.keys.len());
    }

    #[test]
    fn flags_a_personal_best_against_the_same_kind_of_test() {
        let mut store = Store::memory().unwrap();
        let slow = finished(40);
        assert_eq!(store.save(&slow, &slow.stats(0.0)).unwrap()["pb"], json!(false));
        let fast = finished(1);
        assert_eq!(store.save(&fast, &fast.stats(0.0)).unwrap()["pb"], json!(true));
        let bests = store.bests().unwrap();
        assert_eq!(bests.as_array().unwrap().len(), 1);
        assert_eq!(bests[0]["runs"], json!(2));
    }

    #[test]
    fn summaries_cover_saved_tests() {
        let mut store = Store::memory().unwrap();
        for seed in 1..4 {
            let t = finished(seed);
            store.save(&t, &t.stats(0.0)).unwrap();
        }
        assert_eq!(store.totals().unwrap()["tests"], json!(3));
        assert_eq!(store.daily(30).unwrap()[0]["tests"], json!(3));
        assert_eq!(store.recent(10, 0).unwrap().as_array().unwrap().len(), 3);
        assert!(!store.weak_keys(5, 1).unwrap().as_array().unwrap().is_empty());
        let id = store.recent(1, 0).unwrap()[0]["id"].as_i64().unwrap();
        assert!(store.delete(id).unwrap());
        assert_eq!(store.totals().unwrap()["tests"], json!(2));
    }
}
