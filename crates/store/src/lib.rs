//! epher-store — the Storage capability and persisted schema (ADR-0002,
//! ADR-0003). One logical schema as JSON documents; physical backends differ
//! per target — native filesystem (CLI/TUI/desktop), browser storage
//! (web/PWA), and the File System Access bridge for the desktop PWA.
//!
//! Writes are atomic and last-write-wins across co-running frontends.

mod docs;
#[cfg(feature = "fs")]
mod fs;
mod memory;
pub mod persist;
#[cfg(feature = "fs")]
pub mod watch;

pub use docs::{ConstantDoc, Doc, DocStore, FunctionDoc, ScriptDoc, SettingDoc};
#[cfg(feature = "fs")]
pub use fs::FsStore;
pub use memory::MemoryStore;

/// A Store operation result.
pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("serialization error: {0}")]
    Serialize(String),
}

/// The Storage capability: raw key-value bytes. Backends: native filesystem
/// (CLI/TUI/desktop), OPFS/IndexedDB (web/PWA), FSA bridge (desktop PWA).
pub trait Storage {
    fn get(&self, key: &str) -> StoreResult<Option<Vec<u8>>>;
    fn put(&self, key: &str, value: &[u8]) -> StoreResult<()>;
    fn list(&self, prefix: &str) -> StoreResult<Vec<String>>;
    fn remove(&self, key: &str) -> StoreResult<()>;
}
