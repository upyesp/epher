# 0010 — The Desktop App Owns the Native Store; the Webview Bridges to It

Date: 2026-08-14 · Status: accepted · Supersedes: nothing (narrows ADR-0003's
"desktop" column; the FSA bridge remains the plan for the browser PWA)

## Context

The Tauri desktop app shipped as the PWA in a window: it evaluated in the
webview but touched no storage, so functions, scripts, history, and the
language preference saved in the CLI/TUI were invisible to it — and nothing
saved in the desktop app survived a restart. Users reasonably expect "epher"
to be one calculator with one body of saved work across its versions
(`~/.epher`, ADR-0002).

ADR-0001 already fixed where evaluation lives: the webview, on the wasm
core, behind the one Yew frontend. Moving evaluation into the native
process would split the frontend in two and abandon that decision.

## Decision

The desktop app's **native process owns the Native Store**. The webview
bridges to it over Tauri IPC:

- The Tauri shell manages a `DocStore<FsStore>` rooted at
  `default_store_dir()` (`EPHER_STORE_DIR` override, `~/.epher` default) —
  the same files, same schema, same atomic writes as the CLI and TUI.
- It exposes five thin commands — `init`, `save_function`, `save_script`,
  `save_history`, `save_language` — that delegate to `epher_store::persist`.
- On startup the frontend calls `init`, receiving history, the replay
  lines (saved functions, then scripts), and the language preference. It
  rebuilds its wasm `Session` with `Session::with_history` +
  `submit_quiet` per line — the exact `load_session` recipe — so saved
  functions, and any variables set by saved scripts, are restored.
- The shell-command surface (`save`, `save script`, `language`) is shared
  by every interactive frontend through the new `epher-shell` crate: one
  `classify`/`prepare` policy, native adapters persist synchronously, the
  webview persists through the IPC bridge. (This also closes a real gap:
  the TUI never implemented these commands.)
- The PWA keeps session-only behavior (bridge kind `None`): browser
  storage and the FSA bridge (ADR-0003) remain deferred work, unchanged.
- History is persisted after every submitted line (desktop apps are
  killed, not exited — the CLI's save-on-exit doesn't apply).

We rejected: evaluation in the native process (splits the frontend,
contradicts ADR-0001); the webview writing files directly via a Tauri fs
plugin (store code — schema, atomicity, naming — would be duplicated in
JS instead of reused); and autosaving every definition without the
explicit `save` command (would diverge from CLI/TUI semantics).

## Consequences

- One store, four native participants (CLI, TUI, desktop, and the future
  FSA-bridged PWA): last-write-wins per document across concurrently
  running frontends — accepted (ADR-0003 already accepted this).
- The frontend gains a small bridge seam (`None` | `Tauri`) and three
  wasm-only dependencies (js-sys, wasm-bindgen-futures,
  serde-wasm-bindgen); command messages localize through the same Fluent
  catalogs as the native shells.
- `window.__TAURI__` (withGlobalTauri) is the bridge's detection signal.
- UI text beyond command messages stays English until the web-i18n wiring
  lands (unchanged, harness-blocked).
