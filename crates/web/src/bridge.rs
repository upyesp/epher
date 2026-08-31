//! The web frontend's storage bridge (ADR-0010).
//!
//! Inside the Tauri desktop shell the webview persists through the native
//! store over IPC (`window.__TAURI__`, exposed by `withGlobalTauri`). In the
//! browser PWA there is no bridge — work lives in the session only — until
//! the browser store lands (ADR-0002/0003, deferred).

use serde::Deserialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

/// The startup answer from the desktop shell: the same data
/// `persist::load_session` would use natively.
#[derive(Debug, Default, Deserialize)]
pub struct InitState {
    pub history: Vec<String>,
    pub replay: Vec<String>,
    pub language: Option<String>,
    pub theme: Option<String>,
    /// The shared session snapshot (ADR-0010 amendment): the bindings
    /// saved by whichever CLI/REPL/TUI/desktop frontend ran last.
    pub session: std::collections::HashMap<String, epher_core::Value>,
}

/// Which persistence backend this frontend instance can reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bridge {
    /// Browser PWA: no persistence.
    None,
    /// Tauri desktop shell: the Native Store over IPC.
    Tauri,
}

impl Bridge {
    /// `window.__TAURI__` is the desktop shell's marker.
    pub fn detect() -> Self {
        if let Some(window) = web_sys::window() {
            let marker = JsValue::from_str("__TAURI__");
            let present = js_sys::Reflect::get(&window, &marker)
                .map(|v| !v.is_undefined() && !v.is_null())
                .unwrap_or(false);
            if present {
                return Bridge::Tauri;
            }
        }
        Bridge::None
    }

    /// Load history, replay lines, and the language preference.
    pub async fn init(self) -> Result<InitState, String> {
        let value = self
            .invoke("init", &JsValue::UNDEFINED)
            .await
            .map_err(js_err)?;
        serde_wasm_bindgen::from_value(value).map_err(|e| e.to_string())
    }

    /// Subscribe to the desktop shell's `store-changed` broadcasts
    /// (ADR-0010 amendment): every write to the native store — this
    /// window's own or another frontend's (TUI, REPL, one-shot CLI) —
    /// arrives as the fresh [`InitState`], and the caller applies it.
    /// The payload is the same shape as `init`'s answer.
    pub fn listen_store_changed(cb: impl Fn(InitState) + 'static) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let tauri_marker = JsValue::from_str("__TAURI__");
        let Ok(tauri) = js_sys::Reflect::get(&window, &tauri_marker) else {
            return;
        };
        if tauri.is_undefined() || tauri.is_null() {
            return;
        }
        let Ok(event_api) = js_sys::Reflect::get(&tauri, &JsValue::from_str("event")) else {
            return;
        };
        let Ok(listen_fn) = js_sys::Reflect::get(&event_api, &JsValue::from_str("listen")) else {
            return;
        };
        let Ok(listen_fn) = listen_fn.dyn_into::<js_sys::Function>() else {
            return;
        };
        let handler = Closure::wrap(Box::new(move |ev: JsValue| {
            if let Ok(payload) = js_sys::Reflect::get(&ev, &JsValue::from_str("payload")) {
                if let Ok(state) = serde_wasm_bindgen::from_value::<InitState>(payload) {
                    cb(state);
                }
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = listen_fn.call2(
            &event_api,
            &JsValue::from_str("store-changed"),
            handler.as_ref(),
        );
        handler.forget();
    }

    /// Fire-and-forget saves: the UI already showed the prepared message;
    /// a late failure is surfaced through the status region.
    pub fn save_function(self, name: &str, source: &str) {
        let args =
            serde_wasm_bindgen::to_value(&SaveArgs { name, source }).unwrap_or(JsValue::UNDEFINED);
        self.spawn("save_function", args);
    }

    pub fn save_constant(self, name: &str, source: &str) {
        let args =
            serde_wasm_bindgen::to_value(&SaveArgs { name, source }).unwrap_or(JsValue::UNDEFINED);
        self.spawn("save_constant", args);
    }

    pub fn save_script(self, name: &str, source: &str) {
        let args =
            serde_wasm_bindgen::to_value(&SaveArgs { name, source }).unwrap_or(JsValue::UNDEFINED);
        self.spawn("save_script", args);
    }

    pub fn save_history(self, history: &[String]) {
        let args =
            serde_wasm_bindgen::to_value(&HistoryArgs { history }).unwrap_or(JsValue::UNDEFINED);
        self.spawn("save_history", args);
    }

    /// Persist the shared session snapshot (ADR-0010 amendment): the
    /// environment's bindings — user assignments and `ans` — so the next
    /// CLI/REPL/TUI/desktop frontend starts where this one left off.
    /// The bindings travel as an array of pairs, not the HashMap itself:
    /// serde_wasm_bindgen serializes HashMap as a JS Map, which the
    /// Linux webkitgtk IPC cannot transport (the save silently never
    /// arrived and setting/session.json was never written); an array of
    /// [name, value] pairs is plain JSON on every platform.
    pub fn save_session_state(
        self,
        bindings: &std::collections::HashMap<String, epher_core::Value>,
    ) {
        let pairs: Vec<(String, epher_core::Value)> = bindings
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        // The same struct shape as every other save command: a plain
        // object survives every IPC transport (the raw HashMap rendered
        // as a JS Map, which the Linux webkitgtk IPC drops; even a bare
        // array of pairs never arrived, so the struct is the safest
        // wire format).
        let args = serde_wasm_bindgen::to_value(&SessionArgs { bindings: pairs })
            .unwrap_or(JsValue::UNDEFINED);
        self.spawn("save_session", args);
    }

    pub fn save_language(self, code: &str) {
        let args = serde_wasm_bindgen::to_value(&CodeArgs { code }).unwrap_or(JsValue::UNDEFINED);
        self.spawn("save_language", args);
    }

    pub fn save_theme(self, name: &str) {
        let args = serde_wasm_bindgen::to_value(&ThemeArgs { name }).unwrap_or(JsValue::UNDEFINED);
        self.spawn("save_theme", args);
    }

    /// Quit the desktop app (File → Quit). No response — the process
    /// ends before one could arrive.
    pub async fn quit(self) {
        let _ = self.invoke("quit", &JsValue::UNDEFINED).await;
    }

    /// File → Save history/script (ADR-0024): the desktop shell's native
    /// save dialog. `None` = the user cancelled (the UI stays silent);
    /// `Some(path)` is the file actually written.
    pub async fn save_file_dialog(
        self,
        content: &str,
        default_name: &str,
    ) -> Result<Option<String>, String> {
        let args = serde_wasm_bindgen::to_value(&SaveFileArgs {
            content,
            default_name,
        })
        .map_err(|e| e.to_string())?;
        let value = self
            .invoke("save_file_dialog", &args)
            .await
            .map_err(js_err)?;
        if value.is_null() || value.is_undefined() {
            return Ok(None);
        }
        serde_wasm_bindgen::from_value::<Option<String>>(value).map_err(|e| e.to_string())
    }

    /// File → Save PNG (ADR-0042): the desktop save dialog for the
    /// rasterized plot bytes, exactly like [`Self::save_file_dialog`] for
    /// text. `None` = the user cancelled (the UI stays silent).
    pub async fn save_png_dialog(
        self,
        data: &[u8],
        default_name: &str,
    ) -> Result<Option<String>, String> {
        let args = serde_wasm_bindgen::to_value(&SavePngArgs {
            data: data.to_vec(),
            default_name,
        })
        .map_err(|e| e.to_string())?;
        let value = self
            .invoke("save_png_dialog", &args)
            .await
            .map_err(js_err)?;
        if value.is_null() || value.is_undefined() {
            return Ok(None);
        }
        serde_wasm_bindgen::from_value::<Option<String>>(value).map_err(|e| e.to_string())
    }

    /// Can this desktop shell install the `epher` terminal command?
    /// (macOS only, ADR-0011.) The UI asks at startup.
    pub async fn cli_install_supported(self) -> Result<bool, String> {
        let value = self
            .invoke("cli_install_supported", &JsValue::UNDEFINED)
            .await
            .map_err(js_err)?;
        serde_wasm_bindgen::from_value(value).map_err(|e| e.to_string())
    }

    /// Install the `epher` terminal command. Ok carries a Fluent key;
    /// Err carries readable instructions to show.
    pub async fn install_cli(self) -> Result<String, String> {
        let value = self
            .invoke("install_cli", &JsValue::UNDEFINED)
            .await
            .map_err(js_err)?;
        serde_wasm_bindgen::from_value(value).map_err(|e| e.to_string())
    }

    fn spawn(self, cmd: &'static str, args: JsValue) {
        spawn_local(async move {
            if let Err(e) = self.invoke(cmd, &args).await {
                web_sys::console::error_1(&e);
            }
        });
    }

    async fn invoke(self, cmd: &str, args: &JsValue) -> Result<JsValue, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let marker = JsValue::from_str("__TAURI__");
        let tauri = js_sys::Reflect::get(&window, &marker)?;
        let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core"))?;
        let invoke = js_sys::Reflect::get(&core, &JsValue::from_str("invoke"))?
            .dyn_into::<js_sys::Function>()?;
        let promise = invoke.call2(&invoke, &JsValue::from_str(cmd), args)?;
        JsFuture::from(promise.dyn_into::<js_sys::Promise>()?).await
    }
}

/// Render a JS error value as a readable message.
fn js_err(e: JsValue) -> String {
    e.as_string().unwrap_or_else(|| "IPC error".to_string())
}

#[derive(serde::Serialize)]
struct SaveArgs<'a> {
    name: &'a str,
    source: &'a str,
}

/// Invoke arguments are camelCase: the Tauri command macro renames
/// them back to snake_case Rust parameters (`defaultName` →
/// `default_name`). v0.4.13 sent `default_name` and every desktop save
/// dialog failed with "missing required key defaultName" (ADR-0026).
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveFileArgs<'a> {
    content: &'a str,
    default_name: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SavePngArgs<'a> {
    data: Vec<u8>,
    default_name: &'a str,
}

#[derive(serde::Serialize)]
struct SessionArgs {
    bindings: Vec<(String, epher_core::Value)>,
}

#[derive(serde::Serialize)]
struct HistoryArgs<'a> {
    history: &'a [String],
}

#[derive(serde::Serialize)]
struct CodeArgs<'a> {
    code: &'a str,
}

#[derive(serde::Serialize)]
struct ThemeArgs<'a> {
    name: &'a str,
}
