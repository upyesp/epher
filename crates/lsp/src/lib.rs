//! epher-lsp: the epher language server (ADR-0066).
//!
//! One synchronous binary, shared by every IDE extension: it speaks
//! LSP over stdio and answers with the same evaluator the calculator
//! itself uses. Features: full-document diagnostics, inline results
//! (inlay hints), hover with canonical signatures, completion, and
//! semantic tokens.

pub mod analysis;

use std::error::Error;

use analysis::{server_capabilities, Document, Documents};
use lsp_types::notification::{Notification, PublishDiagnostics};
use lsp_types::request::{Completion, HoverRequest, InlayHintRequest, SemanticTokensFullRequest, Request as LspRequest};
use lsp_server::{Connection, Message, Notification as ServerNotification, Request as ServerRequest, Response};

/// Serve until shutdown. `Connection::memory()` pairs with this for
/// tests: one end drives, the other asserts.
pub fn run(connection: Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
    let _params = connection.initialize(serde_json::to_value(server_capabilities())?)?;
    eprintln!("epher-lsp: ready");
    let mut documents = Documents::new();
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let response = answer(&req, &documents);
                connection
                    .sender
                    .send(Message::Response(response))?;
            }
            Message::Notification(notification) => {
                observe(&notification, &mut documents, &connection)?;
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

/// Editor edits arrive: re-analyze the document and publish its
/// diagnostics. The same pass feeds every later request until the next
/// change, so an edit costs one evaluation, not one per feature.
fn observe(
    notification: &ServerNotification,
    documents: &mut Documents,
    connection: &Connection,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match notification.method.as_str() {
        "textDocument/didOpen" => {
            let params: lsp_types::DidOpenTextDocumentParams =
                serde_json::from_value(notification.params.clone())?;
            let uri = params.text_document.uri;
            let document = Document::new(params.text_document.text, params.text_document.version);
            publish(connection, &uri, &document)?;
            documents.insert(uri.to_string(), document);
        }
        "textDocument/didChange" => {
            let params: lsp_types::DidChangeTextDocumentParams =
                serde_json::from_value(notification.params.clone())?;
            let uri = params.text_document.uri;
            let key = uri.to_string();
            if let Some(change) = params.content_changes.last() {
                let version = params.text_document.version;
                let document = Document::new(change.text.clone(), version);
                publish(connection, &uri, &document)?;
                documents.insert(key, document);
            }
        }
        "textDocument/didClose" => {
            let params: lsp_types::DidCloseTextDocumentParams =
                serde_json::from_value(notification.params.clone())?;
            documents.remove(&params.text_document.uri.to_string());
        }
        _ => {}
    }
    Ok(())
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
        <InlayHintRequest as LspRequest>::METHOD => {
            inlay_hint(req, documents).map(|o| o.map(|v| serde_json::to_value(v).expect("serializes")))
        }
        <HoverRequest as LspRequest>::METHOD => {
            hover(req, documents).map(|o| o.map(|v| serde_json::to_value(v).expect("serializes")))
        }
        <Completion as LspRequest>::METHOD => {
            completion(req, documents).map(|o| o.map(|v| serde_json::to_value(v).expect("serializes")))
        }
        <SemanticTokensFullRequest as LspRequest>::METHOD => {
            semantic_tokens(req, documents).map(|o| o.map(|v| serde_json::to_value(v).expect("serializes")))
        }
        _ => Ok(None),
    };
    match result {
        Ok(value) => Response::new_ok(id, value),
        Err(e) => Response::new_err(id, -32602, e.to_string()),
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
