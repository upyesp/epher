//! The bundled example scripts, as data.
//!
//! [`build.rs`](../build.rs) walks the repository's `epher scripts/`
//! folder at compile time and generates this crate's tables, so the
//! apps ship the catalog — each script's path, home, one-line purpose,
//! and full text — without reading anything from disk at runtime. The
//! count in the welcome hints is `ENTRIES.len()`, computed at compile
//! time from the same walk: adding a script updates every number that
//! names the collection.
//!
//! The rules mirror `scripts/build-scripts.mjs` (the website's
//! catalog builder): the one-line purpose is the header's
//! `name.epher -- purpose` line, and the hierarchy is
//! `field/area/script.epher`.

/// One bundled script.
pub struct Entry {
    /// Path under `epher scripts/`, with forward slashes, e.g.
    /// `finance/loans/extra-payment-savings.epher`.
    pub path: &'static str,
    /// The top folder, e.g. `finance`.
    pub field: &'static str,
    /// The folder inside the field, e.g. `loans`.
    pub area: &'static str,
    /// The header's one-line purpose, e.g. "paying extra monthly on a
    /// loan: months saved and interest saved".
    pub title: &'static str,
    /// The script's full text.
    pub content: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/scripts_catalog.rs"));

/// How many scripts ship with the app (compile-time constant).
pub fn count() -> usize {
    ENTRIES.len()
}

/// The (field, area) groups in catalog order: first-appearance order of
/// the sorted entries, so each group's rows stay adjacent. Three area
/// names repeat across fields (coordinates, distributions, triangles),
/// so browsers group by the pair and label with [`area_label`].
pub fn groups() -> Vec<(&'static str, &'static str)> {
    let mut groups: Vec<(&'static str, &'static str)> = Vec::new();
    for entry in ENTRIES {
        let pair = (entry.field, entry.area);
        if !groups.contains(&pair) {
            groups.push(pair);
        }
    }
    groups
}

/// A group's display label: the bare area when it names one group
/// fleet-wide, else `field / area`.
pub fn area_label(field: &str, area: &str) -> String {
    let bare = ENTRIES.iter().filter(|e| e.area == area).count();
    let owned = ENTRIES
        .iter()
        .filter(|e| e.field == field && e.area == area)
        .count();
    if bare == owned {
        area.to_string()
    } else {
        format!("{field} / {area}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalog_carries_the_fleet() {
        let count = count();
        assert!(
            count > 400,
            "the catalog should hold the whole fleet, found {count}"
        );
    }

    #[test]
    fn every_entry_has_the_parts_the_browsers_render() {
        for entry in ENTRIES {
            assert!(entry.title.len() > 8, "{}: empty purpose", entry.path);
            assert!(!entry.field.is_empty() && !entry.area.is_empty());
            assert!(entry.content.contains("/*"), "{}: no header", entry.path);
        }
    }

    #[test]
    fn groups_are_adjacent_and_labelled_without_ambiguity() {
        assert!(!groups().is_empty());
        // Adjacency: a pair may only appear in one run — once a second
        // group starts, the first must not come back.
        let mut prev: Option<(&str, &str)> = None;
        let mut closed = std::collections::HashSet::new();
        for entry in ENTRIES {
            let pair = (entry.field, entry.area);
            if Some(pair) != prev {
                if let Some(old) = prev {
                    closed.insert(old);
                }
                assert!(
                    !closed.contains(&pair),
                    "group {}/{} splits the catalog order",
                    pair.0,
                    pair.1
                );
                prev = Some(pair);
            }
        }
        // An area name shared across fields must be field-qualified so
        // the two groups never read as one.
        for (field, area) in groups() {
            let shared = groups().iter().any(|(f, a)| *a == area && *f != field);
            let label = area_label(field, area);
            if shared {
                assert!(
                    label.contains('/'),
                    "{label} must be field-qualified: the name is shared"
                );
            } else {
                assert_eq!(label, area, "a unique area labels itself");
            }
        }
    }
}
