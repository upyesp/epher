//! The language reference may not drift from the engine (site/reference.md
//! is the formal definition): every name the catalog knows and every name
//! the evaluator answers to must appear on the page. A builtin added or
//! renamed without updating the reference fails this test.
//!
//! The check is deliberately presence-only: the reference spells names
//! inside signature tables (`gcd(a, b)`), so a substring hit is proof the
//! page knows the name; describing it correctly is the reviewer's job and
//! the examples' job.

use epher_core::{builtin_constant_groups, catalog, supplementary_catalog};

const REFERENCE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../site/reference.md");

#[test]
fn the_reference_names_every_builtin() {
    let page = std::fs::read_to_string(REFERENCE_PATH)
        .expect("site/reference.md exists next to the crates");

    let mut missing = Vec::new();
    for entry in catalog() {
        if !page.contains(entry.name) {
            missing.push(entry.name);
        }
    }
    for name in supplementary_catalog() {
        if !page.contains(name.name) {
            missing.push(name.name);
        }
    }
    assert!(
        missing.is_empty(),
        "site/reference.md does not name these builtins: {missing:?}"
    );
}

#[test]
fn the_reference_names_every_builtin_constant() {
    let page = std::fs::read_to_string(REFERENCE_PATH)
        .expect("site/reference.md exists next to the crates");

    let mut missing = Vec::new();
    for (name, _) in builtin_constant_groups() {
        if !page.contains(name) {
            missing.push(name);
        }
    }
    // constants the browser catalog omits but the evaluator answers to
    for name in &["m_earth", "r_earth", "m_moon", "r_moon", "r_e", "mu_n"] {
        if !page.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "site/reference.md does not name these constants: {missing:?}"
    );
}

/// Catalog docs may not drift empty (ADR-0066): hover in editors reads
/// these strings, so every entry, curated or supplementary, carries a
/// signature and a one-line description, lifted from the reference.
#[test]
fn every_catalog_entry_carries_signature_and_description() {
    for entry in catalog().iter().chain(supplementary_catalog()) {
        assert!(
            !entry.signature.is_empty(),
            "{} has no signature",
            entry.name
        );
        assert!(
            !entry.description.is_empty(),
            "{} has no description",
            entry.name
        );
        assert!(
            !entry.description.contains('\u{2014}'),
            "{} description carries an em-dash (house style forbids them)",
            entry.name
        );
    }
}

/// The catalog docs are a lift of the reference's words (ADR-0066), so
/// they may not silently leave the page either: every piece of every
/// signature and description must still be on it. Pieces, because the
/// generator joins a name's overloads with " / " and sibling meanings
/// with "; ", and applies the house-style em-dash cleaning on the way
/// in. A reference edit that changes what hover says fails here until
/// the catalog is regenerated.
#[test]
fn catalog_docs_are_word_for_word_from_the_reference() {
    let page = std::fs::read_to_string(REFERENCE_PATH)
        .expect("site/reference.md exists next to the crates");
    let normalized = page.replace(" \u{2014} ", ": ").replace('\u{2014}', "-");

    let mut left = Vec::new();
    for entry in catalog().iter().chain(supplementary_catalog()) {
        for piece in entry.signature.split(" / ") {
            if !normalized.contains(piece) {
                left.push(format!("{}: signature {piece:?}", entry.name));
            }
        }
        for piece in entry.description.split("; ") {
            if !normalized.contains(piece) {
                left.push(format!("{}: description {piece:?}", entry.name));
            }
        }
    }
    assert!(
        left.is_empty(),
        "catalog docs no longer on the reference page: {left:?}"
    );
}
