use std::fs;
use std::path::{Path, PathBuf};

use crate::{Storage, StoreError, StoreResult};

/// Filesystem-backed [`Storage`] for native frontends (CLI/TUI/desktop) — one
/// human-readable JSON file per key under `dir` (ADR-0002). Writes are atomic
/// (temp file + rename), so concurrent writers don't tear documents.
#[derive(Debug, Clone)]
pub struct FsStore {
    dir: PathBuf,
}

impl FsStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn path_for(&self, key: &str) -> StoreResult<PathBuf> {
        // keys are "kind/name" — map to a relative path with sanitized segments
        let mut path = self.dir.clone();
        for segment in key.split('/') {
            let sanitized: String = segment
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
                return Err(StoreError::Storage(format!(
                    "invalid key segment: {segment:?}"
                )));
            }
            path.push(sanitized);
        }
        Ok(path.with_extension("json"))
    }
}

impl Storage for FsStore {
    fn get(&self, key: &str) -> StoreResult<Option<Vec<u8>>> {
        let path = self.path_for(key)?;
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StoreError::Storage(e.to_string())),
        }
    }

    fn put(&self, key: &str, value: &[u8]) -> StoreResult<()> {
        let path = self.path_for(key)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| StoreError::Storage(e.to_string()))?;
        }
        // atomic write: temp file, then rename (last-write-wins)
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, value).map_err(|e| StoreError::Storage(e.to_string()))?;
        fs::rename(&tmp, &path).map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn list(&self, prefix: &str) -> StoreResult<Vec<String>> {
        let mut out = Vec::new();
        // documents are one kind-directory deep: dir/<kind>/<name>.json
        if let Ok(kinds) = fs::read_dir(&self.dir) {
            for kind_entry in kinds.flatten() {
                let kind_dir = kind_entry.path();
                if kind_dir.is_dir() {
                    let kind = kind_dir
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if let Ok(files) = fs::read_dir(&kind_dir) {
                        for file in files.flatten() {
                            let fp = file.path();
                            if fp.extension().map(|e| e == "json").unwrap_or(false) {
                                if let Some(stem) = fp.file_stem() {
                                    let key = format!("{kind}/{}", stem.to_string_lossy());
                                    if key.starts_with(prefix) {
                                        out.push(key);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        out.sort();
        Ok(out)
    }

    fn remove(&self, key: &str) -> StoreResult<()> {
        let path = self.path_for(key)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StoreError::Storage(e.to_string())),
        }
    }
}

#[allow(dead_code)]
fn _assert_path(_: &Path) {}
