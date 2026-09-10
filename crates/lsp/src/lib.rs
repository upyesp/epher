//! epher-lsp: the epher language server (ADR-0066).
//!
//! One synchronous binary, shared by every IDE extension: it speaks
//! LSP over stdio and answers with the same evaluator the calculator
//! itself uses. Features: full-document diagnostics, inline results
//! (inlay hints), hover with canonical signatures, completion (with
//! the shared snippets), and semantic tokens.
//!
//! Edits are debounced: a change arms a short quiet-gap timer, and the
//! pass runs for the newest text once typing pauses. A document's
//! first open analyzes immediately, so an opened file shows its
//! answers at once.

pub mod analysis;

use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use analysis::{server_capabilities, Document, Documents};
use crossbeam_channel::select;
use lsp_types::notification::{Notification, PublishDiagnostics};
use lsp_types::request::{Completion, HoverRequest, InlayHintRequest, SemanticTokensFullRequest, Request as LspRequest};
use lsp_server::{Connection, Message, Notification as ServerNotification, Request as ServerRequest, Response};

/// How long typing must pause before the pass runs (ADR-0066: "a short
/// debounce").
const DEBOUNCE: Duration = Duration::from_millis(200);

/// Serve until shutdown. `Connection::memory()` pairs with this for
/// tests: one end drives, the other asserts.
pub fn run(connection: Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    let _params = connection.initialize(serde_json::to_value(server_capabilities())?)?;
    eprintln!("epher-lsp: ready");
    let mut documents = Documents::new();
    // Edits seen but not yet analyzed: the debounce holds the newest
    // text per document until the quiet gap ends.
    let mut pending: HashMap<String, (String, i32)> = HashMap::new();
    // Bumped on every held edit; a timer fires only if no newer edit
    // replaced it meanwhile.
    let generation = Arc::new(AtomicU64::new(0));
    let (tick_tx, tick_rx) = crossbeam_channel::unbounded::<()>();
    loop {
        select! {
            recv(&connection.receiver) -> msg => {
                let Ok(msg) = msg else { return Ok(()) };
                match msg {
                    Message::Request(req) => {
                        if connection.handle_shutdown(&req)? {
                            return Ok(());
                        }
                        let response = answer(&req, &documents);
                        connection.sender.send(Message::Response(response))?;
                    }
                    Message::Notification(notification) => {
                        hold_or_apply(
                            &notification,
                            &mut documents,
                            &mut pending,
                            &generation,
                            &tick_tx,
                            &connection,
                        )?;
                    }
                    Message::Response(_) => {}
                }
            }
            recv(tick_rx) -> _ => {
                // The quiet gap ended behind at least one held edit.
                for (key, (text, version)) in std::mem::take(&mut pending) {
                    let document = Document::new(text, version);
                    let uri: lsp_types::Uri = key.parse().map_err(|e| format!("{e}"))?;
                    publish(&connection, &uri, &document)?;
                    documents.insert(key, document);
                }
            }
        }
    }
}

/// One editor notification, three ways: an open applies at once (a
/// freshly opened file should show its answers immediately); a change
/// is held for the debounce; a close drops everything known.
fn hold_or_apply(
    notification: &ServerNotification,
    documents: &mut Documents,
    pending: &mut HashMap<String, (String, i32)>,
    generation: &Arc<AtomicU64>,
    tick_tx: &crossbeam_channel::Sender<()>,
    connection: &Connection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match notification.method.as_str() {
        "textDocument/didOpen" => {
            let params: lsp_types::DidOpenTextDocumentParams =
                serde_json::from_value(notification.params.clone())?;
            let uri = params.text_document.uri;
            let key = uri.to_string();
            pending.remove(&key);
            let document =
                Document::new(params.text_document.text, params.text_document.version);
            publish(connection, &uri, &document)?;
            documents.insert(key, document);
        }
        "textDocument/didChange" => {
            let params: lsp_types::DidChangeTextDocumentParams =
                serde_json::from_value(notification.params.clone())?;
            let uri = params.text_document.uri;
            if let Some(change) = params.content_changes.last() {
                let key = uri.to_string();
                pending.insert(key, (change.text.clone(), params.text_document.version));
                schedule(&generation, tick_tx.clone());
            }
        }
        "textDocument/didClose" => {
            let params: lsp_types::DidCloseTextDocumentParams =
                serde_json::from_value(notification.params.clone())?;
            let key = params.text_document.uri.to_string();
            pending.remove(&key);
            documents.remove(&key);
        }
        _ => {}
    }
    Ok(())
}

/// Arm the quiet gap: sleep for [`DEBOUNCE`], then tick if no newer
/// edit replaced this one meanwhile. One sleeping thread per held edit
/// is cheap; only the last one fires.
fn schedule(generation: &Arc<AtomicU64>, tick_tx: crossbeam_channel::Sender<()>) {
    let armed = generation.fetch_add(1, Ordering::SeqCst) + 1;
    let generation = Arc::clone(generation);
    std::thread::spawn(move || {
        std::thread::sleep(DEBOUNCE);
        if generation.load(Ordering::SeqCst) == armed {
            let _ = tick_tx.send(());
        }
    });
}

fn publish(
    connection: &Connection,
    uri: &lsp_types::Uri,
    document: &Document,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let params = lsp_types::PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: document.diagnostics(),
        version: Some(document.version),
    };
    connection
        .sender
        .send(Message::Notification(ServerNotification::new(
            PublishDiagnostics::METHOD.to_string(),
            params,
        )))?;
    Ok(())
}

/// The requests the server answers; anything else gets `null`.
fn answer(req: &ServerRequest, documents: &Documents) -> Response {
    let id = req.id.clone();
    let result: Result<Option<serde_json::Value>, String> = match req.method.as_str() {
        <InlayHintRequest as LspRequest>::METHOD => inlay_hint(req, documents)
            .map(|o| o.map(|v| serde_json::to_value(v).expect("serializes"))),
        <HoverRequest as LspRequest>::METHOD => hover(req, documents)
            .map(|o| o.map(|v| serde_json::to_value(v).expect("serializes"))),
        <Completion as LspRequest>::METHOD => completion(req, documents)
            .map(|o| o.map(|v| serde_json::to_value(v).expect("serializes"))),
        <SemanticTokensFullRequest as LspRequest>::METHOD => semantic_tokens(req, documents)
            .map(|o| o.map(|v| serde_json::to_value(v).expect("serializes"))),
        _ => Ok(None),
    };
    match result {
        Ok(value) => Response::new_ok(id, value),
        Err(e) => Response::new_err(id, -32602, e),
    }
}

/// The document a request points at, or an error naming the miss.
fn document_for<'a>(req: &ServerRequest, documents: &'a Documents) -> Result<&'a Document, String> {
    let uri = req
        .params
        .get("textDocument")
        .and_then(|td| td.get("uri"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| "missing textDocument.uri".to_string())?;
    documents
        .get(uri)
        .ok_or_else(|| format!("document not open: {uri}"))
}

fn inlay_hint(
    req: &ServerRequest,
    documents: &Documents,
) -> Result<Option<Vec<lsp_types::InlayHint>>, String> {
    let document = document_for(req, documents)?;
    let range: lsp_types::Range = serde_json::from_value(
        req.params
            .get("range")
            .cloned()
            .ok_or_else(|| "missing range".to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(Some(document.inlay_hints(range)))
}

fn hover(req: &ServerRequest, documents: &Documents) -> Result<Option<lsp_types::Hover>, String> {
    let document = document_for(req, documents)?;
    let position: lsp_types::Position = serde_json::from_value(
        req.params
            .get("position")
            .cloned()
            .ok_or_else(|| "missing position".to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(document.hover(position).map(|markup| lsp_types::Hover {
        contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: markup,
        }),
        range: None,
    }))
}

fn completion(
    req: &ServerRequest,
    documents: &Documents,
) -> Result<Option<lsp_types::CompletionResponse>, String> {
    let document = document_for(req, documents)?;
    Ok(Some(lsp_types::CompletionResponse::Array(
        document.completions(),
    )))
}

fn semantic_tokens(
    req: &ServerRequest,
    documents: &Documents,
) -> Result<Option<lsp_types::SemanticTokens>, String> {
    let document = document_for(req, documents)?;
    Ok(Some(document.semantic_tokens()))
}
