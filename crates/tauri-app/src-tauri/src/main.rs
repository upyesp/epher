// The console entry point of the unified `epher` binary (ADR-0011): the
// build terminal users get on PATH, one-shot evaluation, REPL, piped
// scripts, and the TUI, with real stdout/stderr, pipes, and exit codes.
// On Windows, a bare invocation (double-click) hands the GUI action off to
// the GUI-subsystem sibling `epher-gui.exe` (see app_lib::launch_gui), so
// the console only ever exists while a terminal mode is actually running.

fn main() {
    // Script evaluation recurses per expression node, and collection
    // searches legitimately reach hundreds of thousands of frames; the
    // default 8 MiB main-thread stack overflows first. Run the console on
    // a dedicated fat-stack thread: the reservation is virtual only,
    // pages commit as frames actually nest.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(|| app_lib::run_with_args(std::env::args_os()))
        .expect("spawn the main thread with a fat stack");
    child.join().expect("main thread panicked");
}
