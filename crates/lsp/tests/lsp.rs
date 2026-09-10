//! The LSP wire is the seam (ADR-0066): a client drives the server
//! over an in-memory connection and asserts on the messages that come
//! back, exactly as an extension would.

use std::error::Error;
use std::str::FromStr;

use crossbeam_channel::Receiver;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    DidOpenTextDocumentParams, InitializeParams, Position, PublishDiagnosticsParams,
    TextDocumentItem, TextDocumentIdentifier, TextDocumentPositionParams, Uri,
};
use serde_json::{json, Value};

struct Client {
    connection: Connection,
    next_id: i32,
}

impl Client {
    fn send(&self, message: Message) {
        self.connection.sender.send(message).expect("send");
    }

    fn request(&mut self, method: &str, params: Value) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.send(Message::Request(Request::new(id.into(), method.to_string(), params)));
        id
    }

    fn notify(&self, method: &str, params: Value) {
        self.send(Message::Notification(Notification::new(method.to_string(), params)));
    }

    /// The response with the given id, skipping notifications.
    fn response(&self, id: i32) -> Response {
        loop {
            let message = self.connection.receiver.recv().expect("server answers");
            if let Message::Response(response) = message {
                if response.id == id.into() {
                    return response;
                }
            }
        }
    }

    /// The next publishDiagnostics notification.
    fn diagnostics(&self) -> PublishDiagnosticsParams {
        loop {
            let message = self.connection.receiver.recv().expect("server publishes");
            if let Message::Notification(notification) = message {
                if notification.method == "textDocument/publishDiagnostics" {
                    return serde_json::from_value(notification.params).expect("params");
                }
            }
        }
    }
}

fn uri(text: &str) -> Uri {
    Uri::from_str(text).expect("uri")
}

fn start() -> Result<Client, Box<dyn Error>> {
    let (server, client_connection) = Connection::memory();
    // The server thread is never joined: it ends on shutdown, and a
    // failing test must not hang the harness on a thread parked in
    // recv().
    std::thread::spawn(move || {
        let _ = epher_lsp::run(server);
    });
    let mut client = Client {
        connection: client_connection,
        next_id: 1,
    };
    let id = client.request("initialize", serde_json::to_value(InitializeParams::default())?);
    let response = client.response(id);
    assert!(response.error.is_none(), "initialize succeeds");
    client.notify("initialized", json!({}));
    Ok(client)
}

fn open(client: &mut Client, name: &str, text: &str) -> Uri {
    let uri = uri(name);
    client.notify(
        "textDocument/didOpen",
        serde_json::to_value(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "epher".to_string(),
                version: 1,
                text: text.to_string(),
            },
        })
        .expect("params"),
    );
    uri
}

#[test]
fn a_document_evaluates_and_reports_inline_results_and_errors() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    // The pass is fail-fast: `y = 3` after the error never runs, so it
    // earns no hint and no diagnostic.
    let uri = open(&mut client, "memo://calc.epher", "x = 40 + 2\n1 / 0\ny = 3");

    let published = client.diagnostics();
    assert_eq!(published.diagnostics.len(), 1, "one error: the division");
    assert!(published.diagnostics[0].message.contains("division by zero"));
    assert_eq!(published.diagnostics[0].range.start.line, 1);
    assert_eq!(published.version, Some(1));

    // inline results: two value-producing statements get hints
    let id = client.request(
        "textDocument/inlayHint",
        json!({
            "textDocument": { "uri": uri.to_string() },
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 2, "character": 5 } }
        }),
    );
    let response = client.response(id);
    let hints = response.result.expect("hints").as_array().expect("array").clone();
    let labels: Vec<&str> = hints
        .iter()
        .map(|h| h["label"].as_str().expect("label"))
        .collect();
    assert_eq!(labels, vec!["= 42"]);
    assert_eq!(hints[0]["position"]["line"], 0);

    Ok(())
}

#[test]
fn hover_and_completion_answer_from_the_catalog() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let uri = open(&mut client, "memo://pi.epher", "pi");
    let _ = client.diagnostics();

    // hover over `pi` (position 0..2 on line 0)
    let id = client.request(
        "textDocument/hover",
        serde_json::to_value(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 0, character: 1 },
        })?,
    );
    let response = client.response(id);
    let hover = response.result.expect("hover");
    let value = hover["contents"]["value"].as_str().expect("markdown");
    assert!(value.contains("circle constant"), "hover says what pi is: {value}");
    assert!(value.contains("3.14159"), "hover shows the live value: {value}");

    // completion lists builtins and keywords
    let id = client.request(
        "textDocument/completion",
        serde_json::to_value(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line: 0, character: 0 },
        })?,
    );
    let response = client.response(id);
    let items = response.result.expect("items").as_array().expect("array").clone();
    let labels: Vec<&str> = items
        .iter()
        .map(|i| i["label"].as_str().expect("label"))
        .collect();
    assert!(labels.contains(&"sin"), "functions: {labels:?}");
    assert!(labels.contains(&"pi"), "constants: {labels:?}");
    assert!(labels.contains(&"def"), "keywords: {labels:?}");
    assert!(labels.contains(&"satphen"), "supplementary: {labels:?}");

    Ok(())
}

#[test]
fn an_edit_reanalyzes_and_republishes() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let uri = open(&mut client, "memo://edit.epher", "1 / 0");
    let published = client.diagnostics();
    assert_eq!(published.diagnostics.len(), 1);

    client.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri.to_string(), "version": 2 },
            "contentChanges": [ { "text": "2 ^ 10" } ]
        }),
    );
    let published = client.diagnostics();
    assert!(published.diagnostics.is_empty(), "the fix clears the error");
    assert_eq!(published.version, Some(2));

    Ok(())
}

/// Keep the receiver type named for the client struct: the in-memory
/// connection's receiver is read through `recv`, never polled.
#[allow(dead_code)]
fn _receiver_is_drained(_rx: &Receiver<Message>) {}
