//! omatype's engine: typing tests, their statistics, and the history store,
//! behind a small C interface. Every call takes and returns JSON, so the Qt
//! side stays a thin shell and the rules live in one place.

pub mod stats;
pub mod store;
pub mod test;
pub mod words;

use std::ffi::{c_char, CStr, CString};
use std::path::Path;

use serde_json::{json, Value};

use store::Store;
use test::{Config, Test};

pub struct Engine {
    store: Result<Store, String>,
    test: Option<Test>,
    last_quote: Option<usize>,
}

impl Engine {
    pub fn new(store: Result<Store, String>) -> Engine {
        Engine { store, test: None, last_quote: None }
    }

    fn store(&mut self) -> Result<&mut Store, String> {
        self.store.as_mut().map_err(|e| format!("history unavailable: {e}"))
    }

    /// Saves the test if it just ended and returns its result.
    fn settle(&mut self, now: f64) -> Result<Option<Value>, String> {
        let Some(t) = self.test.as_ref().filter(|t| t.finished) else { return Ok(None) };
        let stats = t.stats(now);
        let result = match self.store.as_mut() {
            Ok(store) => store.save(t, &stats)?,
            // Without a history the result is still shown, just not kept.
            Err(_) => {
                let mut v = json!(stats);
                v["mode"] = json!(t.config.mode.name());
                v["value"] = json!(t.config.value);
                v["pb"] = json!(false);
                v
            }
        };
        self.test = None;
        Ok(Some(result))
    }

    fn update(&mut self, update: test::Update, now: f64) -> Result<Value, String> {
        let mut v = json!(update);
        if let Some(result) = self.settle(now)? {
            v["result"] = result;
        }
        Ok(v)
    }

    fn live(&self, now: f64) -> Value {
        match &self.test {
            Some(t) if t.started() => {
                let s = t.stats(now);
                json!({ "wpm": s.wpm, "raw": s.raw, "accuracy": s.accuracy, "progress": t.progress(now) })
            }
            Some(t) => json!({ "wpm": 0, "raw": 0, "accuracy": 100, "progress": t.progress(now) }),
            None => Value::Null,
        }
    }

    pub fn call(&mut self, method: &str, args: &Value) -> Result<Value, String> {
        let now = args["now"].as_f64().unwrap_or(0.0);
        let int = |key: &str, default: i64| args[key].as_i64().unwrap_or(default);
        match method {
            "status" => Ok(json!({ "storeError": self.store.as_ref().err() })),
            "start" => {
                let config: Config = serde_json::from_value(args["config"].clone()).map_err(|e| e.to_string())?;
                let t = Test::new(config, self.last_quote);
                if let Some((i, _)) = &t.quote {
                    self.last_quote = Some(*i);
                }
                let v = json!({
                    "words": t.words,
                    "quote": t.quote.as_ref().map(|q| q.1.clone()),
                    "live": Value::Null,
                });
                self.test = Some(t);
                let mut v = v;
                v["live"] = self.live(now);
                Ok(v)
            }
            "input" => {
                let text = args["text"].as_str().unwrap_or_default();
                let u = self.test.as_mut().ok_or("no test")?.input(text, now);
                self.update(u, now)
            }
            "backspace" => {
                let word = args["word"].as_bool().unwrap_or(false);
                let u = self.test.as_mut().ok_or("no test")?.backspace(word, now);
                self.update(u, now)
            }
            "finish" => {
                let u = self.test.as_mut().ok_or("no test")?.finish(now);
                self.update(u, now)
            }
            "tick" => {
                let ended = self.test.as_mut().is_some_and(|t| t.tick(now));
                let live = self.live(now);
                let mut v = json!({ "finished": ended, "live": live });
                if let Some(result) = self.settle(now)? {
                    v["result"] = result;
                }
                Ok(v)
            }
            "test" => self.store()?.get(int("id", 0)).map(|v| v.unwrap_or(Value::Null)),
            "recent" => self.store()?.recent(int("limit", 50), int("offset", 0)),
            "daily" => self.store()?.daily(int("days", 30)),
            "bests" => self.store()?.bests(),
            "totals" => self.store()?.totals(),
            "weakKeys" => self.store()?.weak_keys(int("limit", 10), int("minSeen", 20)),
            "delete" => {
                let id = int("id", 0);
                self.store()?.delete(id).map(Value::from)
            }
            _ => Err(format!("unknown method {method}")),
        }
    }
}

fn to_c(value: Value) -> *mut c_char {
    CString::new(value.to_string()).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
}

/// Opens the engine with its history at `db_path`. A history that can't be
/// opened leaves the engine usable; `status` reports why.
///
/// # Safety
/// `db_path` must be a valid NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn omt_open(db_path: *const c_char) -> *mut Engine {
    let path = if db_path.is_null() { String::new() } else { CStr::from_ptr(db_path).to_string_lossy().into_owned() };
    let store = if path.is_empty() { Store::memory() } else { Store::open(Path::new(&path)) };
    Box::into_raw(Box::new(Engine::new(store)))
}

/// Runs `method` with JSON `args` and returns `{"ok": true, "data": …}` or
/// `{"ok": false, "error": "…"}`, to be released with `omt_free`.
///
/// # Safety
/// `engine` must come from `omt_open`; the strings must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn omt_call(engine: *mut Engine, method: *const c_char, args: *const c_char) -> *mut c_char {
    let Some(engine) = engine.as_mut() else { return to_c(json!({ "ok": false, "error": "no engine" })) };
    let method = CStr::from_ptr(method).to_string_lossy();
    let args = if args.is_null() { Value::Null } else { serde_json::from_slice(CStr::from_ptr(args).to_bytes()).unwrap_or(Value::Null) };
    to_c(match engine.call(&method, &args) {
        Ok(data) => json!({ "ok": true, "data": data }),
        Err(error) => json!({ "ok": false, "error": error }),
    })
}

/// # Safety
/// `s` must come from `omt_call`, once.
#[no_mangle]
pub unsafe extern "C" fn omt_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// # Safety
/// `engine` must come from `omt_open`, once.
#[no_mangle]
pub unsafe extern "C" fn omt_close(engine: *mut Engine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whole_test_through_the_json_interface() {
        let mut e = Engine::new(Store::memory());
        let started = e
            .call("start", &json!({ "config": { "mode": "words", "value": 3, "seed": 9 }, "now": 0 }))
            .unwrap();
        let text = started["words"].as_array().unwrap().iter().map(|w| w.as_str().unwrap()).collect::<Vec<_>>().join(" ");
        let mut last = Value::Null;
        for (i, c) in text.chars().enumerate() {
            last = e.call("input", &json!({ "text": c.to_string(), "now": i as f64 * 90.0 })).unwrap();
        }
        assert_eq!(last["finished"], json!(true));
        assert_eq!(last["result"]["accuracy"], json!(100.0));
        assert_eq!(e.call("totals", &json!({})).unwrap()["tests"], json!(1));
    }
}
