//! app_lib, the Tauri desktop shell (ADR-0001, ADR-0010).
//!
//! The native process owns the Native Store: a `DocStore<FsStore>` rooted
//! at `default_store_dir()` (`EPHER_STORE_DIR` override, `~/.epher` default),
//! the same files the CLI and TUI use. The webview bridges to it through
//! five IPC commands, all thin wrappers over epher-store's persist helpers;
//! evaluation itself stays in the webview on the wasm core.

use std::path::PathBuf;
use std::sync::Mutex;

use clap::Parser;
use epher_store::persist;
use epher_store::{DocStore, FsStore};
use serde::Serialize;
use tauri::{Emitter, Manager, State};

/// The script file the OS handed this launch (a double-clicked `.epher`
/// file: the Linux desktop entry's `gui %f`, the Windows file
/// association, `epher gui plan.epher`). The webview consumes it through
/// [`take_open_file`] once it has mounted, file-open events can arrive
/// before the page is ready, so the shell holds the path until then.
pub struct PendingOpen(pub Mutex<Option<PathBuf>>);

/// The answer to `take_open_file`: the opened file's name and contents.
#[derive(Debug, Serialize)]
pub struct OpenFile {
    pub name: String,
    pub content: String,
}

/// The desktop's native store: one instance, managed by Tauri and shared by
/// every command.
pub struct DesktopStore {
    store: DocStore<FsStore>,
}

impl DesktopStore {
    pub fn with_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            store: DocStore::new(FsStore::new(dir)),
        }
    }

    /// Everything the webview needs at startup: history, the replay lines
    /// (functions, then constants, then scripts), the language preference,
    /// and the shared session snapshot (ADR-0010 amendment).
    pub fn init(&self) -> epher_store::StoreResult<InitState> {
        Ok(InitState {
            history: persist::history(&self.store)?,
            replay: persist::replay_lines(&self.store)?,
            language: persist::load_language(&self.store)?,
            theme: persist::load_theme(&self.store)?,
            session: persist::session_bindings(&self.store)?.unwrap_or_default(),
        })
    }

    pub fn save_function(&self, name: &str, source: &str) -> epher_store::StoreResult<()> {
        persist::save_function(&self.store, name, source)
    }

    pub fn save_constant(&self, name: &str, source: &str) -> epher_store::StoreResult<()> {
        persist::save_constant(&self.store, name, source)
    }

    pub fn save_script(&self, name: &str, source: &str) -> epher_store::StoreResult<()> {
        persist::save_script(&self.store, name, source)
    }

    pub fn save_history(&self, history: &[String]) -> epher_store::StoreResult<()> {
        persist::save_history(&self.store, history)
    }

    pub fn save_session(
        &self,
        bindings: &std::collections::HashMap<String, epher_core::Value>,
    ) -> epher_store::StoreResult<()> {
        persist::save_session(&self.store, bindings)
    }

    pub fn save_language(&self, language: &str) -> epher_store::StoreResult<()> {
        persist::save_language(&self.store, language)
    }

    pub fn save_theme(&self, theme: &str) -> epher_store::StoreResult<()> {
        persist::save_theme(&self.store, theme)
    }

    pub fn save_exact(&self, exact: bool) -> epher_store::StoreResult<()> {
        persist::save_exact(&self.store, exact)
    }

    pub fn save_format(&self, format: &str) -> epher_store::StoreResult<()> {
        persist::save_format(&self.store, format)
    }

    pub fn save_separators(&self, separators: bool) -> epher_store::StoreResult<()> {
        persist::save_separators(&self.store, separators)
    }
}

/// The answer to `init`: the store's contents as plain data, so the webview
/// can rebuild its Session exactly like `load_session` does natively.
#[derive(Debug, Serialize)]
pub struct InitState {
    pub history: Vec<String>,
    pub replay: Vec<String>,
    pub language: Option<String>,
    /// The theme preference (light/dark/night), if the user set one.
    pub theme: Option<String>,
    /// The shared session snapshot (ADR-0010 amendment): bindings saved by
    /// whichever CLI/REPL/TUI/desktop frontend ran last.
    pub session: std::collections::HashMap<String, epher_core::Value>,
}

#[tauri::command]
fn init(state: State<DesktopStore>, window: tauri::WebviewWindow) -> Result<InitState, String> {
    // Windows launches hidden (ADR-0032): the frontend calls init after
    // its first mount, the shell has painted its dark first frame by
    // then, so this is the first-paint signal. Show the window here and
    // the user's first visible frame is the dark app, never a white one.
    // (The boot-fallback script calls init too, so a failed wasm boot
    // shows its dark fallback window instead of nothing.)
    #[cfg(target_os = "windows")]
    {
        let _ = window.show();
        let _ = window.set_focus();
    }
    #[cfg(not(target_os = "windows"))]
    let _ = &window;
    state.init().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_function(state: State<DesktopStore>, name: String, source: String) -> Result<(), String> {
    state
        .save_function(&name, &source)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_constant(state: State<DesktopStore>, name: String, source: String) -> Result<(), String> {
    state
        .save_constant(&name, &source)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_script(state: State<DesktopStore>, name: String, source: String) -> Result<(), String> {
    state.save_script(&name, &source).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_history(state: State<DesktopStore>, history: Vec<String>) -> Result<(), String> {
    state.save_history(&history).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_session(
    state: State<DesktopStore>,
    bindings: Vec<(String, epher_core::Value)>,
) -> Result<(), String> {
    // The webview ships the bindings inside a struct (SessionArgs), the
    // same shape as every other save command: serde_wasm_bindgen renders
    // HashMap as a JS Map, which the Linux webkitgtk IPC cannot
    // transport; the save silently never arrived, so the desktop app
    // never wrote setting/session.json.
    // The webview ships the bindings inside a struct (SessionArgs, field
    // `bindings`), the same shape as every other save command: the raw
    // HashMap rendered as a JS Map that the Linux webkitgtk IPC drops,
    // and a bare array never matched the command's field name, so the
    // desktop app never wrote setting/session.json before.
    let session: std::collections::HashMap<String, epher_core::Value> =
        bindings.into_iter().collect();
    state.save_session(&session).map_err(|e| e.to_string())
}

pub mod cli_install;
pub mod dispatch;

#[tauri::command]
fn save_language(state: State<DesktopStore>, code: String) -> Result<(), String> {
    state.save_language(&code).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_theme(state: State<DesktopStore>, name: String) -> Result<(), String> {
    state.save_theme(&name).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_exact(state: State<DesktopStore>, exact: bool) -> Result<(), String> {
    state.save_exact(exact).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_format(state: State<DesktopStore>, format: String) -> Result<(), String> {
    state.save_format(&format).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_separators(state: State<DesktopStore>, separators: bool) -> Result<(), String> {
    state.save_separators(separators).map_err(|e| e.to_string())
}

/// The script the OS handed the launch, consumed once by the webview.
/// `None` after the first ask (and in browser builds, where the command
/// does not exist).
#[tauri::command]
fn take_open_file(pending: State<PendingOpen>) -> Option<OpenFile> {
    let path = pending.0.lock().ok()?.take()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Some(OpenFile { name, content })
}

/// File → Quit (ADR-0023): close the app's last window; Tauri exits the
/// process when none remain.
#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

/// Open a URL in the system browser (the brand link: the epher mark →
/// epher.org). The webview must not navigate away from the app, the
/// calculator would be gone with no way back, so external links ride
/// the OS opener instead of the webview's own navigation.
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    open::that_detached(&url).map_err(|e| e.to_string())
}

/// Write a file at a chosen path (ADR-0024). Split from the dialog
/// command so tests can cover the write without a native dialog.
fn write_file(path: &std::path::Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// File → Save script (ADR-0024): the operating system's save
/// dialog, the user picks the directory and the file name, then the
/// file is written there. `Ok(None)` means the user cancelled: the UI
/// stays silent, as native apps do. `Ok(Some(path))` is the written
/// path, shown in the status line.
///
/// The command is **async** and the dialog runs inside
/// `spawn_blocking`: a synchronous Tauri command executes on the main
/// thread, and a modal OS dialog parked there freezes the whole
/// webview for as long as it is open, on Linux Mint the dialog could
/// end up behind the window, looking like a hard lock (ADR-0027). Off
/// the main thread the app stays live regardless of what the dialog
/// backend does.
#[tauri::command]
async fn save_file_dialog(
    app: tauri::AppHandle,
    content: String,
    default_name: String,
) -> Result<Option<String>, String> {
    let path = tauri::async_runtime::spawn_blocking(move || {
        // No extension filter: the user may rename to any extension
        // (.epher is the pre-filled default, not a restriction), and
        // rfd's Windows filter would otherwise fight typed names by
        // appending the filtered extension (ADR-0027).
        use tauri_plugin_dialog::DialogExt;
        let mut builder = app.dialog().file();
        if let Some(window) = app.get_webview_window("main") {
            builder = builder.set_parent(&window);
        }
        builder.set_file_name(&default_name).blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(path) = path else {
        return Ok(None);
    };
    let path = path
        .into_path()
        .map_err(|_| "the chosen location is not a local file".to_string())?;
    write_file(&path, &content)?;
    Ok(Some(path.display().to_string()))
}

/// Save PNG (ADR-0042): the same save dialog as [`save_file_dialog`], for
/// the rasterized plot bytes. Async + spawn_blocking keeps the dialog off
/// the main thread (ADR-0027) for the same reasons.
#[tauri::command]
async fn save_png_dialog(
    app: tauri::AppHandle,
    data: Vec<u8>,
    default_name: String,
) -> Result<Option<String>, String> {
    let path = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        let mut builder = app.dialog().file();
        if let Some(window) = app.get_webview_window("main") {
            builder = builder.set_parent(&window);
        }
        builder.set_file_name(&default_name).blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(path) = path else {
        return Ok(None);
    };
    let path = path
        .into_path()
        .map_err(|_| "the chosen location is not a local file".to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(Some(path.display().to_string()))
}

/// Can this shell install the `epher` terminal command? (macOS app bundle
/// only: see cli_install.) The webview asks at startup to decide whether
/// to show the button.
#[tauri::command]
fn cli_install_supported() -> bool {
    cfg!(target_os = "macos")
}

/// Install the `epher` command (macOS): symlink `/usr/local/bin/epher` to
/// the app bundle's executable, with an osascript administrator-privilege
/// fallback. Ok carries a Fluent key; Err carries readable instructions.
/// Async + spawn_blocking: the password prompt can be open a long while,
/// and the UI must stay responsive.
#[tauri::command]
async fn install_cli() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(cli_install::install)
        .await
        .map_err(|e| format!("join error: {e}"))?
}

/// Run the desktop GUI (the Tauri event loop). On Windows this is called
/// via [`launch_gui`] after the detach dance; on macOS/Linux it runs
/// in-process in the foreground, like any GUI binary launched from a
/// terminal.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_desktop(None);
}

/// The desktop GUI with an optional script the OS handed the launch (the
/// `.epher` file association): it lands in [`PendingOpen`] for the
/// webview to pick up.
fn run_desktop(pending_script: Option<PathBuf>) {
    // The driver stack's probing noise: launching the GUI from a terminal
    // on a machine without working 3D acceleration prints a wall of libEGL
    // and driver warnings (DRI3, DRI2, VMware svga) that reads as if epher
    // were failing, though the window opens fine on the software fallback.
    // None of it is epher's or actionable for the user, so on Linux the
    // GUI startup holds stderr on /dev/null until the window is up (first
    // focus) or the app exits, whichever comes first; anything printed
    // afterwards still reaches the terminal. A startup failure restores
    // stderr before the error prints (the guard drops when run_desktop
    // returns). Linux only: the warning wall is a Linux/Mesa shape.
    #[cfg(target_os = "linux")]
    let startup_stderr = StartupStderr::suppress();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DesktopStore::with_dir(persist::default_store_dir()))
        .manage(PendingOpen(Mutex::new(pending_script)))
        .invoke_handler(tauri::generate_handler![
            init,
            save_function,
            save_constant,
            save_script,
            save_history,
            save_session,
            save_language,
            save_theme,
            save_exact,
            save_format,
            save_separators,
            cli_install_supported,
            install_cli,
            save_file_dialog,
            save_png_dialog,
            take_open_file,
            quit,
            open_url
        ])
        .on_window_event(move |_window, _event| {
            // The driver probing is over once the window is up and
            // interactive: stderr goes back to the terminal at the first
            // focus (or at app exit, when the guard drops), so anything
            // printed afterwards still reaches the user.
            #[cfg(target_os = "linux")]
            if matches!(_event, tauri::WindowEvent::Focused(_)) {
                startup_stderr.restore();
            }
        })
        .setup(|app| {
            // Publish/subscribe for the shared store (ADR-0010
            // amendment): the desktop writes every state change to the
            // store immediately (the webview's save_* commands), and a
            // watcher thread delivers a `store-changed` event whenever
            // another frontend, the TUI, the REPL, a one-shot CLI run
            //, writes the store, so the open app refreshes live.
            // The watcher re-reads the store itself (the managed
            // DesktopStore is not Send, and this thread is long-lived);
            // FsStore is just a directory, so a second reader is
            // trivially consistent. The webview applies the same
            // InitState it consumed at startup.
            let store_dir = persist::default_store_dir();
            let store_rx = epher_store::watch::spawn_store_watcher(store_dir.clone());
            let app_handle = app.handle().clone();
            std::thread::Builder::new()
                .name("epher-store-broadcast".into())
                .spawn(move || {
                    let store = DesktopStore::with_dir(store_dir);
                    for _ in &store_rx {
                        // Collapse the burst of atomic-write events into
                        // one reload, then broadcast the fresh state.
                        while store_rx.try_recv().is_ok() {}
                        let Ok(state) = store.init() else {
                            continue;
                        };
                        let _ = app_handle.emit("store-changed", &state);
                    }
                })
                .ok();
            // Version in the title bar: every release ships an installer
            // with the same filename, and stale downloads are a recurring
            // support issue; a glance at the title settles which build is
            // running. The version lives in one place (Cargo.toml, which
            // tauri.conf.json mirrors for the bundle).
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(&format!("epher {}", env!("CARGO_PKG_VERSION")));
            }
            // Windows: the window is created hidden and shown when the
            // page has loaded (ADR-0032). Tao ignores the window
            // background color on Windows and the WebView2 default is
            // white, so a shown-at-creation window flashes white for a
            // frame or two; hidden-until-loaded shows the already-dark
            // page as the very first frame, and the corrected
            // --default-background-color=FF141416 browser argument
            // (AARRGGBB, the bare six digits v0.4.19 passed are not a
            // valid color and were ignored) darkens the webview itself.
            // The show happens in the `init` command (the frontend calls
            // it right after its first mount, the first-paint signal);
            // the boot-fallback script invokes it too.
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // macOS hands the app opened documents (Finder double-click
            // on an associated file, drag onto the dock icon) through
            // the Apple open event, launch time or later. Windows and
            // Linux deliver files as argv instead, handled in
            // [`run_with_args`]. Stage the file for the webview AND
            // broadcast: launch-time events beat the page's listener
            // (the pending slot covers that), later events need the
            // broadcast (the page is live).
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
            if let tauri::RunEvent::Opened { urls } = event {
                for url in urls {
                    if url.scheme() != "file" {
                        continue;
                    }
                    let Ok(path) = url.to_file_path() else {
                        continue;
                    };
                    if let Ok(mut slot) = app.state::<PendingOpen>().0.lock() {
                        *slot = Some(path.clone());
                    }
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let _ = app.emit("open-script", content);
                    }
                }
            }
            #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
            let _ = (app, event);
        });
}

/// The unified-binary entry point (ADR-0011): parse arguments with
/// [`dispatch`], then run the chosen frontend; every mode is a thin call
/// into the frontend's own library entry point, so behavior is defined
/// once (CLI/REPL/stdin: epher-cli; TUI: epher-tui; GUI: this crate).
/// Errors print to stderr (red on a terminal) and exit 1; usage errors
/// exit 2 through clap (ADR-0013).
pub fn run_with_args<I>(args: I)
where
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString> + Clone,
{
    let parsed = dispatch::Args::try_parse_from(args).unwrap_or_else(|e| e.exit());
    // Windows, console build, nothing passed (ADR-0074): the bare run
    // explains itself in the console and ends after one keypress. The
    // GUI-subsystem build (epher-gui.exe, the double-click target) and
    // every other platform keep the bare-run GUI. Temporary until
    // Microsoft grants the HeadlessAppBypass waiver; removal rides the
    // waiver's follow-up.
    if cfg!(windows) && dispatch::is_bare_run(&parsed) && !is_gui_subsystem_binary() {
        explain_bare_windows_run();
    }
    let result = match dispatch::action_from(&parsed) {
        dispatch::Action::OneShot(expr) => epher_cli::run_one_shot(&expr),
        dispatch::Action::Stdin => epher_cli::run_stdin_and_exit(),
        dispatch::Action::ScriptFile(path) => {
            // The association's target build has no console: route the
            // script to the window instead of invisible terminal output.
            if is_gui_subsystem_binary() {
                launch_gui(Some(path));
                return;
            }
            epher_cli::run_script_file(&path).and_then(|failed| {
                if failed {
                    std::process::exit(1);
                }
                Ok(())
            })
        }
        dispatch::Action::MissingScriptFile(path) => {
            epher_cli::term::error(&format!("error: no such script file: {path}"));
            std::process::exit(1);
        }
        dispatch::Action::Repl => epher_cli::run_repl(),
        dispatch::Action::Tui => {
            epher_tui::run().map_err(|e| epher_core::EpherError::Io(e.to_string()))
        }
        dispatch::Action::Gui(file) => {
            launch_gui(file);
            return;
        }
        dispatch::Action::HelpManual => std::process::exit(epher_cli::help::manual()),
        dispatch::Action::HelpTopic(topic) => epher_cli::help::topic(&topic),
    };
    if let Err(e) = result {
        epher_cli::term::error(&format!("error: {e}"));
        std::process::exit(1);
    }
}

/// The GUI startup's stderr hold (Linux only): [`Self::suppress`] points
/// fd 2 at /dev/null and keeps the original on the side;
/// [`Self::restore`] puts it back. Restore runs once (the flag wins the
/// race between the first focus event and the drop at app exit), and a
/// suppress that could not redirect leaves nothing to restore, so a
/// hostile or unusual environment degrades to today's behavior: the
/// driver noise prints as it always did.
#[cfg(target_os = "linux")]
struct StartupStderr {
    saved: Option<i32>,
    restored: std::sync::atomic::AtomicBool,
}

#[cfg(target_os = "linux")]
impl StartupStderr {
    fn suppress() -> Self {
        use std::os::unix::io::AsRawFd;
        let saved = unsafe { libc::dup(2) };
        if saved < 0 {
            return Self {
                saved: None,
                restored: std::sync::atomic::AtomicBool::new(true),
            };
        }
        let redirected = std::fs::File::create("/dev/null").ok()
            .is_some_and(|devnull| unsafe { libc::dup2(devnull.as_raw_fd(), 2) != -1 });
        if !redirected {
            unsafe { libc::close(saved) };
            return Self {
                saved: None,
                restored: std::sync::atomic::AtomicBool::new(true),
            };
        }
        Self {
            saved: Some(saved),
            restored: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn restore(&self) {
        use std::sync::atomic::Ordering;
        if self.restored.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Some(fd) = self.saved {
            unsafe {
                libc::dup2(fd, 2);
                libc::close(fd);
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for StartupStderr {
    fn drop(&mut self) {
        self.restore();
    }
}

/// Launch the desktop GUI.
///
/// The console `epher` binary is a *console* application (so `epher "2 + 2"`
/// can print and pipe from CMD/PowerShell). On Windows the GUI therefore
/// runs in the GUI-subsystem sibling `epher-gui.exe` (ADR-0011): the
/// console process spawns it detached, no console window, ever, and
/// exits immediately, so a double-click never lingers on a terminal and a
/// terminal prompt returns right away while the window appears. The
/// GUI-subsystem build itself (`epher-gui.exe`, the double-click target)
/// and the env-marked child have no console to shed, so they run the
/// window in-process. The spawn prefers the sibling `epher-gui.exe` (same
/// directory, then one level up); if none exists it falls back to
/// re-spawning itself with `EPHER_GUI_CHILD` set, the guard (and the
/// `DETACHED_PROCESS` child having no console to begin with) stops the
/// chain after one hop. On macOS/Linux the GUI runs in-process in the
/// foreground, like any GUI binary run from a terminal.
fn launch_gui(pending: Option<PathBuf>) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        let exe = std::env::current_exe().unwrap_or_default();
        let is_gui_build = exe_is_gui_build(&exe);
        if !is_gui_build && std::env::var_os("EPHER_GUI_CHILD").is_none() {
            for candidate in gui_launch_candidates(&exe) {
                // The script path (when the association or `epher gui
                // plan.epher` handed one over) rides argv: the GUI build
                // treats a script argument as "stage it in the entry".
                let spawned = std::process::Command::new(&candidate)
                    .args(pending.iter())
                    .env("EPHER_GUI_CHILD", "1")
                    .creation_flags(DETACHED_PROCESS)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
                if spawned.is_ok() {
                    std::process::exit(0);
                }
                // Spawn failed: try the next candidate, then run in-process.
            }
        }
    }
    #[cfg(not(windows))]
    let _ = &pending;
    run_desktop(pending);
}

/// Is this process the GUI-subsystem Windows build (`epher-gui.exe`, the
/// file association's target, ADR-0011)? That binary exists for
/// double-clicks and has no console, so a script argument can only mean
/// "open it in the window", never "print answers somewhere invisible".
fn is_gui_subsystem_binary() -> bool {
    std::env::current_exe()
        .map(|exe| exe_is_gui_build(&exe))
        .unwrap_or(false)
}

/// The name half of [`is_gui_subsystem_binary`], pure so it is testable
/// on any host.
fn exe_is_gui_build(exe: &std::path::Path) -> bool {
    exe.file_stem().is_some_and(|stem| stem == "epher-gui")
}

/// The Windows GUI-spawn candidates for the console binary at `current_exe`:
/// the sibling GUI-subsystem build first (same directory, then the parent
/// directory), then `current_exe` itself as the pre-W2 fallback. Pure path
/// logic so it is testable on any host.
#[cfg_attr(not(windows), allow(dead_code))]
fn gui_launch_candidates(current_exe: &std::path::Path) -> Vec<std::path::PathBuf> {
    let Some(dir) = current_exe.parent() else {
        return vec![current_exe.to_path_buf()];
    };
    let mut candidates = vec![dir.join("epher-gui.exe")];
    if let Some(parent) = dir.parent() {
        candidates.push(parent.join("epher-gui.exe"));
    }
    candidates.push(current_exe.to_path_buf());
    candidates
}

/// What the Windows console build prints for a bare run (ADR-0074), the
/// product wording: examples first, then the keypress line, so a
/// double-clicked console window stays open long enough to read.
#[cfg_attr(not(windows), allow(dead_code))]
const BARE_RUN_BANNER: &str = r#"It looks like you have run the terminal command `epher` without including any arguments. Here are some examples of how to use epher in the terminal:

epher "2 + 3"            # simple addition
epher "2 + 3 * 4"        # operator precedence
epher "solve x^2 == 9"   # solve an equation
epher repl               # start the interactive REPL in the terminal
epher tui                # start the full-screen terminal interface
epher gui                # start the desktop app
epher help               # print help

epher is about to quit, press any key to continue..."#;

/// The bare Windows run (ADR-0074): print the examples, wait for one
/// key, end with 0. When stdout is not a console (a Start tile, a
/// pipe) the printout has no reader, so the run ends quietly, as it
/// did before this banner existed.
#[cfg(windows)]
fn explain_bare_windows_run() -> ! {
    use std::io::Write;
    if !stdout_is_a_console() {
        std::process::exit(0);
    }
    let mut out = std::io::stdout();
    let _ = out.write_all(BARE_RUN_BANNER.as_bytes());
    let _ = out.write_all(b"\n\n");
    let _ = out.flush();
    wait_for_any_key();
    std::process::exit(0);
}

/// The non-Windows body never runs: the call site is gated on
/// `cfg!(windows)`, and this fn exists only so the whole call site
/// type-checks on every host (the console calls stay Windows-only).
#[cfg(not(windows))]
fn explain_bare_windows_run() -> ! {
    unreachable!("the bare-run explanation only runs on the Windows console build")
}

/// Does stdout belong to a console a user can read (not a pipe, a
/// file, or a console-less launch)?
#[cfg(windows)]
fn stdout_is_a_console() -> bool {
    use windows_sys::Win32::System::Console::{GetConsoleMode, GetStdHandle, STD_OUTPUT_HANDLE};
    unsafe {
        let mut mode = 0u32;
        GetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), &mut mode) != 0
    }
}

/// Block until one key: line input and echo are switched off for the
/// wait, so any key works and none is printed back (Enter is not
/// required). The switch is undone before the process ends: the console
/// mode belongs to the console, not to this process, so every later
/// program in the same terminal inherits it, and a no-echo raw console
/// makes the next `epher repl` look dead (typed letters never appear).
/// A redirected or absent stdin has no console mode to change, and its
/// reads end immediately, so the wait cannot hang.
#[cfg(windows)]
fn wait_for_any_key() {
    use std::io::Read;
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT,
        STD_INPUT_HANDLE,
    };
    let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    let mut mode = 0u32;
    let mut switched = false;
    unsafe {
        if GetConsoleMode(input, &mut mode) != 0 {
            switched =
                SetConsoleMode(input, mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT)) != 0;
        }
    }
    let mut byte = [0u8; 1];
    let _ = std::io::stdin().read(&mut byte);
    if switched {
        unsafe {
            SetConsoleMode(input, mode);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epher_store::persist::load_session;

    #[test]
    fn write_file_puts_the_content_at_the_chosen_path() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("chosen-name.epher");
        write_file(&target, "2 + 3  = 5\n").unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "2 + 3  = 5\n");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn startup_stderr_hold_swallows_during_and_returns_after() {
        use std::io::{Read, Write};
        use std::os::unix::io::AsRawFd;
        let (mut probe_rx, probe_tx) = std::os::unix::net::UnixStream::pair().unwrap();
        let real_stderr = unsafe { libc::dup(2) };
        // fd 2 now points at the probe stream: everything written to
        // stderr lands in probe_rx, so the test sees what a terminal
        // would see.
        assert_ne!(unsafe { libc::dup2(probe_tx.as_raw_fd(), 2) }, -1);
        writeln!(std::io::stderr(), "epher-probe before").unwrap();
        let guard = StartupStderr::suppress();
        writeln!(std::io::stderr(), "epher-probe during").unwrap();
        guard.restore();
        writeln!(std::io::stderr(), "epher-probe after").unwrap();
        // put the harness stderr back, then close the probe so the read
        // below sees end of stream
        assert_ne!(unsafe { libc::dup2(real_stderr, 2) }, -1);
        unsafe { libc::close(real_stderr) };
        drop(probe_tx);
        let mut seen = String::new();
        probe_rx.read_to_string(&mut seen).unwrap();
        assert!(seen.contains("epher-probe before"), "seen: {seen}");
        assert!(!seen.contains("epher-probe during"), "seen: {seen}");
        assert!(seen.contains("epher-probe after"), "seen: {seen}");
    }

    #[test]
    fn init_reports_what_the_cli_would_load() {
        let dir = tempfile::tempdir().unwrap();
        let desktop = DesktopStore::with_dir(dir.path());
        desktop
            .save_function(
                "fib",
                "def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)",
            )
            .unwrap();
        desktop.save_constant("k", "const k = 41").unwrap();
        desktop
            .save_script("count", "x = 0; while x < 5 do x = x + 1; x")
            .unwrap();
        desktop.save_history(&["2 + 3  = 5".to_string()]).unwrap();
        desktop.save_language("fr").unwrap();

        let state = desktop.init().unwrap();
        assert_eq!(state.history, vec!["2 + 3  = 5".to_string()]);
        assert_eq!(state.language, Some("fr".to_string()));
        assert_eq!(state.replay.len(), 3);
        assert!(state.replay[0].starts_with("def fib"));
        assert_eq!(state.replay[1], "const k = 41");
        assert!(state.replay[2].starts_with("x = 0"));
    }

    #[test]
    fn the_cli_loads_what_the_desktop_saved() {
        // The whole point (ADR-0010): the same files. The CLI's own startup
        // path must see the desktop's writes, function *and* variables set
        // by a saved script.
        let dir = tempfile::tempdir().unwrap();
        let desktop = DesktopStore::with_dir(dir.path());
        // the function body uses the constant: proves constants replay and
        // are visible inside functions (ADR-0012)
        desktop.save_function("f", "def f(x) = x ^ 2 + c").unwrap();
        desktop.save_constant("c", "const c = 5").unwrap();
        desktop.save_script("vars", "y = 7").unwrap();

        let mut session = load_session(&DocStore::new(FsStore::new(dir.path()))).unwrap();
        assert!(session.def_sources().contains_key("f"));
        assert_eq!(session.submit("f(3) + y"), "= 21");
    }

    #[test]
    fn init_on_an_empty_store_is_empty_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let state = DesktopStore::with_dir(dir.path()).init().unwrap();
        assert!(state.history.is_empty());
        assert!(state.replay.is_empty());
        assert_eq!(state.language, None);
    }

    // --- GUI hand-off candidates (ADR-0011, W2) -------------------------

    #[test]
    fn console_binary_prefers_the_gui_sibling_then_parent_then_itself() {
        use std::path::{Path, PathBuf};
        // forward slashes: valid path separators on every host
        let exe = Path::new("C:/Program Files/epher/epher.exe");
        let candidates = gui_launch_candidates(exe);
        assert_eq!(
            candidates,
            vec![
                PathBuf::from("C:/Program Files/epher/epher-gui.exe"),
                PathBuf::from("C:/Program Files/epher-gui.exe"),
                PathBuf::from("C:/Program Files/epher/epher.exe"),
            ]
        );
    }

    #[test]
    fn a_path_without_a_parent_yields_the_sibling_and_itself() {
        use std::path::{Path, PathBuf};
        let exe = Path::new("epher.exe");
        assert_eq!(
            gui_launch_candidates(exe),
            vec![PathBuf::from("epher-gui.exe"), PathBuf::from("epher.exe")]
        );
    }

    #[test]
    fn the_gui_build_never_dances_it_runs_in_process() {
        // launch_gui short-circuits for the GUI-subsystem build: it has no
        // console to shed. The file-stem check is what this asserts.
        use std::path::Path;
        let exe = Path::new("C:/Program Files/epher/epher-gui.exe");
        assert_eq!(exe.file_stem().and_then(|s| s.to_str()), Some("epher-gui"));
    }

    #[test]
    fn the_association_target_build_is_recognized_by_name() {
        use std::path::Path;
        assert!(exe_is_gui_build(Path::new("C:/Program Files/epher/epher-gui.exe")));
        assert!(exe_is_gui_build(Path::new("/usr/bin/epher-gui")));
        // the console unified binary and anything else are terminal
        // programs: their script arguments stay terminal output
        assert!(!exe_is_gui_build(Path::new("/usr/bin/epher")));
        assert!(!exe_is_gui_build(Path::new("C:/Program Files/epher/epher.exe")));
    }
}
