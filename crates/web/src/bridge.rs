//! The web frontend's storage bridge (ADR-0010).
//!
//! Inside the Tauri desktop shell the webview persists through the native
//! store over IPC (`window.__TAURI__`, exposed by `withGlobalTauri`). In the
//! browser PWA there is no bridge — work lives in the session only — until
//! the browser store lands (ADR-0002/0003, deferred).

use serde::Deserialize;
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

    pub fn save_language(self, code: &str) {
        let args = serde_wasm_bindgen::to_value(&CodeArgs { code }).unwrap_or(JsValue::UNDEFINED);
        self.spawn("save_language", args);
    }

    pub fn save_theme(self, name: &str) {
        let args =
            serde_wasm_bindgen::to_value(&ThemeArgs { name }).unwrap_or(JsValue::UNDEFINED);
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
        let value = self.invoke("save_file_dialog", &args).await.map_err(js_err)?;
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

#[derive(serde::Serialize)]
struct SaveFileArgs<'a> {
    content: &'a str,
    default_name: &'a str,
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
