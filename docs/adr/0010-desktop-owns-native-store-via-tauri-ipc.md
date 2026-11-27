# 0010: The Desktop App Owns the Native Store; the Webview Bridges to It

Date: 2026-08-14 · Status: accepted · Supersedes: nothing (narrows ADR-0003's
"desktop" column; the FSA bridge remains the plan for the browser PWA)

## Context

The Tauri desktop app shipped as the PWA in a window: it evaluated in the
webview but touched no storage, so functions, scripts, history, and the
language preference saved in the CLI/TUI were invisible to it, and nothing
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
  `default_store_dir()` (`EPHER_STORE_DIR` override, `~/.epher` default),
  the same files, same schema, same atomic writes as the CLI and TUI.
- It exposes five thin commands, `init`, `save_function`, `save_script`,
  `save_history`, `save_language`, that delegate to `epher_store::persist`.
- On startup the frontend calls `init`, receiving history, the replay
  lines (saved functions, then scripts), and the language preference. It
  rebuilds its wasm `Session` with `Session::with_history` +
  `submit_quiet` per line, the exact `load_session` recipe, so saved
  functions, and any variables set by saved scripts, are restored.
- The shell-command surface (`save`, `save script`, `language`) is shared
  by every interactive frontend through the new `epher-shell` crate: one
  `classify`/`prepare` policy, native adapters persist synchronously, the
  webview persists through the IPC bridge. (This also closes a real gap:
  the TUI never implemented these commands.)
- The PWA keeps session-only behavior (bridge kind `None`): browser
  storage and the FSA bridge (ADR-0003) remain deferred work, unchanged.
- History is persisted after every submitted line (desktop apps are
  killed, not exited; the CLI's save-on-exit doesn't apply).

We rejected: evaluation in the native process (splits the frontend,
contradicts ADR-0001); the webview writing files directly via a Tauri fs
plugin (store code, schema, atomicity, naming, would be duplicated in
JS instead of reused); and autosaving every definition without the
explicit `save` command (would diverge from CLI/TUI semantics).

## Consequences

- One store, four native participants (CLI, TUI, desktop, and the future
  FSA-bridged PWA): last-write-wins per document across concurrently
  running frontends, accepted (ADR-0003 already accepted this).
- The frontend gains a small bridge seam (`None` | `Tauri`) and three
  wasm-only dependencies (js-sys, wasm-bindgen-futures,
  serde-wasm-bindgen); command messages localize through the same Fluent
  catalogs as the native shells.
- `window.__TAURI__` (withGlobalTauri) is the bridge's detection signal.
- UI text beyond command messages stays English until the web-i18n wiring
  lands (unchanged, harness-blocked).

## Amendment (2026-08-27): the store also carries the shared session snapshot

The store's five settings grew one more: `setting/session.json` holds the
environment's variable bindings, user assignments and `ans`, as a JSON
map. Every interactive frontend saves it with the same cadence as history
(the CLI/REPL per line, the TUI at each submit, the desktop webview
through a new `save_session` IPC command next to `save_history`), and
every frontend restores it at startup: `load_session` applies it after
replaying functions/constants/scripts, and the webview's `init` returns
it for the same restore. One installation, one calculator state: define
`x = 5` in the TUI, close it, and `epher "x * 2"` or the desktop app
knows `x`, and `ans`, already.

The CLI one-shot (`epher "expr"`) joins the store: it evaluates against
the saved session and records the command in the shared history, so a
command entered at the shell prompt is part of the same body of saved
work as the REPL's. Piped mode (`epher -`) still only *reads* the store,
batch runs never write history or session state (unchanged behavior).

The earlier ADR-0021 statement that `ans` is "never persisted" is
superseded for the desktop installation; the browser PWA remains
session-only (this ADR's PWA column is unchanged).

## Amendment (2026-08-27): publish/subscribe: the shared store syncs live between open frontends, and the Windows store path is fixed

The store is the single source of truth, but an open frontend only saw
what it loaded at startup: history created in the TUI never appeared in
an already-running desktop app until a restart. The shared-storage
requirement is now a **publish/subscribe** contract (ADR-0010 amendment):

- **Publish.** Every frontend writes each state change to the store
  immediately as it happens: CLI/REPL per submitted line, TUI per
  submit, desktop per webview submit (history and the session snapshot
  together, plus language/theme on change). No batching, no save-on-quit.
- **Subscribe.** The long-lived frontends watch the store directory
  (epher-store's `watch` module, `notify` behind the `fs` feature) and
  refresh in place when anything changes: the TUI reloads its session
  and settings on a watcher signal (keeping the in-flight entry text),
  and the desktop shell re-reads the store in a broadcast thread and
  emits `store-changed` with the fresh `InitState`, which the webview
  applies through the same path as startup. Reloads never write, so the
  loop cannot feed itself. CLI one-shots and the REPL stay
  subscribe-at-start (they are prompt/transient interfaces; their
  per-line publish already covers the other direction).
- **Windows store path.** `default_store_dir` fell back to `.epher` in
  the current directory when `HOME` was unset, the normal case in
  cmd/PowerShell, so the desktop app and the TUI looked at different
  stores depending on where each was launched, and neither saw the
  other's history. It now resolves `USERPROFILE` on Windows
  (`%USERPROFILE%\.epher`), the same `~/.epher` every frontend uses.

## Amendment (2026-08-28): the desktop's own session saves and reloads never lose live constants

Two defects in the desktop publish path made the guide's animated
examples fail in the installed app (the 3D one: `const a = 1` then
`graph3d sin(a * (x ^ 2 + y ^ 2)) from -3 to 3` errored with "unknown
name: a", so no plot, slider, or play button appeared):

- **The session save never crossed the Linux IPC.** The webview sent
  the bindings as a JS `Map` (serde_wasm_bindgen renders `HashMap`
  that way), which the webkitgtk IPC cannot transport, and a bare
  array of pairs did not match the command's field name; the
  `save_session` command silently never ran and
  `setting/session.json` was never written. The bindings now travel
  in a struct (`SessionArgs`, field `bindings`), the same shape as
  every other save command, and the command parameter carries the
  matching name.
- **A reload could observe the pre-submit session.** The
  store-changed apply rebuilt the session from the store and replayed
  the live session's `def`/`const` sources, but it read them through
  the Yew state handle, whose deref can still hold the value from
  before a `set` made in another callback; the replay then saw no
  constants and the fresh submit's constant vanished. The live
  sources now come from an `Rc<RefCell<Session>>` live cell that the
  submit, the slider, and the apply all read and write in lockstep
  with the render state (the project's stale-deref rule), so the
  merge always sees the current session.

## Amendment (2026-11-27): the REPL subscribes live, its saves never clobber, and the desktop refuses to save what it could not read

A live audit (REPL, TUI, desktop app, one-shot, and a fresh second
REPL all open and active against one shared store; a pty harness
driving the CLI/TUI and screenshots the desktop app) found the
2026-08-27 amendment's subscribe-at-start claim for the REPL false in
both directions, and one of the two failures destroyed data:

- **The REPL's arrow walk never saw foreign writes.** rustyline's list
  was seeded once at startup and never re-read, so entries the TUI or
  desktop added while the REPL sat open were unreachable; the
  reported bug.
- **Worse, the REPL's per-line publish erased them.** `save_history`
  writes the whole list from the session's view; with no reload that
  view went stale, and a REPL line overwrote `history.json` with it;
  the TUI's entry vanished from the store, not just from a display.

The REPL is now a **live subscriber**, the TUI's pattern at a
prompt's granularity:

- A watcher pump thread flags foreign writes (the channel has one
  consumer; a flag is all a key handler can safely read while
  rustyline owns the editor). If the pump thread itself cannot start,
  the REPL says so and degrades to boundary-only reloads.
- When the store cannot be read (a corrupt or unreadable file), the
  reload fails loudly (one warning per failure episode) and the
  session in memory stays authoritative: a store the REPL cannot read
  is one no frontend can read, so the per-line saves keep going;
  overwriting such a file is repair, not loss. The desktop's save
  gate guards the real hole, a readable store the webview could not
  parse.
- The reader reloads the store's session at every line boundary (after
  a read returns, before the line acts), replaying the session's
  unsaved `def`/`const` sources over it (the TUI's merge rule) and
  rebuilds the recall list through `repl_history_lines`, so the seed
  policy of ADR-0086 applies to every reload, not just startup. A
  piped session reloads the same way, silently, so transcripts stay
  byte-identical. Reloads never write, so no loop is possible.
- rustyline cannot reload mid-read, so **Up ends a stale read**: when
  a foreign write is pending, an Up key handler (the Ctrl-L
  `ConditionalEventHandler` pattern) ends the readline; the loop
  reloads, restores the half-typed buffer into the next prompt
  (`readline_with_initial`), and the very next Up walks the fresh
  list. Nothing is evaluated and nothing typed is lost. This retires
  the 2026-08-27 sentence "subscribe-at-start is all a prompt-based
  interface can honor"; the prompt boundary is a reload point, and
  one Up press is a boundary.
- **The desktop's save gate** closes the audit's second hole: the
  shell's `init` can succeed while the webview's deserialize of the
  same state fails: a web build from an older data format warned
  "could not load saved data … starting fresh" and would then have
  written that fresh session over a store it never read. The webview
  now confirms each applied state (`confirm_init`), and until a
  confirmation arrives every `save_*` command refuses; a failed
  `init` revokes the confirmation, and each applied `store-changed`
  state re-arms it. The browser PWA has no native store, so it is
  out of the gate's scope (its persistence remains session-only).
- Known limitation (accepted): the gate re-arms only on a state the
  webview parsed. A frontend left open across a store-format change
  (an upgraded CLI writing records while an older desktop app runs)
  that fails to parse a `store-changed` payload keeps its pre-change
  view; restart the older app after upgrading any other frontend.
- Regression seams: two piped-REPL integration tests (a foreign
  one-shot write lands mid-session and survives the REPL's next line;
  a foreign binding is visible to the next line), a unit test for the
  reload/rebuild helper, shell gate unit tests, and a pty harness
  (`scripts/repl-pubsub-harness.py`) driving the real editor and the
  real TUI concurrently: the loop that found the bug, kept so the
  next audit starts from evidence instead of screenshots, and wired
  into CI (`pubsub-harness.yml`) on the pushes and PRs that touch the
  pub/sub surfaces.

The one-shot keeps its startup read: its whole run is milliseconds,
so a mid-run foreign write is not a scenario, and its publish already
carries the fresh view it loaded (unchanged).
