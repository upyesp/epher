// The console entry point of the unified `epher` binary (ADR-0011): the
// build terminal users get on PATH, one-shot evaluation, REPL, piped
// scripts, and the TUI, with real stdout/stderr, pipes, and exit codes.
// On Windows, `epher gui` hands the GUI action off to the GUI-subsystem
// sibling `epher-gui.exe` (see app_lib::launch_gui), and a bare invocation
// explains itself and ends (ADR-0074, the temporary bare-run banner). On
// macOS the wrapper thread cannot host the window (tao requires the
// process's main thread for the event loop), so the GUI hands off to the
// wrapper-free sibling the same way (ADR-0084). Either way the console
// only ever exists while a terminal mode is actually running.

fn main() {
    // Script evaluation recurses per expression node, and collection
    // searches legitimately reach hundreds of thousands of frames; the
    // default 8 MiB main-thread stack overflows first. Run the console on
    // a dedicated fat-stack thread: the reservation is virtual only,
    // pages commit as frames actually nest. The thread stays unnamed, so
    // launch_gui can tell it from the real main thread and hand the GUI
    // action to the wrapper-free sibling on macOS (ADR-0084), like
    // Windows already does.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(|| app_lib::run_with_args(std::env::args_os()))
        .expect("spawn the main thread with a fat stack");
    child.join().expect("main thread panicked");
}
