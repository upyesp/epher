//! Publish/subscribe for the shared store (ADR-0010 amendment): every
//! frontend writes its state to the store immediately as changes happen,
//! and the long-lived frontends (TUI, desktop app) watch the store
//! directory so another frontend's write is reflected live — no restart,
//! no refresh action. CLI one-shots and the REPL are transient: they
//! subscribe by reading at startup and publish per line, which is all a
//! prompt-based interface can honor.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

/// Spawn a thread watching `dir` for store changes. Every change to any
/// file inside the store directory produces one `()` on the returned
/// channel; the caller decides what to reload. The directory is created
/// if missing (a fresh install has nothing yet, and notify cannot watch
/// a nonexistent path). The thread lives until the process exits.
pub fn spawn_store_watcher(dir: PathBuf) -> Receiver<()> {
    let _ = std::fs::create_dir_all(&dir);
    let (tx, rx): (Sender<()>, Receiver<()>) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("epher-store-watch".into())
        .spawn(move || {
            use notify::{RecursiveMode, Watcher};
            let Ok(mut watcher) =
                notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    if res.is_ok() {
                        let _ = tx.send(());
                    }
                })
            else {
                return;
            };
            if watcher.watch(&dir, RecursiveMode::Recursive).is_err() {
                return;
            }
            // Keep the thread alive so the watcher is not dropped; the
            // sender keeps the channel open until the process exits.
            loop {
                std::thread::sleep(Duration::from_secs(3600));
            }
        })
        .ok();
    rx
}

/// Drain a burst of store-change signals (atomic writes produce several
/// events per document: the temp file, the rename, the directory
/// update) into one "reload once" signal. Call at the head of the
/// consumer's reload; returns whether anything changed.
pub fn drain_signal(rx: &Receiver<()>) -> bool {
    match rx.try_recv() {
        Ok(()) => {
            while rx.try_recv().is_ok() {}
            true
        }
        Err(_) => false,
    }
}
