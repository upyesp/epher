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
use std::time::{Duration, Instant};

use analysis::{server_capabilities, Document, Documents};
use crossbeam_channel::{RecvTimeoutError, SendTimeoutError};
use lsp_types::notification::{Notification, PublishDiagnostics};
use lsp_types::request::{Completion, HoverRequest, InlayHintRequest, SemanticTokensFullRequest, Request as LspRequest};
use lsp_server::{Connection, Message, Notification as ServerNotification, Request as ServerRequest, Response};

/// How long typing must pause before the pass runs (ADR-0066: "a short
/// debounce").
const DEBOUNCE: Duration = Duration::from_millis(200);

/// How often the main loop re-checks its channels even when idle. The
/// stdio transport's channels are rendezvous channels (capacity zero):
/// every message is a thread-to-thread handoff, and on one field
/// machine (Flatpak IDE, kernel 6.x) such a handoff was observed to
/// lose its wakeup — the main loop slept in futex, the writer slept in
/// its receive, and a finished run's response sat undelivered forever.
/// Polling on a timeout turns any such loss into at most POLL_DELAY of
/// added latency instead of a permanent wedge.
const POLL_DELAY: Duration = Duration::from_millis(100);

/// An edit held for the debounce: the newest text, the version it came
/// with, and when it was seen.
struct HeldEdit {
    text: String,
    version: i32,
    seen: Instant,
}

/// Serve until shutdown. `Connection::memory()` pairs with this for
/// tests: one end drives, the other asserts.
pub fn run(connection: Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    let _params = connection.initialize(serde_json::to_value(server_capabilities())?)?;
    eprintln!("epher-lsp: ready");
    let mut documents = Documents::new();
    // Edits seen but not yet analyzed: the debounce holds the newest
    // text per document until the quiet gap ends. Each hold carries
    // the instant it was seen, so the timed poll below can apply it
    // once the gap has passed.
    let mut pending: HashMap<String, HeldEdit> = HashMap::new();
    loop {
        // Held edits whose quiet gap has ended: apply and publish.
        let now = Instant::now();
        let ready: Vec<String> = pending
            .iter()
            .filter(|(_, held)| now.duration_since(held.seen) >= DEBOUNCE)
            .map(|(key, _)| key.clone())
            .collect();
        for key in ready {
            if let Some(held) = pending.remove(&key) {
                let document = Document::new(held.text, held.version);
                let uri: lsp_types::Uri = key.parse().map_err(|e| format!("{e}"))?;
                publish(&connection, &uri, &document)?;
                documents.insert(key, document);
            }
        }
        let message = match connection.receiver.recv_timeout(POLL_DELAY) {
            Ok(message) => message,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        };
        match message {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let response = if req.method == "epher/run" {
                    run_request(&req, &documents, &pending)
                } else {
                    answer(&req, &documents)
                };
                send_message(&connection, Message::Response(response))?;
            }
            Message::Notification(notification) => {
                hold_or_apply(&notification, &mut documents, &mut pending, &connection)?;
            }
            Message::Response(_) => {}
        }
    }
}

/// The script run (ADR-0069): `epher/run` evaluates the document's
/// newest text through the shell's script loop and answers with the
/// per-statement lines and the run's plot SVGs. A document still inside
/// the debounce runs from its held text, so Run always reflects the
/// buffer the editor showed when the command fired. The run is
/// hermetic (a fresh session), and the editor's locale names its
/// messages.
fn run_request(
    req: &ServerRequest,
    documents: &Documents,
    pending: &HashMap<String, HeldEdit>,
) -> Response {
    let uri = req.params["textDocument"]["uri"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if uri.is_empty() {
        return Response::new_err(
            req.id.clone(),
            -32602,
            "epher/run needs a textDocument uri".to_string(),
        );
    }
    // The held edit is the newest text: it wins over the analyzed
    // document, which only reflects the last quiet gap.
    let text = pending
        .get(&uri)
        .map(|held| held.text.clone())
        .or_else(|| {
            documents
                .iter()
                .find(|(key, _)| key.as_str() == uri)
                .map(|(_, document)| document.text.clone())
        });
    let Some(text) = text else {
        return Response::new_err(
            req.id.clone(),
            -32602,
            "document not open".to_string(),
        );
    };
    let localizer = epher_i18n::Localizer::resolve(None, &[]);
    let run = epher_shell::run_script(&text, &localizer);
    let result = serde_json::json!({
        "statements": run
            .lines
            .iter()
            .map(|line| serde_json::json!({
                "line": line.line,
                "source": line.source,
                "display": line.display,
                "error": line.error,
            }))
            .collect::<Vec<_>>(),
        "svgs": run.svgs,
    });
    Response::new_ok(req.id.clone(), result)
}

/// One editor notification, three ways: an open applies at once (a
/// freshly opened file should show its answers immediately); a change
/// is held for the debounce; a close drops everything known.
fn hold_or_apply(
    notification: &ServerNotification,
    documents: &mut Documents,
    pending: &mut HashMap<String, HeldEdit>,
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
                pending.insert(
                    key,
                    HeldEdit {
                        text: change.text.clone(),
                        version: params.text_document.version,
                        seen: Instant::now(),
                    },
                );
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

/// Send through the connection's writer channel, retrying on timeout.
/// The channel is a rendezvous (capacity zero): a plain send parks
/// until the writer thread comes to collect, and a lost wakeup there
/// would park the whole server (observed in the field: both the main
/// loop and LspServerWriter asleep in futex, a finished response
/// never written). A timed send that retries re-arms the handoff; the
/// writer is parked in its receive, so the retry meets it at once.
fn send_message(
    connection: &Connection,
    mut message: Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    for _ in 0..120 {
        match connection.sender.send_timeout(message, Duration::from_secs(1)) {
            Ok(()) => return Ok(()),
            Err(SendTimeoutError::Timeout(again)) => message = again,
            Err(SendTimeoutError::Disconnected(lost)) => {
                let note = if matches!(lost, Message::Response(_)) {
                    "a response"
                } else {
                    "a message"
                };
                return Err(format!(
                    "the LSP writer channel is gone; dropped {note}"
                )
                .into());
            }
        }
    }
    Err("the LSP writer never collected a message in two minutes".into())
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
    send_message(
        connection,
        Message::Notification(ServerNotification::new(
            PublishDiagnostics::METHOD.to_string(),
            params,
        )),
    )
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
