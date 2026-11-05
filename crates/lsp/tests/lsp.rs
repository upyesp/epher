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

    /// The next publishDiagnostics within `millis`, or None: how the
    /// tests wait out the debounce without flaking.
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

#[test]
fn snippets_arrive_as_snippet_completion_items() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let uri = open(&mut client, "memo://snip.epher", "1");
    let _ = client.diagnostics();
    let id = client.request(
        "textDocument/completion",
        serde_json::to_value(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line: 0, character: 0 },
        })?,
    );
    let response = client.response(id);
    let items = response.result.expect("items").as_array().expect("array").clone();
    let def = items
        .iter()
        .find(|i| i["label"] == "def")
        .expect("a def snippet");
    assert_eq!(def["insertTextFormat"], 2, "snippet format");
    assert!(def["insertText"].as_str().expect("body").contains("def ${1:"));
    Ok(())
}

#[test]
fn rapid_edits_coalesce_into_one_pass() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let uri = open(&mut client, "memo://burst.epher", "1");
    let first = client.diagnostics();
    assert_eq!(first.version, Some(1));

    // Two keystrokes inside the quiet gap: one pass for the newest text.
    for (version, text) in [(2, "2"), (3, "3")] {
        client.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri.to_string(), "version": version },
                "contentChanges": [ { "text": text } ]
            }),
        );
    }
    let coalesced = client
        .diagnostics_within(2000)
        .expect("the quiet gap ends with one publish");
    assert_eq!(coalesced.version, Some(3), "the newest text wins");
    assert!(
        client.diagnostics_within(500).is_none(),
        "no second pass for the intermediate version"
    );
    Ok(())
}

#[test]
fn unit_suffixes_color_as_units() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    // 2 m is a quantity (the m is a unit); y is a variable; f(1) a call.
    let uri = open(&mut client, "memo://units.epher", "2 m + y\nf(1)");
    let _ = client.diagnostics();
    let id = client.request(
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri.to_string() } }),
    );
    let response = client.response(id);
    let data = response.result.expect("tokens")["data"]
        .as_array()
        .expect("array")
        .clone();
    // legend: 0 number, 1 variable, 2 function, 3 keyword, 4 operator, 5 string, 6 unit
    let types: Vec<u64> = data
        .chunks(5)
        .map(|t| t[3].as_u64().expect("token type"))
        .collect();
    assert_eq!(types, vec![0, 6, 4, 1, 2, 4, 0, 4], "the m in 2 m is a unit");
    Ok(())
}

#[test]
fn run_reports_lines_and_graphs() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    // One value statement, one statement with no value, one plot: the
    // run's report mirrors what the calculator would print, and the
    // plot renders as an in-memory SVG document (ADR-0069).
    let uri = open(
        &mut client,
        "memo://run.epher",
        "x = 40 + 2\ndef double(n) = n * 2\ngraph sin(x) from 0 to 6\n",
    );
    let _ = client.diagnostics();
    let id = client.request(
        "epher/run",
        json!({ "textDocument": { "uri": uri.to_string() } }),
    );
    let response = client.response(id);
    assert!(response.error.is_none(), "run succeeds: {:?}", response.error);
    let result = response.result.expect("run report");
    let statements = result["statements"].as_array().expect("statements");
    assert_eq!(statements.len(), 3, "one entry per statement");
    assert_eq!(statements[0]["line"], json!(0));
    assert!(
        statements[0]["display"].as_str().unwrap_or("").contains("42"),
        "the assignment's answer shows: {:?}",
        statements[0]["display"]
    );
    assert_eq!(
        statements[1]["display"], json!(null),
        "a def produces no value"
    );
    assert!(!statements[2]["error"].as_bool().unwrap_or(true), "the graph line plots");
    let svgs = result["svgs"].as_array().expect("svgs");
    assert_eq!(svgs.len(), 1, "one 2D graph document");
    assert!(svgs[0].as_str().unwrap().contains("<svg"), "an SVG document");
    Ok(())
}

#[test]
fn run_survives_a_failing_statement() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let uri = open(
        &mut client,
        "memo://run-error.epher",
        "x = 40 + 2\nnope + 1\ny = 1 + 1\n",
    );
    let _ = client.diagnostics();
    let id = client.request(
        "epher/run",
        json!({ "textDocument": { "uri": uri.to_string() } }),
    );
    let response = client.response(id);
    assert!(response.error.is_none(), "the run itself succeeds");
    let result = response.result.expect("run report");
    let statements = result["statements"].as_array().expect("statements");
    assert!(statements[1]["error"].as_bool().unwrap_or(false), "the bad statement is marked");
    assert!(
        statements[2]["display"].as_str().unwrap_or("").contains("2"),
        "evaluation continues past the failure"
    );
    // A failure is its own tier (ADR-0079): the pane presentation
    // marks the failed statement's lines as errors, never answers.
    let failed = &statements[1]["presentation"]["lines"];
    assert_eq!(failed[0]["kind"], json!("error"), "the failure tier");
    assert_eq!(statements[1]["presentation"]["answer"], json!(false));
    Ok(())
}

#[test]
fn run_carries_the_pane_presentation() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    // One knob echo, one labeled answer, a data loop that grids, a
    // long list that folds, and a trailing check: the run carries the
    // result pane's presentation (ADR-0078) beside the verbatim
    // display (ADR-0069 amendment). The silent const (ADR-0080)
    // produces no display at all, so it cannot reach the pane.
    let uri = open(
        &mut client,
        "memo://present.epher",
        "const r = 5\nr\nprint(\"area:\", pi * r^2)\ndef row(k) = {k, 2 * k}\nfor k in 1 to 3 do row(k)\nbig = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24}\nbig\nprint(\"symmetric:\", 1 == 1)\n",
    );
    let _ = client.diagnostics();
    let id = client.request(
        "epher/run",
        json!({ "textDocument": { "uri": uri.to_string() } }),
    );
    let response = client.response(id);
    assert!(response.error.is_none(), "run succeeds: {:?}", response.error);
    let result = response.result.expect("run report");
    let statements = result["statements"].as_array().expect("statements");
    assert_eq!(statements.len(), 8, "one entry per statement");

    // The silent const binds without a display, and carries no
    // presentation: nothing for a pane to draw.
    assert_eq!(statements[0]["display"], json!(null));
    assert!(statements[0].get("presentation").is_none());

    // The knob echo is setup: dim, value voice, no answer.
    assert_eq!(statements[1]["presentation"]["lines"][0]["kind"], json!("setup"));
    assert_eq!(statements[1]["presentation"]["lines"][0]["text"], json!("= 5"));
    assert_eq!(statements[1]["presentation"]["answer"], json!(false));

    // The labeled result carries the answer (the trailing check
    // points above itself), and the run names its source line.
    assert_eq!(statements[2]["presentation"]["lines"][0]["kind"], json!("labeled"));
    assert_eq!(statements[2]["presentation"]["answer"], json!(true));
    assert_eq!(result["answerLine"], json!(2));

    // The data loop's matrix grids: cells, column widths, `= ` prefix.
    let grid = &statements[4]["presentation"]["lines"][0]["grid"];
    assert_eq!(grid["cells"], json!([["1", "2"], ["2", "4"], ["3", "6"]]));
    assert_eq!(grid["widths"], json!([1, 1]));
    assert_eq!(grid["prefix"], json!("= "));

    // The 24-element list folds at 20 shown, 4 hidden.
    let list = &statements[6]["presentation"]["lines"][0]["list"];
    assert_eq!(list["elements"].as_array().map(Vec::len), Some(24));
    assert_eq!(list["hidden"], json!(4));

    // The trailing check reads as a verdict, never the answer.
    let check = &statements[7]["presentation"]["lines"][0];
    assert_eq!(check["kind"], json!("labeled"));
    assert_eq!(check["verdict"], json!(true));
    assert_eq!(statements[7]["presentation"]["answer"], json!(false));
    Ok(())
}

#[test]
fn run_reflects_the_debounced_text() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    let uri = open(&mut client, "memo://run-debounce.epher", "x = 1");
    let _ = client.diagnostics_within(2000);
    // An edit inside the quiet gap: the run must see the held text,
    // not the stale analyzed one.
    client.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri.to_string(), "version": 2 },
            "contentChanges": [ { "text": "x = 40 + 2" } ]
        }),
    );
    let id = client.request(
        "epher/run",
        json!({ "textDocument": { "uri": uri.to_string() } }),
    );
    let response = client.response(id);
    let result = response.result.expect("run report");
    assert!(
        result["statements"][0]["display"]
            .as_str()
            .unwrap_or("")
            .contains("42"),
        "the held text runs: {:?}",
        result["statements"][0]["display"]
    );
    Ok(())
}

#[test]
fn run_shows_tables_multiline() -> Result<(), Box<dyn Error>> {
    let mut client = start()?;
    // The table is the long, multi-line answer the results pane exists
    // for: its rendering keeps its newlines (ADR-0069).
    let uri = open(
        &mut client,
        "memo://table.epher",
        "table sin(x) from 0 to 3",
    );
    let _ = client.diagnostics();
    let id = client.request(
        "epher/run",
        json!({ "textDocument": { "uri": uri.to_string() } }),
    );
    let response = client.response(id);
    assert!(response.error.is_none(), "run succeeds: {:?}", response.error);
    let result = response.result.expect("run report");
    let statements = result["statements"].as_array().expect("statements");
    assert_eq!(statements.len(), 1);
    let display = statements[0]["display"].as_str().expect("table rendered");
    assert!(display.contains('\n'), "the table keeps its newlines: {display:?}");
    assert!(!statements[0]["error"].as_bool().unwrap_or(true));
    Ok(())
}
