//! Generate the catalog tables from `epher scripts/` at compile time.
//!
//! Walks `<workspace>/epher scripts/**/*.epher`, reads each header's
//! one-line purpose (the `name.epher -- purpose` line; the same rule
//! `scripts/build-scripts.mjs` applies for the website), and emits a
//! sorted `ENTRIES` table with the scripts' full text via
//! `include_str!`. Fails the build when the folder is missing or a
//! script breaks the header standard, so a fleet edit that used to
//! need only `check-scripts` now also has to keep its header intact.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let scripts = manifest
        .ancestors()
        .nth(2)
        .expect("crate lives at crates/<name>")
        .join("epher scripts");
    println!("cargo:rerun-if-changed={}", scripts.display());

    let mut paths: Vec<PathBuf> = walk(&scripts);
    assert!(
        !paths.is_empty(),
        "no scripts found under {}",
        scripts.display()
    );
    paths.sort();

    let mut entries = String::new();
    entries.push_str("pub static ENTRIES: &[Entry] = &[\n");
    for path in &paths {
        let rel = path
            .strip_prefix(&scripts)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let mut parts = rel.split('/');
        let field = parts.next().unwrap_or_default();
        let area = parts.next().unwrap_or_default();
        let name = parts.next().unwrap_or_default();
        let content = fs::read_to_string(path).expect("script reads as utf8");
        let title = purpose(&content, name);
        assert!(
            !title.is_empty(),
            "{}: no `name.epher -- purpose` line in the first 12 header lines",
            rel
        );
        entries.push_str(&format!(
            "    Entry {{\n        path: {rel:?},\n        field: {field:?},\n        area: {area:?},\n        title: {title:?},\n        content: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../epher scripts/\", {rel:?})),\n    }},\n"
        ));
    }
    entries.push_str("];\n");

    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("scripts_catalog.rs");
    fs::write(out, entries).expect("write the generated catalog");
}

/// Every `.epher` file under `dir`, recursively.
fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(read) = fs::read_dir(dir) else {
        return out;
    };
    for child in read.flatten() {
        let path = child.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if path.extension().is_some_and(|e| e == "epher") {
            out.push(path);
        }
    }
    out
}

/// The header's one-line purpose: the first line of the shape
/// `name.epher -- purpose` in the first 12 lines, comment markers
/// stripped (the same parse as scripts/build-scripts.mjs).
fn purpose(content: &str, name: &str) -> String {
    let prefix = format!("{name} -- ");
    for line in content.lines().take(12) {
        let cleaned = line
            .trim_start()
            .trim_start_matches("/*")
            .trim_start_matches("//")
            .trim_start();
        if let Some(rest) = cleaned.strip_prefix(&prefix) {
            return rest.trim().to_string();
        }
    }
    String::new()
}
