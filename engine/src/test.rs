//! One typing test: the text, what has been typed into each word, and every
//! keystroke with its time. Rules follow the common typing-test conventions:
//! space submits a word, backspace can only step back into a word that was
//! left wrong, and the test starts on the first keystroke.

use serde::{Deserialize, Serialize};

use crate::stats::{self, Keystroke, Kind, Stats};
use crate::words::{self, Generator};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Time,
    Words,
    Quote,
    Zen,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Time => "time",
            Mode::Words => "words",
            Mode::Quote => "quote",
            Mode::Zen => "zen",
        }
    }

    pub fn parse(s: &str) -> Mode {
        match s {
            "words" => Mode::Words,
            "quote" => Mode::Quote,
            "zen" => Mode::Zen,
            _ => Mode::Time,
        }
    }
}

fn default_language() -> String {
    "en".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub mode: Mode,
    /// Seconds for time, word count for words, length bucket for quotes
    /// (0 any, 1 short, 2 medium, 3 long), unused for zen.
    #[serde(default)]
    pub value: u32,
    #[serde(default)]
    pub punctuation: bool,
    #[serde(default)]
    pub numbers: bool,
    #[serde(default)]
    pub stop_on_error: bool,
    /// No corrections at all: backspace does nothing.
    #[serde(default)]
    pub confidence: bool,
    #[serde(default = "default_language")]
    pub language: String,
    /// Fixed text for tests; normally random.
    #[serde(default)]
    pub seed: Option<u64>,
}

/// Words a time test keeps ahead of the caret.
const LOOKAHEAD: usize = 40;
/// Extra letters a word accepts past its end, so a stuck key can't grow it forever.
const MAX_EXTRA: usize = 20;

#[derive(Debug, Default, Serialize)]
pub struct Changed {
    pub index: usize,
    pub typed: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Update {
    pub current: usize,
    pub changed: Vec<Changed>,
    pub append: Vec<String>,
    pub started: bool,
    pub finished: bool,
}

pub struct Test {
    pub config: Config,
    pub words: Vec<String>,
    targets: Vec<Vec<char>>,
    pub typed: Vec<Vec<char>>,
    pub current: usize,
    pub keys: Vec<Keystroke>,
    origin: Option<f64>,
    pub elapsed: f64,
    pub finished: bool,
    pub quote: Option<(usize, String)>,
    generator: Generator,
}

impl Test {
    pub fn new(config: Config, previous_quote: Option<usize>) -> Test {
        let mut generator = Generator::new(&config.language, config.punctuation, config.numbers, config.seed);
        let mut quote = None;
        let words: Vec<String> = match config.mode {
            Mode::Time => generator.words(LOOKAHEAD * 2),
            Mode::Words => {
                let mut w = generator.words(config.value.clamp(1, 1000) as usize);
                generator.finish_text(&mut w);
                w
            }
            Mode::Quote => {
                let i = words::pick_quote(&config.language, config.value, previous_quote, generator.rng());
                let q = &words::quotes(&config.language)[i];
                quote = Some((i, q.source.to_string()));
                q.text.split_whitespace().map(String::from).collect()
            }
            Mode::Zen => Vec::new(),
        };
        let targets = words.iter().map(|w| w.chars().collect()).collect();
        Test {
            config,
            words,
            targets,
            typed: vec![Vec::new()],
            current: 0,
            keys: Vec::new(),
            origin: None,
            elapsed: 0.0,
            finished: false,
            quote,
            generator,
        }
    }

    fn zen(&self) -> bool {
        self.config.mode == Mode::Zen
    }

    pub fn started(&self) -> bool {
        self.origin.is_some()
    }

    /// The word as the text asks for it; in zen whatever was typed is right.
    pub fn target(&self, i: usize) -> &[char] {
        if self.zen() {
            &self.typed[i]
        } else {
            self.targets.get(i).map(Vec::as_slice).unwrap_or(&[])
        }
    }

    pub fn targets(&self) -> Vec<Vec<char>> {
        (0..self.typed.len().max(self.targets.len()))
            .map(|i| if self.zen() { self.typed.get(i).cloned().unwrap_or_default() } else { self.target(i).to_vec() })
            .collect()
    }

    fn since_start(&self, now: f64) -> f64 {
        self.origin.map(|o| (now - o).max(0.0)).unwrap_or(0.0)
    }

    fn deadline(&self) -> Option<f64> {
        (self.config.mode == Mode::Time).then(|| self.config.value.max(1) as f64 * 1000.0)
    }

    fn record(&mut self, t: f64, kind: Kind, ch: &str, expected: Option<char>, correct: bool) {
        self.keys.push(Keystroke {
            t,
            kind,
            ch: ch.to_string(),
            expected: expected.map(String::from).unwrap_or_default(),
            correct,
            word: self.current,
        });
    }

    fn update(&self, changed: &[usize]) -> Update {
        Update {
            current: self.current,
            changed: changed
                .iter()
                .map(|&i| Changed { index: i, typed: self.typed.get(i).map(|w| w.iter().collect()).unwrap_or_default() })
                .collect(),
            append: Vec::new(),
            started: self.started(),
            finished: self.finished,
        }
    }

    fn end(&mut self, elapsed: f64) {
        self.finished = true;
        self.elapsed = elapsed.max(1.0);
    }

    /// Ends a test that has no natural end (zen), at its last keystroke.
    pub fn finish(&mut self, now: f64) -> Update {
        if !self.finished && self.started() {
            let last = self.keys.last().map(|k| k.t).unwrap_or_else(|| self.since_start(now));
            self.end(last);
        }
        self.update(&[])
    }

    /// Ends a time test once its time is up. Returns whether it ended now.
    pub fn tick(&mut self, now: f64) -> bool {
        if self.finished || !self.started() {
            return false;
        }
        if let Some(deadline) = self.deadline() {
            if self.since_start(now) >= deadline {
                self.end(deadline);
                return true;
            }
        }
        false
    }

    pub fn input(&mut self, text: &str, now: f64) -> Update {
        let mut changed = Vec::new();
        let mut append = Vec::new();
        for ch in text.chars() {
            if self.finished || ch.is_control() {
                continue;
            }
            if self.origin.is_none() {
                self.origin = Some(now);
            }
            if self.tick(now) {
                break;
            }
            let t = self.since_start(now);
            if ch == ' ' {
                self.space(t, &mut changed, &mut append);
            } else {
                self.letter(ch, t, &mut changed);
            }
        }
        changed.sort_unstable();
        changed.dedup();
        let mut u = self.update(&changed);
        u.append = append;
        u
    }

    fn space(&mut self, t: f64, changed: &mut Vec<usize>, append: &mut Vec<String>) {
        let cur = self.current;
        if self.typed[cur].is_empty() {
            return;
        }
        let complete = self.typed[cur].as_slice() == self.target(cur);
        if self.config.stop_on_error && !complete {
            let expected = self.target(cur).get(self.typed[cur].len()).copied();
            self.record(t, Kind::Space, " ", expected, false);
            return;
        }
        self.record(t, Kind::Space, " ", Some(' '), complete);
        if matches!(self.config.mode, Mode::Words | Mode::Quote) && cur + 1 >= self.targets.len() {
            self.end(t);
            return;
        }
        self.current += 1;
        self.typed.push(Vec::new());
        changed.push(self.current);
        if self.config.mode == Mode::Time && self.current + LOOKAHEAD > self.words.len() {
            for _ in 0..LOOKAHEAD {
                let w = self.generator.next_word();
                self.targets.push(w.chars().collect());
                self.words.push(w.clone());
                append.push(w);
            }
        }
    }

    fn letter(&mut self, ch: char, t: f64, changed: &mut Vec<usize>) {
        let cur = self.current;
        let pos = self.typed[cur].len();
        let (expected, correct) = if self.zen() {
            (Some(ch), true)
        } else {
            let e = self.target(cur).get(pos).copied();
            (e, e == Some(ch))
        };
        if self.config.stop_on_error && !correct {
            self.record(t, Kind::Char, &ch.to_string(), expected, false);
            return;
        }
        if !self.zen() && pos >= self.target(cur).len() + MAX_EXTRA {
            return;
        }
        self.typed[cur].push(ch);
        self.record(t, Kind::Char, &ch.to_string(), expected, correct);
        changed.push(cur);
        // Words and quotes end the moment the last word is right, without a space.
        if matches!(self.config.mode, Mode::Words | Mode::Quote)
            && cur + 1 == self.targets.len()
            && self.typed[cur].as_slice() == self.target(cur)
        {
            self.end(t);
        }
    }

    pub fn backspace(&mut self, whole_word: bool, now: f64) -> Update {
        if self.finished || !self.started() || self.config.confidence {
            return self.update(&[]);
        }
        if self.tick(now) {
            return self.update(&[]);
        }
        let t = self.since_start(now);
        let kind = if whole_word { Kind::WordBack } else { Kind::Back };
        let cur = self.current;
        let mut changed = vec![cur];
        if !self.typed[cur].is_empty() {
            if whole_word {
                self.typed[cur].clear();
            } else {
                self.typed[cur].pop();
            }
            self.record(t, kind, "", None, true);
        } else if cur > 0 {
            // A correct word is done; only a wrong one can be reopened.
            let prev = cur - 1;
            if self.zen() || self.typed[prev].as_slice() != self.target(prev) {
                self.typed.pop();
                self.current = prev;
                if whole_word {
                    self.typed[prev].clear();
                }
                changed = vec![prev, cur];
                self.record(t, kind, "", None, true);
            }
        }
        self.update(&changed)
    }

    pub fn stats(&self, now: f64) -> Stats {
        let elapsed = if self.finished { self.elapsed } else { self.since_start(now) };
        stats::compute(&self.targets(), &self.typed, &self.keys, elapsed)
    }

    /// Seconds left in a time test, or words done out of the total otherwise.
    pub fn progress(&self, now: f64) -> serde_json::Value {
        match self.deadline() {
            Some(d) => serde_json::json!({ "remaining": ((d - self.since_start(now)) / 1000.0).max(0.0).ceil() }),
            None => serde_json::json!({ "done": self.current, "total": self.targets.len() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(mode: Mode, value: u32) -> Config {
        Config {
            mode,
            value,
            punctuation: false,
            numbers: false,
            stop_on_error: false,
            confidence: false,
            language: "en".into(),
            seed: Some(7),
        }
    }

    fn type_text(t: &mut Test, text: &str, start: f64) -> f64 {
        let mut now = start;
        for c in text.chars() {
            t.input(&c.to_string(), now);
            now += 100.0;
        }
        now
    }

    #[test]
    fn words_test_ends_on_the_last_correct_letter() {
        let mut t = Test::new(config(Mode::Words, 5), None);
        let text = t.words.join(" ");
        type_text(&mut t, &text, 1000.0);
        assert!(t.finished);
        let s = t.stats(0.0);
        assert_eq!(s.accuracy, 100.0);
        assert!(s.wpm > 0.0);
        assert_eq!(t.keys.len(), text.chars().count());
    }

    #[test]
    fn time_test_ends_at_the_deadline_and_refills_words() {
        let mut t = Test::new(config(Mode::Time, 120), None);
        let before = t.words.len();
        let text = t.words[..50].join(" ") + " ";
        let end = type_text(&mut t, &text, 0.0);
        assert!(end < 120_000.0 && !t.finished);
        assert!(t.words.len() > before, "words are added ahead of the caret");
        assert!(!t.tick(100_000.0));
        assert!(t.tick(121_000.0));
        assert_eq!(t.elapsed, 120_000.0);
    }

    #[test]
    fn backspace_reopens_only_wrong_words() {
        let mut t = Test::new(config(Mode::Words, 10), None);
        let first = t.words[0].clone();
        type_text(&mut t, &format!("{first} "), 0.0);
        t.backspace(false, 5000.0);
        assert_eq!(t.current, 1, "a correct word stays closed");

        let second = t.words[1].clone();
        type_text(&mut t, &format!("{}x ", second), 6000.0);
        t.backspace(false, 9000.0);
        assert_eq!(t.current, 1, "a wrong word can be reopened");
        assert_eq!(t.typed[1].iter().collect::<String>(), format!("{second}x"));
    }

    #[test]
    fn stop_on_error_blocks_wrong_letters_but_records_them() {
        let mut c = config(Mode::Words, 3);
        c.stop_on_error = true;
        let mut t = Test::new(c, None);
        let first = t.words[0].clone();
        t.input("#", 0.0);
        assert!(t.typed[0].is_empty());
        assert_eq!(t.keys.len(), 1);
        assert!(!t.keys[0].correct);
        type_text(&mut t, &first, 100.0);
        assert_eq!(t.typed[0].iter().collect::<String>(), first);
    }

    #[test]
    fn confidence_mode_ignores_backspace() {
        let mut c = config(Mode::Words, 3);
        c.confidence = true;
        let mut t = Test::new(c, None);
        t.input("q", 0.0);
        t.backspace(false, 100.0);
        assert_eq!(t.typed[0], vec!['q']);
    }

    #[test]
    fn zen_accepts_anything_until_finished() {
        let mut t = Test::new(config(Mode::Zen, 0), None);
        type_text(&mut t, "hello wrld ", 0.0);
        assert_eq!(t.current, 2);
        let u = t.finish(5000.0);
        assert!(u.finished);
        assert_eq!(t.stats(0.0).accuracy, 100.0);
    }

    #[test]
    fn quote_mode_uses_the_quote_words() {
        let t = Test::new(config(Mode::Quote, 1), None);
        let (i, _) = t.quote.clone().unwrap();
        assert_eq!(t.words.join(" "), words::QUOTES_EN[i].text);
    }
}
