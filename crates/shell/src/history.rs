//! The history walk (ADR-0093): the policy the TUI and the web/desktop
//! entry share for recalling earlier lines with the arrow keys while the
//! entry is empty. The REPL gets the same behaviour from rustyline
//! (ADR-0086); this is the spelling for the frontends that show the
//! list, so the walk follows the list as it renders (newest on top) and
//! the list can highlight the row the entry shows.

/// Where one step of the history walk lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryStep {
    /// Nothing moved: the history is empty, or past the oldest line
    /// (Down at the end does not wrap, like a shell). The walk, if one
    /// is under way, stays where it was.
    Stay,
    /// The walk left the list (Up from the newest line): the entry goes
    /// back to empty, the way a shell's Down past its newest line
    /// returns to the fresh prompt.
    Leave,
    /// The entry shows the line at displayed row `row` (0 = newest).
    Row(usize),
}

/// Step through a history of `len` lines from `position`, the displayed
/// row currently recalled (`None`: the entry is fresh and empty). `dir`
/// is +1 for Down and -1 for Up.
///
/// The arrows follow the list on screen, newest on top: the first Down
/// from a fresh entry recalls the newest line, Down walks older, Up
/// walks newer (the rule the TUI's history focus already uses, the
/// ADR-0033 amendment), so the highlight moves the way the key points.
/// Up from the newest line leaves the list (the entry is empty again);
/// Down from the oldest stays put. The walk only starts from an empty
/// entry, and either arrow starts it (the ADR-0093 amendment): Up on a
/// fresh entry recalls the newest line too, the gesture every shell
/// teaches, instead of dying against a rule the on-screen hint
/// (`↑ ↓ history`) contradicts.
pub fn history_step(len: usize, position: Option<usize>, dir: isize) -> HistoryStep {
    if len == 0 {
        return HistoryStep::Stay;
    }
    match (position, dir) {
        // Either arrow starts the walk (the ADR-0093 amendment): the
        // entry sits above the list, so the first step in either
        // direction lands on the top row. Up is the gesture every shell
        // teaches; leaving it dead was a purity the hint contradicted.
        (None, _) => HistoryStep::Row(0),
        (Some(row), d) => {
            let next = row as isize + d;
            if next < 0 {
                HistoryStep::Leave
            } else {
                let next = (next as usize).min(len - 1);
                if next == row {
                    HistoryStep::Stay
                } else {
                    HistoryStep::Row(next)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_history_never_moves() {
        assert_eq!(history_step(0, None, 1), HistoryStep::Stay);
        assert_eq!(history_step(0, None, -1), HistoryStep::Stay);
    }

    #[test]
    fn down_recalls_the_newest_line_then_walks_older() {
        assert_eq!(history_step(3, None, 1), HistoryStep::Row(0));
        assert_eq!(history_step(3, Some(0), 1), HistoryStep::Row(1));
        assert_eq!(history_step(3, Some(1), 1), HistoryStep::Row(2));
    }

    #[test]
    fn down_at_the_oldest_line_stays() {
        assert_eq!(history_step(3, Some(2), 1), HistoryStep::Stay);
        assert_eq!(history_step(1, Some(0), 1), HistoryStep::Stay);
    }

    #[test]
    fn up_walks_back_to_newer_and_leaves_at_the_newest() {
        assert_eq!(history_step(3, Some(2), -1), HistoryStep::Row(1));
        assert_eq!(history_step(3, Some(1), -1), HistoryStep::Row(0));
        assert_eq!(history_step(3, Some(0), -1), HistoryStep::Leave);
    }

    #[test]
    fn either_arrow_starts_the_walk() {
        // The ADR-0093 amendment: Up on a fresh entry recalls the newest
        // line too, instead of dying against the spatial model.
        assert_eq!(history_step(3, None, -1), HistoryStep::Row(0));
        // From there the spatial rule governs: Up again leaves (empty
        // entry), Down walks older.
        assert_eq!(history_step(3, Some(0), -1), HistoryStep::Leave);
        assert_eq!(history_step(3, Some(0), 1), HistoryStep::Row(1));
    }
}
