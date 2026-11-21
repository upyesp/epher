//! The markdown seam (ADR-0088): a client drives the server over the
//! same in-memory connection the lsp.rs harness uses, opening
//! markdown documents whose epher fences must analyze, answer, and
//! run at their real positions in the markdown file.

use std::error::Error;
use std::str::FromStr;

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, Position,
    PublishDiagnosticsParams, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, Uri, VersionedTextDocumentIdentifier,
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
        self.send(Message::Request(Request::new(
            id.into(),
            method.to_string(),
            params,
        )));
        id
    }

    fn notify(&self, method: &str, params: Value) {
        self.send(Message::Notification(Notification::new(
            method.to_string(),
            params,
        )));
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

    /// The next publishDiagnostics within `millis`, or None.
    fn diagnostics_within(&self, millis: u64) -> Option<PublishDiagnosticsParams> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(millis);
        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            let message = self.connection.receiver.recv_timeout(left).ok()?;
            if let Message::Notification(notification) = message {
                if notification.method == "textDocument/publishDiagnostics" {
                    return Some(serde_json::from_value(notification.params).expect("params"));
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
    std::thread::spawn(move || {
        let _ = epher_lsp::run(server);
    });
    let mut client = Client {
        connection: client_connection,
        next_id: 1,
    };
    let id = client.request(
        "initialize",
        serde_json::to_value(InitializeParams::default())?,
    );
    let response = client.response(id);
    assert!(response.error.is_none(), "initialize succeeds");
    client.notify("initialized", json!({}));
    Ok(client)
}

fn open_with_language(client: &mut Client, name: &str, language: &str, text: &str) -> Uri {
    let uri = uri(name);
    client.notify(
        "textDocument/didOpen",
        serde_json::to_value(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: language.to_string(),
                version: 1,
                text: text.to_string(),
            },
        })
        .expect("params"),
    );
    uri
}

fn open_markdown(client: &mut Client, name: &str, text: &str) -> Uri {
    open_with_language(client, name, "markdown", text)
}

fn change(client: &Client, uri: &Uri, version: i32, text: &str) {
    client.notify(
        "textDocument/didChange",
        serde_json::to_value(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.to_string(),
            }],
        })
        .expect("params"),
    );
}

fn hover_at(client: &mut Client, uri: &Uri, line: u32, character: u32) -> Option<Value> {
    let id = client.request(
        "textDocument/hover",
        serde_json::to_value(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line, character },
        })
        .expect("params"),
    );
    client.response(id).result
}

#[test]
fn fenced_diagnostics_hints_and_hover_land_in_markdown_coordinates() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let text = "prose line\n\n```epher\ngamma = 2\n1 / 0\n```\n\ntail prose";
    let uri = open_markdown(&mut client, "memo://notes.md", text);

    let published = client.diagnostics();
    assert_eq!(published.diagnostics.len(), 1, "one error: the division");
    assert!(published.diagnostics[0]
        .message
        .contains("division by zero"));
    // The error statement is the markdown file's line 4, not line 0
    // of some virtual text.
    assert_eq!(published.diagnostics[0].range.start.line, 4);
    assert_eq!(published.diagnostics[0].range.start.character, 0);
    assert_eq!(published.version, Some(1));

    // Hover inside the fence answers; hover in the prose does not.
    let inside = hover_at(&mut client, &uri, 3, 2).expect("hover inside the fence");
    assert!(
        inside["contents"]["value"]
            .as_str()
            .expect("markdown")
            .contains("a variable in this document"),
        "gamma's hover: {inside}"
    );
    assert!(
        hover_at(&mut client, &uri, 0, 2).map_or(true, |v| v.is_null()),
        "the prose line answers no hover"
    );

    Ok(())
}

#[test]
fn fences_share_one_session_and_hints_land_at_their_fences() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let text = "```epher\na = 21\n```\n\ntext between\n\n```epher\na * 2\n```";
    let uri = open_markdown(&mut client, "memo://tutorial.md", text);
    let _ = client.diagnostics();

    let id = client.request(
        "textDocument/inlayHint",
        json!({
            "textDocument": { "uri": uri.to_string() },
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 7, "character": 6 } }
        }),
    );
    let response = client.response(id);
    let result = response.result.expect("hints");
    let hints = result.as_array().expect("array");
    let labels: Vec<&str> = hints
        .iter()
        .map(|h| h["label"].as_str().expect("label"))
        .collect();
    // `a * 2` sees `a` from the first fence: one session across blocks.
    assert_eq!(labels, vec!["= 21", "= 42"]);
    // The hints sit at the ends of the markdown lines 1 and 7.
    assert_eq!(hints[0]["position"]["line"], 1);
    assert_eq!(hints[0]["position"]["character"], 6);
    assert_eq!(hints[1]["position"]["line"], 7);
    assert_eq!(hints[1]["position"]["character"], 5);

    Ok(())
}

#[test]
fn the_run_anchors_statements_and_the_answer_to_markdown_lines() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let text = "```epher\na = 5\n```\n\ntext between\n\n```epher\na * 2\n```";
    let uri = open_markdown(&mut client, "memo://run.md", text);
    let _ = client.diagnostics();

    let id = client.request(
        "epher/run",
        json!({ "textDocument": { "uri": uri.to_string() } }),
    );
    let response = client.response(id);
    assert!(
        response.error.is_none(),
        "run answers: {:?}",
        response.error
    );
    let result = response.result.expect("run result");
    let statements = result["statements"].as_array().expect("statements");
    assert_eq!(statements.len(), 2);
    // The statement lines are markdown lines, so the results pane's
    // links jump into the right fences.
    assert_eq!(statements[0]["line"], 1);
    assert_eq!(statements[1]["line"], 7);
    assert_eq!(statements[1]["display"], "= 10");
    assert_eq!(result["answerLine"], 7);

    Ok(())
}

#[test]
fn semantic_tokens_decode_to_markdown_positions() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let text = "prose\n\n```epher\n1 + 1\n```\n";
    let uri = open_markdown(&mut client, "memo://tokens.md", text);
    let _ = client.diagnostics();

    let id = client.request(
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri.to_string() } }),
    );
    let response = client.response(id);
    let flat: Vec<u64> = response.result.expect("tokens")["data"]
        .as_array()
        .expect("array")
        .iter()
        .map(|v| v.as_u64().expect("number"))
        .collect();
    // Decode the wire's flat [deltaLine, deltaStart, length,
    // tokenType, modifiers] quintuplets into markdown positions.
    let legend = [
        "number", "variable", "function", "keyword", "operator", "string", "unit",
    ];
    let mut line = 0u32;
    let mut start = 0u32;
    let mut absolute = Vec::new();
    for token in flat.chunks(5) {
        let delta_line = token[0] as u32;
        let delta_start = token[1] as u32;
        let length = token[2] as u32;
        let token_type = token[3] as usize;
        line += delta_line;
        start = if delta_line == 0 {
            start + delta_start
        } else {
            delta_start
        };
        absolute.push((line, start, length, legend[token_type]));
    }
    assert_eq!(
        absolute,
        vec![
            (3, 0, 1, "number"),
            (3, 2, 1, "operator"),
            (3, 4, 1, "number"),
        ],
        "the fence's tokens sit on markdown line 3"
    );

    Ok(())
}

#[test]
fn an_edit_keeps_markdown_through_the_debounce() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let uri = open_markdown(
        &mut client,
        "memo://editing.md",
        "# Notes\n\n```epher\n1 / 0\n```\n",
    );
    let published = client.diagnostics();
    assert_eq!(published.diagnostics[0].range.start.line, 3);

    change(
        &client,
        &uri,
        2,
        "# Notes\n\nsome new prose\n\n```epher\n9 / 0\n```\n",
    );
    let republished = client
        .diagnostics_within(2000)
        .expect("the edit republishes");
    assert_eq!(republished.diagnostics.len(), 1);
    // The new error lands on markdown line 5: still a projection.
    assert_eq!(republished.diagnostics[0].range.start.line, 5);
    assert_eq!(republished.version, Some(2));
    assert!(
        client.diagnostics_within(500).is_none(),
        "quiet after the republish"
    );

    Ok(())
}

#[test]
fn the_language_id_and_the_uri_suffix_decide_markdown() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let text = "```epher\n1 / 0\n```";

    // Case-insensitive language id.
    let uri = open_with_language(&mut client, "memo://a.md", "Markdown", text);
    let published = client.diagnostics();
    assert_eq!(published.uri, uri, "diagnostics name the opened document");
    assert_eq!(published.diagnostics.len(), 1);
    assert_eq!(published.diagnostics[0].range.start.line, 1, "projected");

    // Empty language id falls back to the suffix.
    let uri = open_with_language(&mut client, "memo://b.md", "", text);
    let published = client.diagnostics();
    assert_eq!(published.uri, uri);
    assert_eq!(published.diagnostics.len(), 1);
    assert_eq!(published.diagnostics[0].range.start.line, 1, "projected");

    // A markdown file forced to the epher language is a plain script:
    // the fence lines themselves are the parse errors.
    let uri = open_with_language(&mut client, "memo://forced.epher", "epher", text);
    let published = client.diagnostics();
    assert_eq!(published.uri, uri);
    assert!(
        !published.diagnostics.is_empty(),
        "the raw fence text does not parse as epher"
    );
    assert_eq!(
        published.diagnostics[0].range.start.line, 0,
        "unprojected: the error is on the fence line itself"
    );

    Ok(())
}

#[test]
fn a_markdown_file_without_fences_is_the_empty_script() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    open_markdown(
        &mut client,
        "memo://plain.md",
        "# Title\n\nJust prose, no fences.\n",
    );
    let published = client.diagnostics();
    assert!(
        published.diagnostics.is_empty(),
        "prose earns no diagnostics"
    );

    Ok(())
}

#[test]
fn other_languages_fences_are_ignored() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let text = "```rust\nfn main() { let x = 1 / 0; }\n```\n\n```epher\n1 / 0\n```";
    open_markdown(&mut client, "memo://mixed.md", text);
    let published = client.diagnostics();
    assert_eq!(
        published.diagnostics.len(),
        1,
        "the rust fence is nobody's business"
    );
    assert_eq!(
        published.diagnostics[0].range.start.line, 5,
        "the epher fence"
    );
    assert!(published.diagnostics[0]
        .message
        .contains("division by zero"));

    Ok(())
}
