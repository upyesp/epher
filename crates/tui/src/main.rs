//! epher-tui — native full-screen terminal frontend (ADR-0001).
//!
//! A thin binary wrapper: the event loop and rendering live in the library
//! ([`epher_tui::run`]) so the unified `epher` binary (crates/tauri-app) can
//! offer the TUI as `epher tui` without duplicating a line.

fn main() -> std::io::Result<()> {
    epher_tui::run()
}
