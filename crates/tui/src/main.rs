//! epher-tui, native full-screen terminal frontend (ADR-0001).
//!
//! A thin binary wrapper: the event loop and rendering live in the library
//! ([`epher_tui::run`]) so the unified `epher` binary (crates/tauri-app) can
//! offer the TUI as `epher tui` without duplicating a line.

fn main() {
    // Script evaluation recurses per expression node, and collection
    // searches legitimately reach hundreds of thousands of frames; the
    // default 8 MiB main-thread stack overflows first. Run the frontend
    // on a dedicated fat-stack thread: the reservation is virtual only,
    // pages commit as frames actually nest.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(epher_tui::run)
        .expect("spawn the main thread with a fat stack");
    match child.join().expect("main thread panicked") {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}
