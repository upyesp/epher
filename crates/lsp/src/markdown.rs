//! The markdown projection (ADR-0088): epher code fences inside a
//! markdown document, as one virtual script.
//!
//! [`Projection::extract`] scans the client's text for fenced code
//! blocks whose info string names `epher`, concatenates their content
//! in document order into the virtual script, and keeps the byte
//! ranges that translate in both directions. The client's text stays
//! the wire truth: client positions map in with [`Projection::offset_in`]
//! (nothing outside a fence maps), and every span the analysis produces
//! maps out with [`Projection::position_out`].

use lsp_types::Position;

/// One epher fence: the byte range of its content in the virtual
/// script and the byte range of the same content in the markdown text.
/// The content bytes are identical in both texts, so the mapping
/// inside a block is pure offset arithmetic.
struct Block {
    script_start: usize,
    script_end: usize,
    md_start: usize,
    md_end: usize,
}

/// The projection of one markdown document onto its epher script.
pub struct Projection {
    /// The client's text, verbatim.
    text: String,
    /// Byte offset where each markdown line starts.
    line_starts: Vec<usize>,
    /// The fences, in document order, non-overlapping and sorted.
    blocks: Vec<Block>,
    /// The virtual script: the blocks' content, `\n`-joined.
    script: String,
    /// Byte offset where each script line starts.
    script_line_starts: Vec<usize>,
}

impl Projection {
    /// Scan a markdown document for epher fences and build the
    /// projection. A document without fences projects to the empty
    /// script.
    pub fn extract(text: &str) -> Projection {
        let line_starts = crate::analysis::line_starts(text);
        let lines: Vec<&str> = (0..line_starts.len())
            .map(|i| {
                let start = line_starts[i];
                let end = line_starts.get(i + 1).map_or(text.len(), |&next| next - 1);
                &text[start..end]
            })
            .collect();
        let mut blocks = Vec::new();
        let mut script = String::new();
        let mut i = 0usize;
        while i < lines.len() {
            if let Some((fence_char, fence_len, info)) = fence_open(lines[i]) {
                if first_word(info).eq_ignore_ascii_case("epher") {
                    // The content runs to the matching closing fence,
                    // or to end of file when the fence never closes.
                    let content_first = i + 1;
                    let mut content_end = lines.len();
                    for j in content_first..lines.len() {
                        if is_close(lines[j], fence_char, fence_len) {
                            content_end = j;
                            break;
                        }
                    }
                    if content_first < content_end {
                        let md_start = line_starts[content_first];
                        let md_end = if content_end < lines.len() {
                            // The content ends at the newline just
                            // before the closing fence line: one
                            // trailing newline stripped.
                            line_starts[content_end] - 1
                        } else {
                            text.len()
                        };
                        if md_end > md_start {
                            if !script.is_empty() {
                                script.push('\n');
                            }
                            let script_start = script.len();
                            script.push_str(&text[md_start..md_end]);
                            blocks.push(Block {
                                script_start,
                                script_end: script.len(),
                                md_start,
                                md_end,
                            });
                        }
                    }
                    i = content_end;
                    continue;
                }
            }
            i += 1;
        }
        let script_line_starts = crate::analysis::line_starts(&script);
        Projection {
            text: text.to_string(),
            line_starts,
            blocks,
            script,
            script_line_starts,
        }
    }

    /// The virtual script the analysis runs on.
    pub fn script(&self) -> &str {
        &self.script
    }

    /// Script offset <- markdown position. Positions outside any
    /// fence (or beyond the document) map to nothing.
    pub fn offset_in(&self, position: Position) -> Option<usize> {
        let md = offset_at(&self.text, &self.line_starts, position)?;
        let index = self.blocks.partition_point(|b| b.md_end < md);
        let block = self.blocks.get(index)?;
        if md < block.md_start || md > block.md_end {
            return None;
        }
        Some(block.script_start + (md - block.md_start))
    }

    /// Markdown position <- script offset. Offsets in the `\n`
    /// separators between blocks clamp to the nearer block's edge.
    pub fn position_out(&self, offset: usize) -> Position {
        let offset = offset.min(self.script.len());
        let Some(block) = self.block_at(offset) else {
            return Position {
                line: 0,
                character: 0,
            };
        };
        let within = (offset - block.script_start).min(block.md_end - block.md_start);
        position_at(&self.text, &self.line_starts, block.md_start + within)
    }

    /// Markdown line <- script line: where the run payload's
    /// per-statement lines and `answerLine` land.
    pub fn line_out(&self, script_line: u32) -> u32 {
        let offset = self
            .script_line_starts
            .get(script_line as usize)
            .copied()
            .unwrap_or(self.script.len());
        self.position_out(offset).line
    }

    /// Script span <- markdown range: the union of the blocks the
    /// range touches, clamped to block edges. `(0, 0)` when the range
    /// touches no fence.
    pub fn span_in(&self, range: lsp_types::Range) -> (usize, usize) {
        let md_start = offset_at(&self.text, &self.line_starts, range.start).unwrap_or(0);
        let md_end = offset_at(&self.text, &self.line_starts, range.end).unwrap_or(self.text.len());
        let mut lo = usize::MAX;
        let mut hi = 0usize;
        for block in &self.blocks {
            if block.md_end >= md_start && block.md_start <= md_end {
                lo = lo.min(block.script_start);
                hi = hi.max(block.script_end);
            }
        }
        if lo == usize::MAX {
            (0, 0)
        } else {
            (lo, hi)
        }
    }

    /// The block whose script span contains `offset`, or the last
    /// block when `offset` is past its end.
    fn block_at(&self, offset: usize) -> Option<&Block> {
        let index = self.blocks.partition_point(|b| b.script_end < offset);
        self.blocks.get(index)
    }
}

/// Byte offset <- LSP position over one text and its line starts.
/// The position clamps to the line's end; a line past the end of the
/// text maps to nothing.
pub(crate) fn offset_at(text: &str, line_starts: &[usize], position: Position) -> Option<usize> {
    let line = position.line as usize;
    let start = *line_starts.get(line)?;
    let rest = &text[start..];
    let mut offset = start;
    let mut units = 0u32;
    for c in rest.chars() {
        if units >= position.character || c == '\n' {
            break;
        }
        units += c.len_utf16() as u32;
        offset += c.len_utf8();
    }
    Some(offset)
}

/// LSP position <- byte offset over one text and its line starts.
pub(crate) fn position_at(text: &str, line_starts: &[usize], offset: usize) -> Position {
    let offset = offset.min(text.len());
    let line = match line_starts.binary_search(&offset) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    };
    let start = line_starts[line];
    let mut units = 0u32;
    for c in text[start..offset].chars() {
        units += c.len_utf16() as u32;
    }
    Position {
        line: line as u32,
        character: units,
    }
}

/// A span's length in UTF-16 code units, the unit LSP token lengths
/// are counted in.
pub(crate) fn utf16_len(text: &str) -> u32 {
    text.chars().map(|c| c.len_utf16() as u32).sum()
}

/// The line an opening fence starts on: the fence character, its run
/// length, and the info string. Up to three leading spaces, three or
/// more of one fence character; a backtick info string may not contain
/// a backtick (CommonMark).
fn fence_open(line: &str) -> Option<(char, usize, &str)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let stripped = &line[indent..];
    let fence_char = stripped.chars().next()?;
    if fence_char != '`' && fence_char != '~' {
        return None;
    }
    let run_len = stripped.chars().take_while(|&c| c == fence_char).count();
    if run_len < 3 {
        return None;
    }
    let info = &stripped[run_len..];
    if fence_char == '`' && info.contains('`') {
        return None;
    }
    Some((fence_char, run_len, info))
}

/// Whether a line closes a fence: same character, at least as long,
/// and nothing else on the line.
fn is_close(line: &str, fence_char: char, fence_len: usize) -> bool {
    let stripped = line.trim_start_matches(' ');
    let run_len = stripped.chars().take_while(|&c| c == fence_char).count();
    run_len >= fence_len && stripped[run_len..].trim().is_empty()
}

/// The info string's first word: the language name.
fn first_word(info: &str) -> &str {
    info.split_whitespace().next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(text: &str) -> Projection {
        Projection::extract(text)
    }

    #[test]
    fn a_simple_fence_projects_to_its_content() {
        let p = project("prose\n```epher\n1 + 1\n```\nmore prose");
        assert_eq!(p.script(), "1 + 1");
    }

    #[test]
    fn multiple_blocks_join_in_order() {
        let p = project("```epher\na = 2\n```\ntext\n```epher\na * a\n```\n");
        assert_eq!(p.script(), "a = 2\na * a");
    }

    #[test]
    fn the_language_match_is_the_first_word_case_insensitive() {
        assert_eq!(project("```epher showLines\n1\n```").script(), "1");
        assert_eq!(project("```EPHER\n1\n```").script(), "1");
        assert_eq!(project("```  epher  \n1\n```").script(), "1");
        assert_eq!(project("```ephemeral\n1\n```").script(), "");
        assert_eq!(project("```rust epher\n1\n```").script(), "");
    }

    #[test]
    fn tilde_fences_work() {
        assert_eq!(project("~~~epher\n1 + 1\n~~~").script(), "1 + 1");
    }

    #[test]
    fn an_unclosed_fence_runs_to_the_end_of_the_file() {
        let p = project("# Title\n```epher\n1 / 0\nno closing fence here");
        assert_eq!(p.script(), "1 / 0\nno closing fence here");
    }

    #[test]
    fn a_longer_fence_holds_shorter_fence_lines_as_content() {
        let text = "````epher\n```\nnot a fence line\n```\n````";
        assert_eq!(project(text).script(), "```\nnot a fence line\n```");
    }

    #[test]
    fn a_backtick_info_string_with_a_backtick_is_not_a_fence() {
        assert_eq!(project("```epher `quoted`\n1\n```").script(), "");
    }

    #[test]
    fn a_closing_fence_must_be_long_enough() {
        // The inner ``` cannot close a ```` fence; the content
        // continues to the ````.
        let text = "````epher\nx = 1\n```\nstill content\n````";
        let p = project(text);
        assert_eq!(p.script(), "x = 1\n```\nstill content");
    }

    #[test]
    fn crlf_fences_are_recognized_and_content_stays_verbatim() {
        let text = "prose\r\n```epher\r\n1 + 1\r\n```\r\n";
        assert_eq!(project(text).script(), "1 + 1\r");
    }

    #[test]
    fn other_languages_fences_are_ignored() {
        assert_eq!(project("```rust\nfn main() {}\n```").script(), "");
        assert_eq!(project("```\nbare fence\n```").script(), "");
    }

    #[test]
    fn indented_fences_up_to_three_spaces_work() {
        assert_eq!(project("   ```epher\n   1\n   ```").script(), "   1");
    }

    #[test]
    fn positions_round_trip_through_the_projection() {
        let text = "# Demo\n\n```epher\n1 / 0\n2 + 2\n```\n\ntrail";
        let p = project(text);
        // Script line 0 lives on markdown line 3; line 1 on line 4.
        assert_eq!(p.line_out(0), 3);
        assert_eq!(p.line_out(1), 4);
        // A markdown position inside the fence maps to the script.
        let inside = Position {
            line: 3,
            character: 2,
        };
        let offset = p.offset_in(inside).expect("inside the fence");
        assert_eq!(&p.script()[offset..offset + 1], "/");
        // Outside fences maps to nothing.
        assert_eq!(
            p.offset_in(Position {
                line: 0,
                character: 2,
            }),
            None
        );
        assert_eq!(
            p.offset_in(Position {
                line: 5,
                character: 0,
            }),
            None
        );
        // The script offset maps back to the markdown position.
        let back = p.position_out(offset);
        assert_eq!(back.line, 3);
        assert_eq!(back.character, 2);
    }

    #[test]
    fn span_in_covers_the_blocks_a_range_touches() {
        let text = "```epher\na = 1\n```\nmid\n```epher\nb = 2\n```";
        let p = project(text);
        let everything = p.span_in(lsp_types::Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 6,
                character: 3,
            },
        });
        assert_eq!(everything, (0, p.script().len()));
        let only_second = p.span_in(lsp_types::Range {
            start: Position {
                line: 4,
                character: 0,
            },
            end: Position {
                line: 6,
                character: 3,
            },
        });
        assert_eq!(only_second.0, 6, "the second block starts after `a = 1\\n`");
        let nowhere = p.span_in(lsp_types::Range {
            start: Position {
                line: 3,
                character: 0,
            },
            end: Position {
                line: 3,
                character: 3,
            },
        });
        assert_eq!(nowhere, (0, 0));
    }

    #[test]
    fn utf16_lengths_count_code_units_not_bytes() {
        assert_eq!(utf16_len("café"), 4);
        assert_eq!(utf16_len("𝕏 = 1"), 6);
        assert_eq!(utf16_len("abc"), 3);
    }
}
