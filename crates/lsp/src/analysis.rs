//! The document model: what the server knows about one open file, and
//! the analysis pass that keeps it current (ADR-0066).
//!
//! Every change re-runs the whole file on a fresh [`Session`] through
//! [`evaluation_trace`]: evaluation is deterministic and step-bounded,
//! so the simplest correct pass is also the fast one. The pass yields
//! everything the editor features need at once: ranged diagnostics,
//! inline-result hints, classified tokens for coloring and hover, and
//! the session state for completion.

use std::collections::HashMap;

use epher_core::{
    catalog, evaluation_trace, format_value, supplementary_catalog, token_classes, CatalogKind,
    DisplayPrefs, Session, Span, SpannedToken, StatementOutcome, TokenClass, Value,
};

/// One open document, current as of `version`.
pub struct Document {
    pub text: String,
    pub version: i32,
    /// Byte offset where each line starts (line 0 starts at 0).
    line_starts: Vec<usize>,
    /// The latest analysis pass. A parse failure is its first error
    /// outcome, spanning the offending token.
    pub outcomes: Vec<StatementOutcome>,
    pub tokens: Vec<SpannedToken>,
    pub session: Session,
}

impl Document {
    /// Analyze `text` afresh: a fresh Session runs the whole file, and
    /// the outcomes, tokens, and session state are cached for the
    /// editor's requests until the next change.
    pub fn new(text: String, version: i32) -> Self {
        let line_starts = line_starts(&text);
        let mut session = Session::default();
        let outcomes = evaluation_trace(&mut session, &text);
        let tokens = token_classes(&text).unwrap_or_default();
        Document {
            text,
            version,
            line_starts,
            outcomes,
            tokens,
            session,
        }
    }

    /// Byte offset <- LSP position (0-based line, UTF-16 column).
    pub fn offset_of(&self, position: lsp_types::Position) -> Option<usize> {
        let line = position.line as usize;
        let start = *self.line_starts.get(line)?;
        let rest = &self.text[start..];
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

    /// LSP position <- byte offset.
    pub fn position_of(&self, offset: usize) -> lsp_types::Position {
        let offset = offset.min(self.text.len());
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let start = self.line_starts[line];
        let mut units = 0u32;
        for c in self.text[start..offset].chars() {
            units += c.len_utf16() as u32;
        }
        lsp_types::Position {
            line: line as u32,
            character: units,
        }
    }

    /// The span as an LSP range.
    pub fn range_of(&self, span: Span) -> lsp_types::Range {
        lsp_types::Range {
            start: self.position_of(span.start),
            end: self.position_of(span.end),
        }
    }

    /// Diagnostics for the current pass: every statement that evaluated
    /// to an error, as one squiggle on that statement.
    pub fn diagnostics(&self) -> Vec<lsp_types::Diagnostic> {
        let mut out = Vec::new();
        for outcome in &self.outcomes {
            if let Some(error) = &outcome.error {
                out.push(diagnostic(self.range_of(outcome.span), &error.to_string()));
            }
        }
        out
    }

    /// Inline results (ADR-0066): one hint per value-producing
    /// statement, at the statement's end, labeled with the
    /// calculator's own rendering. Hints inside `range` only.
    pub fn inlay_hints(&self, range: lsp_types::Range) -> Vec<lsp_types::InlayHint> {
        let mut hints = Vec::new();
        for outcome in &self.outcomes {
            let Some(display) = &outcome.display else {
                continue;
            };
            let position = self.position_of(outcome.span.end);
            if position < range.start || position > range.end {
                continue;
            }
            hints.push(lsp_types::InlayHint {
                position,
                label: lsp_types::InlayHintLabel::String(format!("= {display}")),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: Some(true),
                padding_right: None,
                data: None,
            });
        }
        hints
    }

    /// Hover: what the name under the cursor means. User-defined names
    /// answer first, then the builtin catalogs.
    pub fn hover(&self, position: lsp_types::Position) -> Option<String> {
        let offset = self.offset_of(position)?;
        let token = self
            .tokens
            .iter()
            .find(|t| t.span.start <= offset && offset <= t.span.end)?;
        if token.class != TokenClass::Name {
            return None;
        }
        let name = &token.text;

        if let Some(sig) = self.session.env().function_signature(name) {
            let source = self
                .session
                .def_sources()
                .get(name)
                .map(|s| format!("\n```epher\n{s}\n```"))
                .unwrap_or_default();
            return Some(format!("**{sig}**: user-defined function{source}"));
        }
        if let Some(value) = self.session.env().get(name) {
            return Some(format!(
                "**{name}**: a variable in this document\n\n= {}",
                format_value(value, &DisplayPrefs::default())
            ));
        }
        if let Some(value) = self.session.env().constant(name) {
            return Some(format!(
                "**{name}**: a `const` in this document\n\n= {}",
                format_value(value, &DisplayPrefs::default())
            ));
        }
        for entry in catalog().iter().chain(supplementary_catalog()) {
            if entry.name != name {
                continue;
            }
            let kind_word = if entry.is_constant() { "constant" } else { "function" };
            let value_line = match builtin_value(entry.name, entry.kind) {
                Some(v) => format!("\n\n= {}", format_value(&v, &DisplayPrefs::default())),
                None => String::new(),
            };
            return Some(format!(
                "**{}** ({}): {}{value_line}",
                entry.signature, kind_word, entry.description
            ));
        }
        None
    }

    /// Completion: builtins (curated + supplementary), keywords, and
    /// the document's own names. The client filters by prefix; the
    /// server sends everything it knows, sorted by label.
    pub fn completions(&self) -> Vec<lsp_types::CompletionItem> {
        use lsp_types::CompletionItemKind as K;
        let mut items = Vec::new();
        for entry in catalog().iter().chain(supplementary_catalog()) {
            items.push(lsp_types::CompletionItem {
                label: entry.name.to_string(),
                kind: Some(if entry.is_constant() { K::CONSTANT } else { K::FUNCTION }),
                detail: Some(format!("{}: {}", entry.signature, entry.description)),
                ..Default::default()
            });
        }
        items.extend(snippet_items());
        for name in self.session.env().binding_names() {
            let detail = self
                .session
                .env()
                .get(&name)
                .map(|v| format!("= {}", format_value(v, &DisplayPrefs::default())));
            items.push(lsp_types::CompletionItem {
                label: name,
                kind: Some(K::VARIABLE),
                detail,
                ..Default::default()
            });
        }
        for name in self.session.env().constant_names() {
            items.push(lsp_types::CompletionItem {
                label: name,
                kind: Some(K::CONSTANT),
                ..Default::default()
            });
        }
        for name in self.session.env().function_names() {
            let detail = self.session.env().function_signature(&name);
            items.push(lsp_types::CompletionItem {
                label: name,
                kind: Some(K::FUNCTION),
                detail,
                ..Default::default()
            });
        }
        for kw in epher_core::KEYWORDS {
            items.push(lsp_types::CompletionItem {
                label: (*kw).to_string(),
                kind: Some(K::KEYWORD),
                ..Default::default()
            });
        }
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items.dedup_by(|a, b| a.label == b.label);
        items
    }

    /// Semantic tokens: the classified token stream, delta-encoded.
    pub fn semantic_tokens(&self) -> lsp_types::SemanticTokens {
        let mut data = Vec::new();
        let mut last_line = 0u32;
        let mut last_start = 0u32;
        for token in &self.tokens {
            let kind = match token.class {
                TokenClass::Number => TOKEN_NUMBER,
                TokenClass::Keyword => TOKEN_KEYWORD,
                TokenClass::Operator => TOKEN_OPERATOR,
                TokenClass::String => TOKEN_STRING,
                TokenClass::Unit => TOKEN_UNIT,
                TokenClass::Name => {
                    // a name directly followed by `(` is a call
                    if self.text[token.span.end..].starts_with('(') {
                        TOKEN_FUNCTION
                    } else {
                        TOKEN_VARIABLE
                    }
                }
            };
            let position = self.position_of(token.span.start);
            let delta_line = position.line - last_line;
            let delta_start = if delta_line == 0 {
                position.character - last_start
            } else {
                position.character
            };
            last_line = position.line;
            last_start = position.character;
            data.push(lsp_types::SemanticToken {
                delta_line,
                delta_start,
                length: (token.span.end - token.span.start) as u32,
                token_type: kind,
                token_modifiers_bitset: 0,
            });
        }
        lsp_types::SemanticTokens {
            result_id: None,
            data,
        }
    }
}

pub const TOKEN_NUMBER: u32 = 0;
pub const TOKEN_VARIABLE: u32 = 1;
pub const TOKEN_FUNCTION: u32 = 2;
pub const TOKEN_KEYWORD: u32 = 3;
pub const TOKEN_OPERATOR: u32 = 4;
pub const TOKEN_STRING: u32 = 5;
pub const TOKEN_UNIT: u32 = 6;

pub fn semantic_token_types() -> Vec<lsp_types::SemanticTokenType> {
    vec![
        lsp_types::SemanticTokenType::NUMBER,
        lsp_types::SemanticTokenType::VARIABLE,
        lsp_types::SemanticTokenType::FUNCTION,
        lsp_types::SemanticTokenType::KEYWORD,
        lsp_types::SemanticTokenType::OPERATOR,
        lsp_types::SemanticTokenType::STRING,
        lsp_types::SemanticTokenType::new("unit"),
    ]
}

/// The shared snippet assets, as completion items. One JSON file in the
/// repository is the source every client reads, shipped inside the
/// server so editors without a snippet engine of their own get them
/// too (ADR-0066).
fn snippet_items() -> Vec<lsp_types::CompletionItem> {
    const SNIPPETS: &str = include_str!("../assets/epher-snippets.json");
    let Ok(raw) = serde_json::from_str::<serde_json::Value>(SNIPPETS) else {
        return Vec::new();
    };
    let Some(entries) = raw.as_object() else {
        return Vec::new();
    };
    entries
        .values()
        .filter_map(|snippet| {
            let prefix = snippet.get("prefix")?.as_str()?.to_string();
            let body = snippet.get("body")?.as_array()?;
            let text = body
                .iter()
                .filter_map(|line| line.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let description = snippet
                .get("description")
                .and_then(|d| d.as_str())
                .map(str::to_string);
            Some(lsp_types::CompletionItem {
                label: prefix,
                kind: None,
                detail: description,
                insert_text: Some(text),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            })
        })
        .collect()
}

fn diagnostic(range: lsp_types::Range, message: &str) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range,
        severity: Some(lsp_types::DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("epher".to_string()),
        message: message.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// The live value of a builtin constant, for hover (functions are not
/// values).
fn builtin_value(name: &str, kind: CatalogKind) -> Option<Value> {
    if kind == CatalogKind::Constant {
        return epher_core::evaluate(name).ok();
    }
    None
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, c) in text.char_indices() {
        if c == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// The server's advertised capabilities, exactly what it answers.
pub fn server_capabilities() -> lsp_types::ServerCapabilities {
    use lsp_types::{
        CompletionOptions, HoverProviderCapability, OneOf,
        SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    };
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions::default()),
        semantic_tokens_provider: Some(
            lsp_types::SemanticTokensServerCapabilities::SemanticTokensOptions(
                SemanticTokensOptions {
                    work_done_progress_options: Default::default(),
                    legend: SemanticTokensLegend {
                        token_types: semantic_token_types(),
                        token_modifiers: vec![],
                    },
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    range: None,
                },
            ),
        ),
        inlay_hint_provider: Some(OneOf::Left(true)),
        ..Default::default()
    }
}

/// The document store, keyed by URI text.
pub type Documents = HashMap<String, Document>;
