//! Speed, accuracy, and consistency, all derived from the saved keystrokes
//! and the final state of the words, so a stored test can be recomputed (and
//! re-graphed) exactly as it was shown when it finished.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// A printable character other than space.
    Char,
    /// The space that submits a word.
    Space,
    /// Backspace, one character (or a step back into the previous word).
    Back,
    /// Ctrl+Backspace, the whole word.
    WordBack,
}

impl Kind {
    pub fn code(self) -> i64 {
        match self {
            Kind::Char => 0,
            Kind::Space => 1,
            Kind::Back => 2,
            Kind::WordBack => 3,
        }
    }

    pub fn from_code(code: i64) -> Kind {
        match code {
            1 => Kind::Space,
            2 => Kind::Back,
            3 => Kind::WordBack,
            _ => Kind::Char,
        }
    }

    /// Characters and spaces are what speed and accuracy count; corrections
    /// are only recorded.
    pub fn is_typed(self) -> bool {
        matches!(self, Kind::Char | Kind::Space)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Keystroke {
    /// Milliseconds since the first keystroke of the test.
    pub t: f64,
    pub kind: Kind,
    /// What was typed (empty for corrections).
    pub ch: String,
    /// What the text asked for at that position (empty past the word's end).
    pub expected: String,
    pub correct: bool,
    pub word: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Second {
    /// End of the interval, in seconds since the start.
    pub t: f64,
    /// Net speed so far.
    pub wpm: f64,
    /// Raw speed within this second.
    pub raw: f64,
    pub errors: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub duration: f64,
    pub wpm: f64,
    pub raw: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub correct: u32,
    pub incorrect: u32,
    pub extra: u32,
    pub missed: u32,
    pub seconds: Vec<Second>,
}

fn per_minute(chars: f64, ms: f64) -> f64 {
    if ms <= 0.0 { 0.0 } else { chars / 5.0 / (ms / 60000.0) }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Maps the coefficient of variation of per-second speed to 0-100, the way
/// Monkeytype does, so a steady typist scores near 100 and an erratic one
/// near 0 without the scale ever going negative.
pub fn consistency(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return if values.is_empty() { 0.0 } else { 100.0 };
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if mean <= 0.0 {
        return 0.0;
    }
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let cv = variance.sqrt() / mean;
    100.0 * (1.0 - (cv + cv.powi(3) / 3.0 + cv.powi(5) / 5.0).tanh())
}

/// `targets[i]` is what word i asked for and `typed[i]` what ended up in it;
/// `typed` stops at the word the test ended in. `elapsed` is in milliseconds.
pub fn compute(targets: &[Vec<char>], typed: &[Vec<char>], keys: &[Keystroke], elapsed: f64) -> Stats {
    let elapsed = elapsed.max(1.0);
    let mut s = Stats { duration: round2(elapsed / 1000.0), ..Default::default() };

    // Net speed counts the characters of correct words and the spaces after
    // them; a last word still in progress counts if it's right so far.
    let mut good_chars = 0usize;
    let words = typed.len();
    for (i, tp) in typed.iter().enumerate() {
        let empty = Vec::new();
        let tgt = targets.get(i).unwrap_or(&empty);
        let last = i + 1 == words;
        for j in 0..tp.len().max(tgt.len()) {
            match (tp.get(j), tgt.get(j)) {
                (Some(a), Some(b)) if a == b => s.correct += 1,
                (Some(_), Some(_)) => s.incorrect += 1,
                (Some(_), None) => s.extra += 1,
                (None, Some(_)) if !last => s.missed += 1,
                _ => {}
            }
        }
        if !tp.is_empty() && tp == tgt {
            good_chars += tgt.len() + usize::from(!last);
        } else if last && !tp.is_empty() && tgt.starts_with(tp) {
            good_chars += tp.len();
        }
    }

    let typed_keys: Vec<&Keystroke> = keys.iter().filter(|k| k.kind.is_typed()).collect();
    let right = typed_keys.iter().filter(|k| k.correct).count();
    s.wpm = round2(per_minute(good_chars as f64, elapsed));
    s.raw = round2(per_minute(typed_keys.len() as f64, elapsed));
    s.accuracy = if typed_keys.is_empty() { 0.0 } else { round2(100.0 * right as f64 / typed_keys.len() as f64) };

    // One point per second. A sliver of a second at the end has too few
    // keystrokes to give a meaningful rate, so it folds into the one before.
    let mut buckets = (elapsed / 1000.0).ceil().max(1.0) as usize;
    if buckets > 1 && elapsed - (buckets - 1) as f64 * 1000.0 < 500.0 {
        buckets -= 1;
    }
    let mut cumulative_right = 0usize;
    let mut k = 0usize;
    for b in 0..buckets {
        let start = b as f64 * 1000.0;
        let end = if b + 1 == buckets { elapsed } else { start + 1000.0 };
        let (mut count, mut errors) = (0usize, 0u32);
        while k < typed_keys.len() && (typed_keys[k].t < end || b + 1 == buckets) {
            count += 1;
            if typed_keys[k].correct {
                cumulative_right += 1;
            } else {
                errors += 1;
            }
            k += 1;
        }
        s.seconds.push(Second {
            t: round2(end / 1000.0),
            wpm: round2(per_minute(cumulative_right as f64, end)),
            raw: round2(per_minute(count as f64, end - start)),
            errors,
        });
    }
    // The net line ends on the figure the result shows, which also discounts
    // words that were left wrong.
    if let Some(last) = s.seconds.last_mut() {
        last.wpm = s.wpm;
    }
    let raws: Vec<f64> = s.seconds.iter().map(|x| x.raw).collect();
    s.consistency = round2(consistency(&raws));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(t: f64, ch: char, expected: char) -> Keystroke {
        Keystroke {
            t,
            kind: if ch == ' ' { Kind::Space } else { Kind::Char },
            ch: ch.to_string(),
            expected: expected.to_string(),
            correct: ch == expected,
            word: 0,
        }
    }

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn perfect_run_has_full_accuracy_and_matching_speeds() {
        // "the cat" in 6 seconds, one key per second: 7 chars.
        let text = "the cat";
        let keys: Vec<_> = text.chars().enumerate().map(|(i, c)| key(i as f64 * 1000.0, c, c)).collect();
        let s = compute(&[chars("the"), chars("cat")], &[chars("the"), chars("cat")], &keys, 6000.0);
        assert_eq!(s.accuracy, 100.0);
        assert_eq!(s.correct, 6);
        assert_eq!(s.wpm, s.raw);
        assert_eq!(s.wpm, 14.0); // 7 chars / 5 / 0.1 min
        assert_eq!(s.seconds.len(), 6);
    }

    #[test]
    fn wrong_word_does_not_count_toward_net_speed() {
        let mut space = key(1500.0, ' ', ' ');
        space.correct = false;
        let keys = vec![key(0.0, 't', 't'), key(500.0, 'x', 'h'), key(1000.0, 'e', 'e'), space];
        let s = compute(&[chars("the"), chars("cat")], &[chars("txe"), vec![]], &keys, 2000.0);
        assert_eq!(s.wpm, 0.0);
        assert_eq!(s.incorrect, 1);
        assert_eq!(s.accuracy, 50.0); // the space after a wrong word is wrong too
        assert_eq!(s.seconds.iter().map(|x| x.errors).sum::<u32>(), 2);
    }

    #[test]
    fn missed_and_extra_are_counted() {
        let s = compute(&[chars("cat"), chars("dog")], &[chars("ca"), chars("dogs")], &[], 1000.0);
        assert_eq!((s.correct, s.missed, s.extra), (5, 1, 1));
    }

    #[test]
    fn steady_typing_is_consistent() {
        assert!(consistency(&[60.0, 60.0, 60.0]) > 99.0);
        assert!(consistency(&[10.0, 120.0, 30.0, 90.0]) < 60.0);
    }
}
