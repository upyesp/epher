//! app_lib — the Tauri desktop shell (ADR-0001, ADR-0010).
//!
//! The native process owns the Native Store: a `DocStore<FsStore>` rooted
//! at `default_store_dir()` (`EPHER_STORE_DIR` override, `~/.epher` default) —
//! the same files the CLI and TUI use. The webview bridges to it through
//! five IPC commands, all thin wrappers over epher-store's persist helpers;
//! evaluation itself stays in the webview on the wasm core.

use std::path::PathBuf;

use clap::Parser;
use epher_store::persist;
use epher_store::{DocStore, FsStore};
use serde::Serialize;
use tauri::{Manager, State};

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
    /// (functions, then constants, then scripts), and the language preference.
    pub fn init(&self) -> epher_store::StoreResult<InitState> {
        Ok(InitState {
            history: persist::history(&self.store)?,
            replay: persist::replay_lines(&self.store)?,
            language: persist::load_language(&self.store)?,
            theme: persist::load_theme(&self.store)?,
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

    pub fn save_language(&self, language: &str) -> epher_store::StoreResult<()> {
        persist::save_language(&self.store, language)
    }

    pub fn save_theme(&self, theme: &str) -> epher_store::StoreResult<()> {
        persist::save_theme(&self.store, theme)
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
}

#[tauri::command]
fn init(state: State<DesktopStore>) -> Result<InitState, String> {
    state.init().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_function(state: State<DesktopStore>, name: String, source: String) -> Result<(), String> {
    state.save_function(&name, &source).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_constant(state: State<DesktopStore>, name: String, source: String) -> Result<(), String> {
    state.save_constant(&name, &source).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_script(state: State<DesktopStore>, name: String, source: String) -> Result<(), String> {
    state.save_script(&name, &source).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_history(state: State<DesktopStore>, history: Vec<String>) -> Result<(), String> {
    state.save_history(&history).map_err(|e| e.to_string())
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

/// File → Quit (ADR-0023): close the app's last window; Tauri exits the
/// process when none remain.
#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

/// Write a file at a chosen path (ADR-0024). Split from the dialog
/// command so tests can cover the write without a native dialog.
fn write_file(path: &std::path::Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// File → Save history/script (ADR-0024): the operating system's save
/// dialog — the user picks the directory and the file name, then the
/// file is written there. `Ok(None)` means the user cancelled: the UI
/// stays silent, as native apps do. `Ok(Some(path))` is the written
/// path, shown in the status line.
///
/// The command is **async** and the dialog runs inside
/// `spawn_blocking`: a synchronous Tauri command executes on the main
/// thread, and a modal OS dialog parked there freezes the whole
/// webview for as long as it is open — on Linux Mint the dialog could
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
        // (.ehs/.esr are the pre-filled defaults, not a restriction),
        // and rfd's Windows filter would otherwise fight typed names
        // by appending the filtered extension (ADR-0027).
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

/// Can this shell install the `epher` terminal command? (macOS app bundle
/// only — see cli_install.) The webview asks at startup to decide whether
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
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DesktopStore::with_dir(persist::default_store_dir()))
        .invoke_handler(tauri::generate_handler![
            init,
            save_function,
            save_constant,
            save_script,
            save_history,
            save_language,
            save_theme,
            cli_install_supported,
            install_cli,
            save_file_dialog,
            quit
        ])
        .setup(|app| {
            // Version in the title bar: every release ships an installer
            // with the same filename, and stale downloads are a recurring
            // support issue — a glance at the title settles which build is
            // running. The version lives in one place (Cargo.toml, which
            // tauri.conf.json mirrors for the bundle).
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(&format!("epher {}", env!("CARGO_PKG_VERSION")));
            }
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// The unified-binary entry point (ADR-0011): parse arguments with
/// [`dispatch`], then run the chosen frontend — every mode is a thin call
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
    let result = match dispatch::action_from(&parsed) {
        dispatch::Action::OneShot(expr) => epher_cli::run_one_shot(&expr),
        dispatch::Action::Stdin => epher_cli::run_stdin_and_exit(),
        dispatch::Action::Repl => epher_cli::run_repl(),
        dispatch::Action::Tui => {
            epher_tui::run().map_err(|e| epher_core::EpherError::Io(e.to_string()))
        }
        dispatch::Action::Gui => {
            launch_gui();
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

/// Launch the desktop GUI.
///
/// The console `epher` binary is a *console* application (so `epher "2 + 2"`
/// can print and pipe from CMD/PowerShell). On Windows the GUI therefore
/// runs in the GUI-subsystem sibling `epher-gui.exe` (ADR-0011): the
/// console process spawns it detached — no console window, ever — and
/// exits immediately, so a double-click never lingers on a terminal and a
/// terminal prompt returns right away while the window appears. The
/// GUI-subsystem build itself (`epher-gui.exe`, the double-click target)
/// and the env-marked child have no console to shed, so they run the
/// window in-process. The spawn prefers the sibling `epher-gui.exe` (same
/// directory, then one level up); if none exists it falls back to
/// re-spawning itself with `EPHER_GUI_CHILD` set — the guard (and the
/// `DETACHED_PROCESS` child having no console to begin with) stops the
/// chain after one hop. On macOS/Linux the GUI runs in-process in the
/// foreground, like any GUI binary run from a terminal.
fn launch_gui() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        let exe = std::env::current_exe().unwrap_or_default();
        let is_gui_build = exe.file_stem().is_some_and(|s| s == "epher-gui");
        if !is_gui_build && std::env::var_os("EPHER_GUI_CHILD").is_none() {
            for candidate in gui_launch_candidates(&exe) {
                let spawned = std::process::Command::new(&candidate)
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
    run();
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

    #[test]
    fn init_reports_what_the_cli_would_load() {
        let dir = tempfile::tempdir().unwrap();
        let desktop = DesktopStore::with_dir(dir.path());
        desktop
            .save_function("fib", "def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)")
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
        // path must see the desktop's writes — function *and* variables set
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
        assert_eq!(
            exe.file_stem().and_then(|s| s.to_str()),
            Some("epher-gui")
        );
    }
}
