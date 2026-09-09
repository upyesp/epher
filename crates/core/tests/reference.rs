//! The language reference may not drift from the engine (site/reference.md
//! is the formal definition): every name the catalog knows and every name
//! the evaluator answers to must appear on the page. A builtin added or
//! renamed without updating the reference fails this test.
//!
//! The check is deliberately presence-only: the reference spells names
//! inside signature tables (`gcd(a, b)`), so a substring hit is proof the
//! page knows the name; describing it correctly is the reviewer's job and
//! the examples' job.

use epher_core::{builtin_constant_groups, catalog};

const REFERENCE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../site/reference.md");

/// Callable names the autocomplete catalog does not list (ADR-0042: the
/// catalog is descriptive; these still evaluate): print, the display-verb
/// neighbors, and the statistics entries that arrived with their families.
const UNCATALOGED_CALLABLES: &[&str] = &[
    "anova", "expreg", "logreg", "mod", "modpow", "powreg", "print",
    "quadreg", "randn", "ttestpaired", "variance",
];

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
    for name in UNCATALOGED_CALLABLES {
        if !page.contains(name) {
            missing.push(name);
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
