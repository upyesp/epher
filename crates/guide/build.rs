//! Embed the user guide (ADR-0018): the single source of truth is
//! `site/guide/<locale>.md`, which the website build (`build-guide.mjs`)
//! turns into the guide pages. This build script copies the same files
//! into OUT_DIR and generates a locale lookup, so the web app and the TUI
//! carry byte-identical content without a network fetch.

use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=../../site/guide");
    let guide = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site/guide");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("guide");
    fs::create_dir_all(&out).unwrap();

    let locales = ["ar", "de", "en", "es", "fr", "hi", "pt", "zh-CN"];
    let mut src = String::from("/// The guide in the requested locale; English when a locale has no\n/// translation (the website falls back the same way).\npub fn guide(locale: &str) -> &'static str {\n    match locale {\n");
    for l in locales {
        let from = guide.join(format!("{l}.md"));
        let to = out.join(format!("{l}.md"));
        fs::copy(&from, &to)
            .unwrap_or_else(|e| panic!("missing guide source {}: {e}", from.display()));
        src.push_str(&format!(
            "        \"{l}\" => include_str!(\"guide/{l}.md\"),\n"
        ));
    }
    src.push_str("        _ => include_str!(\"guide/en.md\"),\n    }\n}\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("content.rs"),
        src,
    )
    .unwrap();
}
