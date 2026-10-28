//! The result pane's presentation contract (ADR-0078).
//!
//! Pure string shaping shared by every frontend that renders a script
//! transcript: which lines are setup echo, where a long line may wrap,
//! when a line is really a grid. Nothing here changes a value — copy
//! paths always receive the transcript verbatim; these helpers decide
//! how the pane *shows* what it received. The web pane and the
//! terminal pane both call these, so the two always agree.

/// Which visual tier a transcript line belongs to (ADR-0078 R1). A
/// script's `const` knobs and helper scratch echo as bare values; its
/// `print` lines carry a `label: value` colon. The tier is the
/// renderer's machinery-versus-answer signal, the same one every REPL
/// draws with a dim prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// A bare statement echo (a knob, an intermediate list): dim it.
    Setup,
    /// A labeled line (`sample mean: 81.3`): full weight.
    Labeled,
    /// A failure (`error: …`) or warning (ADR-0079): never the answer,
    /// never dim — a failed paste is the one line the user must see,
    /// and an error's own `error: detail: detail` colons must not let
    /// it pass for a labeled result.
    Error,
}

/// The line's tier. The `= ` answer voice is not part of the test.
pub fn line_kind(line: &str) -> LineKind {
    let body = line.strip_prefix("= ").unwrap_or(line);
    if body.starts_with("error: ") || body.starts_with("warning: ") {
        LineKind::Error
    } else if body.contains(": ") {
        LineKind::Labeled
    } else {
        LineKind::Setup
    }
}

/// A `… : true` / `… : false` verdict line's value (ADR-0078 R6).
/// Scripts end with checks; the pane colors them rather than leaving
/// the verdict to blend into the numbers.
pub fn verdict(line: &str) -> Option<bool> {
    let body = line.strip_prefix("= ").unwrap_or(line);
    let trimmed = body.trim_end();
    if trimmed.ends_with(" true") {
        Some(true)
    } else if trimmed.ends_with(" false") {
        Some(false)
    } else {
        None
    }
}

/// A plain decimal's shape: optional sign, digits, optional fraction.
/// Deliberately no exponent and no `inf`/`NaN` — the unit test only
/// attaches to quantities.
fn plain_number(text: &str) -> bool {
    let rest = text.strip_prefix(['-', '+']).unwrap_or(text);
    let (int, frac) = match rest.split_once('.') {
        Some((int, frac)) => (int, Some(frac)),
        None => (rest, None),
    };
    !int.is_empty()
        && int.chars().all(|c| c.is_ascii_digit())
        && frac.is_none_or(|f| !f.is_empty() && f.chars().all(|c| c.is_ascii_digit()))
}

/// A bare quantity line's trailing unit (`= 12 m`), if the line is
/// exactly `number unit` (ADR-0078 R6). Labels and lists never match:
/// the unit must be the line's only other word, one to three
/// alphabetic (or degree) characters.
pub fn terminal_unit(line: &str) -> Option<&str> {
    let body = line.strip_prefix("= ").unwrap_or(line);
    let trimmed = body.trim_end();
    let (num, unit) = trimmed.split_once(' ')?;
    if unit.is_empty() || unit.chars().count() > 3 {
        return None;
    }
    if !unit.chars().all(|c| c.is_alphabetic() || c == '°') {
        return None;
    }
    plain_number(num).then_some(unit)
}

/// A transcript as the pane displays it: the web splits its private
/// separator into answers first, then every answer may carry its own
/// line breaks (a table, a matrix); the terminal's transcript is
/// newline-joined already. Either way the pane shows one display line
/// per answer line.
pub fn display_lines(result: &str, sep: Option<char>) -> Vec<String> {
    let parts = match sep {
        Some(sep) => result.split(sep).filter(|p| !p.is_empty()).collect::<Vec<_>>(),
        None => vec![result],
    };
    parts
        .iter()
        .flat_map(|p| p.lines())
        .map(|l| l.to_string())
        .collect()
}

/// The display line that carries the transcript's answer (ADR-0078
/// R4, amended): the **last line**, unless that line is a bare verdict
/// (`…: true`), in which case the last labeled line above it.
///
/// The first version emphasized the last *labeled* line, which suited
/// the corpus's `print("label:", value)` convention but skipped the 32
/// scripts whose closing line is a sentence-result without a colon
/// (`= that is 17.4 years`, `= no seventh candidate`) — exactly the
/// lines a reader wants emphasized. Scripts are written results-last;
/// a trailing `: true` check is the one shape that is not an answer,
/// and ADR-0079 already keeps failures out. `None` (no emphasis) only
/// for an empty or entirely failed transcript.
pub fn final_labeled_index(lines: &[String]) -> Option<usize> {
    let last_content = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| line_kind(l) != LineKind::Error)
        .map(|(i, _)| i)
        .next_back()?;
    let is_verdict = verdict(&lines[last_content]).is_some();
    if !is_verdict {
        return Some(last_content);
    }
    // A trailing check: emphasize the answer it verifies.
    (0..last_content)
        .rev()
        .find(|&i| line_kind(&lines[i]) == LineKind::Labeled)
}

/// One aligned grid: a line's `[[…]]` or `{{…}}` matrix (ADR-0078
/// R3). `widths` is each column's widest cell, in characters, so a
/// monospace renderer right-aligns by padding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub rows: Vec<Vec<String>>,
    pub widths: Vec<usize>,
}

/// Parse a line's matrix into a [`Grid`], with the text before and
/// after it (a `= ` voice, a label, usually nothing). Returns `None`
/// unless the line carries a real grid — at least two rows of at
/// least two cells — so `{{1, 1}, {1, 2}, …}` pairs grid, and single
/// rows, single columns, and everything else stay text.
pub fn matrix_grid(line: &str) -> Option<(String, Grid, String)> {
    let (start, open) = ["[[", "{{"]
        .iter()
        .filter_map(|o| line.find(o).map(|i| (i, *o)))
        .min_by_key(|(i, _)| *i)?;
    let close = if open == "[[" { "]" } else { "}" };
    let (inner_open, inner_close) = (&open[0..1], &close[0..1]);

    // Find the outer pair's matching close by depth, so a grid may
    // carry text after it ("= [[…]] extra").
    let chars: Vec<char> = line.chars().collect();
    let mut depth = 0usize;
    let mut close_idx = None;
    for (i, &c) in chars.iter().enumerate().skip(start) {
        if Some(c) == open.chars().next() {
            depth += 1;
        } else if Some(c) == close.chars().next() {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                close_idx = Some(i);
                break;
            }
        }
    }
    let close_idx = close_idx?;
    // Skip ONE outer char: the second bracket of "[[" or "{{" is the
    // first row's own opening bracket.
    let mut rest: &str = &line[start + 1..close_idx];
    let suffix: String = chars[close_idx + 1..].iter().collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    loop {
        let row = rest.trim_start();
        if !row.starts_with(inner_open) {
            return None;
        }
        // One balanced inner group: cells split on top-level commas.
        let after_open = &row[1..];
        let mut depth = 1usize;
        let mut cells = String::new();
        let mut consumed = 0usize;
        for (i, c) in after_open.char_indices() {
            consumed = i + c.len_utf8();
            if Some(c) == inner_open.chars().next() {
                depth += 1;
            } else if Some(c) == inner_close.chars().next() {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            cells.push(c);
        }
        if depth != 0 || cells.is_empty() {
            return None;
        }
        let parsed: Vec<String> = cells.split(',').map(|c| c.trim().to_string()).collect();
        if parsed.len() < 2 || parsed.iter().any(|c| c.is_empty()) {
            return None;
        }
        rows.push(parsed);

        let after = row[1 + consumed..].trim_start();
        if after.is_empty() {
            break; // the outer close ended the grid
        }
        if after.starts_with(',') {
            rest = &after[1..];
        } else {
            break;
        }
    }
    if rows.len() < 2 {
        return None;
    }
    let mut widths: Vec<usize> = Vec::new();
    for row in &rows {
        for (c, cell) in row.iter().enumerate() {
            match widths.get_mut(c) {
                Some(w) => *w = (*w).max(cell.chars().count()),
                None => widths.push(cell.chars().count()),
            }
        }
    }
    let prefix = line[..start].to_string();
    Some((prefix, Grid { rows, widths }, suffix))
}

/// A line's long flat list, with the text before and after it
/// (ADR-0078 R3's elision path). Only lists beyond `threshold`
/// elements qualify — shorter ones render as text. Elements split on
/// top-level commas, so a nested `{…}` element stays whole.
pub fn long_list(line: &str, threshold: usize) -> Option<(String, Vec<String>, String)> {
    let chars: Vec<char> = line.chars().collect();
    let start = line.find('{')?;
    let mut depth = 0usize;
    let mut end = None;
    for (i, &c) in chars.iter().enumerate().skip(start) {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end?;
    if end <= start + 1 {
        return None; // `{}` — nothing to elide
    }
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for &c in &chars[start + 1..end] {
        match c {
            '{' | '[' => {
                depth += 1;
                current.push(c);
            }
            '}' | ']' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                elements.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let last = current.trim().to_string();
    if !last.is_empty() {
        elements.push(last);
    }
    if elements.len() <= threshold {
        return None;
    }
    let prefix = chars[..start].iter().collect();
    let suffix = chars[end + 1..].iter().collect();
    Some((prefix, elements, suffix))
}

/// The display line's wrap segments (ADR-0078 R2): the places a pane
/// may break a long line *besides* its spaces. A break after `", "` is
/// offered unless the comma sits between digits — `1,000,000` must
/// never split — and after a closing brace or bracket. Numbers,
/// fractions, and timestamps never contain a segment edge, so they
/// stay whole wherever these segments are honored as soft breaks.
/// (A renderer that already wraps at spaces — the web's `pre-wrap` —
/// uses the segments as extra opportunities; the terminal, which has
/// no wrap of its own, packs tokens whole via [`wrap_to_width`].)
pub fn wrap_segments(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize; // inside { } / [ ]
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
        current.push(c);
        let boundary = if c == ',' && depth > 0 && chars.get(i + 1) == Some(&' ') {
            // Not between digits: "1,234" keeps its comma glued.
            !(chars[i - 1].is_ascii_digit()
                && chars.get(i + 2).map(|n| n.is_ascii_digit()).unwrap_or(false))
        } else {
            (c == '}' || c == ']') && chars.get(i + 1) == Some(&' ')
        };
        if boundary {
            segments.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

/// The terminal's wrap (ADR-0078 R2): a line as monospace rows no
/// wider than `width`, continuation rows indented `hang` spaces.
/// Breaks fall on spaces and segment boundaries; a token wider than
/// the whole pane (a 60-digit answer) chunks so nothing clips.
pub fn wrap_to_width(line: &str, width: usize, hang: usize) -> Vec<String> {
    let width = width.max(8);
    let indent = " ".repeat(hang.min(width));
    let mut rows: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut continuation = false;
    let mut has_content = false; // current holds a token, not just indent
    for segment in wrap_segments(line) {
        for word in segment.split(' ') {
            if word.is_empty() {
                continue;
            }
            // A token fills its row whole; it chunks only when no
            // row could ever hold it (a 60-digit answer).
            let mut rest: String = word.to_string();
            while !rest.is_empty() {
                if !has_content && continuation && current.is_empty() {
                    current.push_str(&indent);
                }
                let fresh_budget = if !has_content && continuation {
                    width - indent.chars().count()
                } else {
                    width
                };
                let budget = if !has_content {
                    fresh_budget
                } else {
                    width - current.chars().count() - 1
                };
                let len = rest.chars().count();
                let take = if len <= budget {
                    // fits where we are
                    len
                } else if len <= fresh_budget {
                    // fits a fresh row whole: break first
                    0
                } else {
                    // wider than any row: chunk
                    budget
                };
                if take == 0 {
                    if has_content {
                        rows.push(std::mem::take(&mut current));
                        has_content = false;
                    }
                    current.clear();
                    continuation = true;
                    continue;
                }
                let tail: String = rest.chars().skip(take).collect();
                if has_content {
                    current.push(' ');
                }
                current.push_str(&rest.chars().take(take).collect::<String>());
                has_content = true;
                rest = tail;
                if !rest.is_empty() {
                    rows.push(std::mem::take(&mut current));
                    has_content = false;
                    continuation = true;
                }
            }
        }
    }
    if has_content {
        rows.push(current);
    }
    if rows.is_empty() {
        rows.push(String::new());
    }
    rows
}

/// A grid as padded monospace rows: each cell right-padded to its
/// column's width, columns joined with two spaces (ADR-0078 R3).
pub fn grid_rows(grid: &Grid) -> Vec<String> {
    grid.rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(c, cell)| {
                    let pad = grid
                        .widths
                        .get(c)
                        .copied()
                        .unwrap_or(0)
                        .saturating_sub(cell.chars().count());
                    format!("{}{}", " ".repeat(pad), cell)
                })
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect()
}

/// Display-side rounding (ADR-0078 R7): the same quantity at `digits`
/// significant digits with an ellipsis, or `None` when the token is
/// already that short. The full token travels beside it as the
/// hover/copy text; this function never feeds a copy path.
pub fn shorten_number(text: &str, digits: usize) -> Option<String> {
    let digits = digits.max(1);
    let (neg, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (mantissa, exponent) = match rest.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (rest, None),
    };
    let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if int.is_empty() || !int.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !frac.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut all: Vec<u8> = int
        .bytes()
        .map(|b| b - b'0')
        .chain(frac.bytes().map(|b| b - b'0'))
        .collect();
    let mut point: isize = int.len() as isize; // digits before the point
    // Normalize: strip leading zeros (they carry no significance);
    // 0.000123 shifts the point left of every kept digit.
    while all.first() == Some(&0) {
        all.remove(0);
        point -= 1;
    }
    if all.is_empty() {
        return None; // zero never shortens
    }
    if all.len() <= digits {
        return None; // already short enough
    }
    // Round half-up at `digits` significant digits.
    let mut keep: Vec<u8> = all[..digits].to_vec();
    if all[digits] >= 5 {
        let mut i = keep.len();
        loop {
            if i == 0 {
                keep.insert(0, 1);
                point += 1;
                break;
            }
            i -= 1;
            keep[i] += 1;
            if keep[i] == 10 {
                keep[i] = 0;
            } else {
                break;
            }
        }
    }
    // Rebuild with the decimal point back where it belongs.
    let mut out = String::new();
    if point <= 0 {
        out.push_str("0.");
        for _ in 0..(-point) {
            out.push('0');
        }
        for &d in &keep {
            out.push((b'0' + d) as char);
        }
    } else if point as usize >= keep.len() {
        for &d in &keep {
            out.push((b'0' + d) as char);
        }
        for _ in keep.len()..point as usize {
            out.push('0');
        }
    } else {
        for &d in &keep[..point as usize] {
            out.push((b'0' + d) as char);
        }
        out.push('.');
        for &d in &keep[point as usize..] {
            out.push((b'0' + d) as char);
        }
    }
    // Trim trailing zeros in the fraction, then a bare point.
    while out.ends_with('0') && out.contains('.') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    if neg {
        out.insert(0, '-');
    }
    if let Some(exp) = exponent {
        out.push('e');
        out.push_str(exp);
    }
    out.push('\u{2026}');
    Some(out)
}

/// Display-side digit grouping (ADR-0078 R6): integer runs of five or
/// more digits group with commas (`359247.02` → `359,247.02`). Four
/// digits stay bare — years, pages, ports — and already-grouped text
/// passes through unchanged, so this is safe to apply twice.
pub fn group_digits(text: &str) -> String {
    let mut out = String::new();
    let mut run: Vec<char> = Vec::new();
    let mut in_fraction = false; // a run straight after '.' never groups
    fn flush(run: &mut Vec<char>, out: &mut String, in_fraction: bool) {
        if run.len() >= 5 && !in_fraction {
            for (i, c) in run.iter().enumerate() {
                if i > 0 && (run.len() - i) % 3 == 0 {
                    out.push(',');
                }
                out.push(*c);
            }
        } else {
            out.extend(run.iter());
        }
        run.clear();
    }
    for c in text.chars() {
        if c.is_ascii_digit() {
            run.push(c);
        } else {
            flush(&mut run, &mut out, in_fraction);
            in_fraction = c == '.';
            out.push(c);
        }
    }
    flush(&mut run, &mut out, in_fraction);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labeled_lines_are_results_and_bare_echoes_are_setup() {
        assert_eq!(line_kind("= sample mean: 81.3"), LineKind::Labeled);
        assert_eq!(
            line_kind("= one-sample t test (t and p): t = 2.0101, p = 0.0753"),
            LineKind::Labeled
        );
        assert_eq!(line_kind("= 1000"), LineKind::Setup);
        assert_eq!(
            line_kind("= {72, 85, 91, 64, 88, 79, 83, 95, 70, 86}"),
            LineKind::Setup
        );
        assert_eq!(line_kind("= 0.01"), LineKind::Setup);
    }

    /// ADR-0079: a failure is neither dim machinery nor a labeled
    /// result — its own colons must not let it pass for one.
    #[test]
    fn error_lines_are_their_own_tier() {
        assert_eq!(
            line_kind("error: cannot define constant r: the name is already a variable"),
            LineKind::Error
        );
        assert_eq!(line_kind("error: parse error: expected ';'"), LineKind::Error);
        assert_eq!(line_kind("warning: answer is approximate"), LineKind::Error);
    }

    /// A failed paste never dresses itself up as the answer.
    #[test]
    fn an_error_line_never_takes_the_answer_emphasis() {
        let lines: Vec<String> = [
            "= area: 78.5",
            "error: cannot define constant r: the name is already a variable",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(final_labeled_index(&lines), Some(0));
        let only_error: Vec<String> = vec!["error: unknown name: pay".to_string()];
        assert_eq!(final_labeled_index(&only_error), None);
    }

    #[test]
    fn verdicts_read_the_check() {
        assert_eq!(verdict("= back in the first row: true"), Some(true));
        assert_eq!(verdict("= halving h halves the error: true"), Some(true));
        assert_eq!(verdict("= a < h holds: false"), Some(false));
        assert_eq!(verdict("= 81.3"), None);
        assert_eq!(verdict("= truly: trueness"), None);
    }

    #[test]
    fn terminal_units_attach_to_bare_quantities_only() {
        assert_eq!(terminal_unit("= 12 m"), Some("m"));
        assert_eq!(terminal_unit("= -3.5 kg"), Some("kg"));
        assert_eq!(terminal_unit("= 48.59 °"), Some("°"));
        assert_eq!(terminal_unit("= 0.01"), None);
        assert_eq!(terminal_unit("= sample size: 10"), None);
        assert_eq!(terminal_unit("= 1994 is MCMXCIV"), None);
        assert_eq!(terminal_unit("= 12 meters long"), None);
    }

    #[test]
    fn display_lines_split_answers_then_breaks() {
        let joined = "a\u{1f}b\nc";
        assert_eq!(display_lines(joined, Some('\u{1f}')), vec!["a", "b", "c"]);
        assert_eq!(display_lines("= 1\n= 2", None), vec!["= 1", "= 2"]);
    }

    /// ADR-0078 R4 (amended): results are written last, so the last
    /// line is the answer — except a trailing verdict, which is a check
    /// on the answer above it.
    #[test]
    fn the_last_line_is_the_answer_unless_it_is_a_check() {
        let with_check: Vec<String> = ["= 1000", "= payment: 88.84", "= check: true"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(final_labeled_index(&with_check), Some(1));

        // A sentence-result without a colon is still the answer.
        let sentence: Vec<String> = ["= principal: 1000", "= that is 17.4 years"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(final_labeled_index(&sentence), Some(1));

        // A list or matrix ending is the answer too.
        let data: Vec<String> = ["= {1, 2, 5, 15, 42, 255}"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(final_labeled_index(&data), Some(0));

        assert_eq!(final_labeled_index(&[]), None);
    }

    #[test]
    fn matrices_of_both_flavors_grid() {
        let (prefix, grid, suffix) =
            matrix_grid("= [[2, 3, -1, -2/3], [4, 1, 2, 3], [-2, 5, 3, -2/3]]").unwrap();
        assert_eq!(prefix, "= ");
        assert_eq!(suffix, "");
        assert_eq!(grid.rows.len(), 3);
        assert_eq!(grid.rows[0], vec!["2", "3", "-1", "-2/3"]);
        assert_eq!(grid.widths, vec![2, 1, 2, 4]);

        let (_, grid2, _) =
            matrix_grid("= {{1, 10, 78.85, 921.15}, {2, 9.21, 79.64, 841.51}}").unwrap();
        assert_eq!(grid2.rows.len(), 2);
        assert_eq!(grid2.widths, vec![1, 4, 5, 6]);
    }

    #[test]
    fn a_labeled_matrix_keeps_its_label_and_suffix() {
        let (prefix, grid, suffix) =
            matrix_grid("= solution x y z: [[1, 0, 0, 0.5], [0, 1, 0, -1/3]] extra").unwrap();
        assert_eq!(prefix, "= solution x y z: ");
        assert_eq!(grid.rows.len(), 2);
        assert_eq!(suffix, " extra");
    }

    #[test]
    fn single_row_blocks_stay_text() {
        assert!(matrix_grid("= [[1.5, 2.5, 3.5]]").is_none());
    }

    #[test]
    fn non_matrices_refuse_the_grid() {
        assert!(matrix_grid("= 81.3").is_none());
        assert!(matrix_grid("= sample mean: 81.3").is_none());
        assert!(matrix_grid("= {1, 2, 3}").is_none());
        assert!(matrix_grid("= [[unclosed").is_none());
        assert!(matrix_grid("= [[1, 2], [3, 4]").is_none()); // missing outer close
    }

    #[test]
    fn long_lists_elide_only_past_the_threshold() {
        let line = "= values: {1, 2, 3}";
        assert!(long_list(line, 20).is_none());
        let items: Vec<String> = (0..25).map(|i| i.to_string()).collect();
        let line = format!("= {{{}}}", items.join(", "));
        let (prefix, els, suffix) = long_list(&line, 20).unwrap();
        assert_eq!(prefix, "= ");
        assert_eq!(els.len(), 25);
        assert_eq!(els[24], "24");
        assert_eq!(suffix, "");
    }

    #[test]
    fn wrap_segments_guard_grouped_numbers() {
        let segs = wrap_segments("= total 1,000,000 and {a, b, c} done");
        // The grouped number must stay inside one segment.
        assert!(segs.iter().any(|s| s.contains("1,000,000")));
        // List elements offer their own boundaries.
        assert!(segs.iter().any(|s| s.ends_with("{a,")));
        assert!(segs.iter().any(|s| s.ends_with(" b,")));
        // Joining the segments rebuilds the line exactly.
        assert_eq!(segs.join(""), "= total 1,000,000 and {a, b, c} done");
    }

    #[test]
    fn wrap_segments_keep_timestamps_whole() {
        let line = "= window: 2026-02-25T19:24:06 to 2026-02-26T05:00:31";
        let joined = wrap_segments(line).join("");
        assert_eq!(joined, line);
        assert!(wrap_segments(line).iter().all(|s| !s.ends_with('-')));
    }

    #[test]
    fn wrap_to_width_keeps_numbers_whole_and_indents() {
        let line = "= scores: {0.393505701095, 0.393505701095, 0.393505701095}";
        let rows = wrap_to_width(line, 32, 2);
        for row in &rows {
            assert!(row.chars().count() <= 32, "row too wide: {row}");
        }
        // Every number appears intact somewhere.
        let joined = rows.join("\n");
        assert_eq!(joined.matches("0.393505701095").count(), 3);
        // Continuations hang under the value, not under "= ".
        for row in rows.iter().skip(1) {
            assert!(row.starts_with("  "), "no hanging indent: {row}");
        }
    }

    #[test]
    fn wrap_to_width_chunks_an_answer_wider_than_the_pane() {
        let long = format!("= {}", "3".repeat(60));
        let rows = wrap_to_width(&long, 32, 2);
        assert!(rows.len() >= 2);
        assert_eq!(rows[0].chars().count(), 32);
        assert_eq!(rows.join("").matches('3').count(), 60);
    }

    #[test]
    fn grid_rows_pad_columns_for_alignment() {
        let (_, grid, _) = matrix_grid("= [[1, 22, 333], [4444, 5, 6]]").unwrap();
        let rows = grid_rows(&grid);
        assert_eq!(rows[0], "   1  22  333");
        assert_eq!(rows[1], "4444   5    6");
    }

    #[test]
    fn shorten_rounds_to_significant_digits() {
        assert_eq!(shorten_number("359.247028874", 6).unwrap(), "359.247\u{2026}");
        assert_eq!(shorten_number("0.393505701095", 6).unwrap(), "0.393506\u{2026}");
        assert_eq!(shorten_number("-1.83990503485", 6).unwrap(), "-1.83991\u{2026}");
        assert_eq!(shorten_number("9.999999", 3).unwrap(), "10\u{2026}");
        assert_eq!(shorten_number("81.3", 6), None);
        assert_eq!(shorten_number("2026", 6), None);
        assert_eq!(shorten_number("0", 6), None);
        assert_eq!(shorten_number("0.000123456789", 3).unwrap(), "0.000123\u{2026}");
    }

    #[test]
    fn grouping_leaves_years_and_grouped_text_alone() {
        assert_eq!(group_digits("359247.028874"), "359,247.028874");
        assert_eq!(group_digits("2026"), "2026");
        assert_eq!(group_digits("1,000,000"), "1,000,000");
        assert_eq!(
            group_digits("= reaching 1,000,000 takes: 359247.02"),
            "= reaching 1,000,000 takes: 359,247.02"
        );
    }
}
