//! Persistence helpers shared by the native frontends (CLI/TUI/desktop):
//! loading a [`Session`] from the store and saving history/functions. Logic
//! once — the web frontend uses the same [`DocStore`] seam with its own
//! backend.

use crate::{ConstantDoc, DocStore, FunctionDoc, ScriptDoc, Storage, StoreError, StoreResult};
use epher_core::Session;

pub const HISTORY_SETTING: &str = "history";
/// The user's language override (ADR-0008): detection is the default, this
/// setting wins when set.
pub const LANGUAGE_SETTING: &str = "language";

/// The store directory for native frontends: `EPHER_STORE_DIR` override, else
/// `~/.epher` (falls back to `.epher`).
pub fn default_store_dir() -> std::path::PathBuf {
    std::env::var_os("EPHER_STORE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(|h| std::path::PathBuf::from(h).join(".epher"))
                .unwrap_or_else(|| std::path::PathBuf::from(".epher"))
        })
}

/// The saved lines to replay at startup, in load order: functions first,
/// then constants, then scripts (the recipe [`load_session`] applies
/// natively; the desktop webview replays them into its own Session —
/// ADR-0010). Constants may call functions; scripts may use both.
pub fn replay_lines<S: Storage>(store: &DocStore<S>) -> StoreResult<Vec<String>> {
    let mut lines = Vec::new();
    for doc in store.list_functions()? {
        lines.push(doc.source);
    }
    for doc in store.list_constants()? {
        lines.push(doc.source);
    }
    for doc in store.list_scripts()? {
        lines.push(doc.source);
    }
    Ok(lines)
}

/// Rebuild a session from the store: history plus saved functions,
/// constants, and scripts (re-run as definitions), then the shared
/// session snapshot (user bindings, `ans` among them — ADR-0010
/// amendment) so a frontend starts where the last one left off.
pub fn load_session<S: Storage>(store: &DocStore<S>) -> StoreResult<Session> {
    let history = history(store)?;
    let mut session = Session::with_history(history);
    for line in replay_lines(store)? {
        session.submit_quiet(&line);
    }
    if let Some(bindings) = session_bindings(store)? {
        session.restore_bindings(&bindings);
    }
    Ok(session)
}

pub fn history<S: Storage>(store: &DocStore<S>) -> StoreResult<Vec<String>> {
    match store.get_setting(HISTORY_SETTING)? {
        Some(value) => {
            serde_json::from_value(value).map_err(|e| StoreError::Serialize(e.to_string()))
        }
        None => Ok(Vec::new()),
    }
}

pub fn save_history<S: Storage>(store: &DocStore<S>, history: &[String]) -> StoreResult<()> {
    let value = serde_json::to_value(history).map_err(|e| StoreError::Serialize(e.to_string()))?;
    store.set_setting(HISTORY_SETTING, value)
}

pub const SESSION_SETTING: &str = "session";

/// The shared session snapshot (ADR-0010 amendment): the environment's
/// variable bindings — user assignments and `ans` — saved by whichever
/// interactive frontend ran last, restored by the next one. One
/// installation, one calculator state, across CLI/REPL/TUI/desktop.
pub fn session_bindings<S: Storage>(
    store: &DocStore<S>,
) -> StoreResult<Option<epher_core::ValueBindings>> {
    match store.get_setting(SESSION_SETTING)? {
        Some(value) => serde_json::from_value(value)
            .map(Some)
            .map_err(|e| StoreError::Serialize(e.to_string())),
        None => Ok(None),
    }
}

pub fn save_session<S: Storage>(
    store: &DocStore<S>,
    bindings: &epher_core::ValueBindings,
) -> StoreResult<()> {
    let value = serde_json::to_value(bindings).map_err(|e| StoreError::Serialize(e.to_string()))?;
    store.set_setting(SESSION_SETTING, value)
}

pub fn save_function<S: Storage>(store: &DocStore<S>, name: &str, source: &str) -> StoreResult<()> {
    store.put_function(&FunctionDoc {
        name: name.to_string(),
        source: source.to_string(),
    })
}

pub fn save_constant<S: Storage>(store: &DocStore<S>, name: &str, source: &str) -> StoreResult<()> {
    store.put_constant(&ConstantDoc {
        name: name.to_string(),
        source: source.to_string(),
    })
}

pub fn save_script<S: Storage>(store: &DocStore<S>, name: &str, source: &str) -> StoreResult<()> {
    store.put_script(&ScriptDoc {
        name: name.to_string(),
        source: source.to_string(),
    })
}

pub fn load_language<S: Storage>(store: &DocStore<S>) -> StoreResult<Option<String>> {
    match store.get_setting(LANGUAGE_SETTING)? {
        Some(value) => Ok(value.as_str().map(String::from)),
        None => Ok(None),
    }
}

pub fn save_language<S: Storage>(store: &DocStore<S>, language: &str) -> StoreResult<()> {
    store.set_setting(LANGUAGE_SETTING, serde_json::json!(language))
}

/// The user's theme override (ADR-0017): light, dark, or night. Detection
/// (dark) is the default; the setting wins when present.
pub const THEME_SETTING: &str = "theme";

pub fn load_theme<S: Storage>(store: &DocStore<S>) -> StoreResult<Option<String>> {
    match store.get_setting(THEME_SETTING)? {
        Some(value) => Ok(value.as_str().map(String::from)),
        None => Ok(None),
    }
}

pub fn save_theme<S: Storage>(store: &DocStore<S>, theme: &str) -> StoreResult<()> {
    store.set_setting(THEME_SETTING, serde_json::json!(theme))
}

/// Whether the graph panel lists the points of interest (default yes).
/// A display toggle owned by the Settings menu — the analysis itself
/// always runs, so switching back is instant.
pub const POIS_SETTING: &str = "pois";

pub fn load_pois<S: Storage>(store: &DocStore<S>) -> StoreResult<Option<bool>> {
    match store.get_setting(POIS_SETTING)? {
        Some(value) => Ok(value.as_bool()),
        None => Ok(None),
    }
}

pub fn save_pois<S: Storage>(store: &DocStore<S>, pois: bool) -> StoreResult<()> {
    store.set_setting(POIS_SETTING, serde_json::json!(pois))
}
