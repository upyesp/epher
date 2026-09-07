// epher-gui — the Windows double-click entry point (ADR-0011).
//
// A GUI-subsystem build of the same program as the console `epher` binary:
// `windows_subsystem = "windows"` means Windows never creates a console
// window for this process, so desktop launches (double-click, Start Menu,
// desktop shortcut) open the window with zero terminal flash. Terminal
// users keep the console `epher.exe` on PATH for CLI/TUI output, waiting,
// pipes, and exit codes.
//
// Bundled only on Windows (tauri.windows.conf.json sets mainBinaryName);
// macOS and Linux keep the single unified binary — their OSes need no
// subsystem split (an .app bundle and a Terminal=false desktop entry do
// the same job). Debug builds stay console apps for dev output.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run_with_args(std::env::args_os());
}
