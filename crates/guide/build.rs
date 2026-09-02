//! Build script: publish the guide markdown where the web app serves it.
//!
//! `site/guide/<locale>.md` is the single source of truth (ADR-0018). The
//! content is NOT compiled into any binary; the web app fetches
//! `guide/<locale>.md` and the TUI reads the installed files on request
//! (ADR-0053). This script copies the markdown into `crates/web/public/
//! guide/` so trunk ships it as static files — the same dist the desktop
//! shell embeds — without a second copy living in git.

use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=../../site/guide");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let guide = root.join("site/guide");
    let public = root.join("crates/web/public/guide");
    fs::create_dir_all(&public).unwrap();

    for locale in LOCALES {
        let from = guide.join(format!("{locale}.md"));
        let to = public.join(format!("{locale}.md"));
        fs::copy(&from, &to)
            .unwrap_or_else(|e| panic!("missing guide source {}: {e}", from.display()));
    }
}

/// The locales the guide ships in, matching site/guide/*.md.
const LOCALES: [&str; 8] = ["ar", "de", "en", "es", "fr", "hi", "pt", "zh-CN"];
