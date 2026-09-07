// The console entry point of the unified `epher` binary (ADR-0011): the
// build terminal users get on PATH — one-shot evaluation, REPL, piped
// scripts, and the TUI, with real stdout/stderr, pipes, and exit codes.
// On Windows, a bare invocation (double-click) hands the GUI action off to
// the GUI-subsystem sibling `epher-gui.exe` (see app_lib::launch_gui), so
// the console only ever exists while a terminal mode is actually running.

fn main() {
    app_lib::run_with_args(std::env::args_os());
}
