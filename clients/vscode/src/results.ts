import * as vscode from "vscode";

// The results pane (ADR-0069): the language server's `epher/run`
// evaluates the whole script and the pane shows the output — every
// answer and every error, nothing else — plus every plot the run
// produced as inline SVG. Statements that print nothing are not rows:
// the pane is the script's output, not a re-reading of the script.
// Every row links back to its line.

export interface RunLine {
  line: number;
  source: string;
  display: string | null;
  error: boolean;
}

export interface RunReport {
  statements: RunLine[];
  svgs: string[];
}

/** The pane surface the run-only debug adapter (debug.ts) shares:
 *  after an F5 run produced graphs, they land in the same pane the
 *  play button uses, so both starts render identically. */
export interface RunPane {
  show(uri: vscode.Uri, report: RunReport): void;
}

// The only client surface the pane needs; both entries (desktop and
// web) hand in their LanguageClient without this module importing
// either host's constructor. The run-only debug adapter (debug.ts)
// shares the interface and the request.
export interface RunClient {
  sendRequest(method: string, params: unknown): Thenable<RunReport>;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

// The SVG documents the engine renders are self-contained and
// generated (never user input); inline them and let CSS size them to
// the pane. Only statements with output are rows: answers and errors,
// not the script re-read.
function reportHtml(report: RunReport, nonce: string): string {
  const rows = report.statements
    .filter((statement) => statement.display !== null)
    .map((statement) => {
      const answer = statement.error
        ? `<span class="error">${escapeHtml(statement.display ?? "error")}</span>`
        : `<span class="answer">${escapeHtml(statement.display ?? "")}</span>`;
      return `<div class="output" data-line="${statement.line}">${answer}</div>`;
    })
    .join("\n");
  const graphs = report.svgs
    .map((svg) => `<div class="graph">${svg}</div>`)
    .join("\n");
  const graphSection = report.svgs.length
    ? `<h2>Graphs</h2>\n${graphs}`
    : "";
  return `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8" />
<meta http-equiv="Content-Security-Policy"
      content="default-src 'none'; style-src 'unsafe-inline'; script-src 'nonce-${nonce}';" />
<style>
  body {
    font-family: var(--vscode-editor-font-family);
    color: var(--vscode-editor-foreground);
    background: var(--vscode-editor-background);
    padding: 8px 14px;
  }
  h2 { font-size: 1.05em; margin: 14px 0 6px; }
  .output { cursor: pointer; padding: 2px 8px; white-space: pre-wrap; }
  .output:hover { background: var(--vscode-list-hoverBackground); }
  .answer { color: var(--vscode-symbolIcon-functionForeground, #c8c8c8); }
  .error { color: var(--vscode-errorForeground); }
  .graph { margin: 10px 0; }
  .graph svg { width: 100%; height: auto; background: #ffffff; border-radius: 4px; }
  .empty { color: var(--vscode-descriptionForeground); font-style: italic; }
</style>
</head>
<body>
${
  rows || report.svgs.length
    ? rows
    : `<p class="empty">No output.</p>`
}
${graphSection}
<script nonce="${nonce}">
  const vscode = acquireVsCodeApi();
  document.addEventListener("click", (event) => {
    const row = event.target.closest(".output");
    if (row) { vscode.postMessage({ kind: "reveal", line: Number(row.dataset.line) }); }
  });
</script>
</body>
</html>`;
}

export function registerRun(
  context: vscode.ExtensionContext,
  getClient: () => RunClient | undefined,
): RunPane {
  const panels = new Map<string, vscode.WebviewPanel>();

  const panelFor = (uri: vscode.Uri, documentName: string): vscode.WebviewPanel => {
    const key = uri.toString();
    const existing = panels.get(key);
    if (existing) {
      existing.reveal(vscode.ViewColumn.Beside);
      return existing;
    }
    const panel = vscode.window.createWebviewPanel(
      "epherResults",
      `epher results — ${documentName}`,
      vscode.ViewColumn.Beside,
      { enableScripts: true },
    );
    panel.webview.onDidReceiveMessage(
      (message: { kind: string; line?: number }) => {
        if (message.kind === "reveal" && typeof message.line === "number") {
          void vscode.workspace.openTextDocument(uri).then((document) =>
            vscode.window
              .showTextDocument(document, { preview: true })
              .then((editor) => {
                const line = Math.min(message.line ?? 0, document.lineCount - 1);
                const range = new vscode.Range(line, 0, line, 0);
                editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
                editor.selection = new vscode.Selection(range.start, range.start);
              }),
          );
        }
      },
      null,
      context.subscriptions,
    );
    panel.onDidDispose(() => panels.delete(key), null, context.subscriptions);
    panels.set(key, panel);
    return panel;
  };

  const pane: RunPane = {
    show(uri: vscode.Uri, report: RunReport): void {
      const panel = panelFor(uri, uri.path.split("/").pop() ?? "script");
      const nonce = String(Math.random()).slice(2);
      panel.webview.html = reportHtml(report, nonce);
    },
  };

  const run = async (uri: vscode.Uri): Promise<void> => {
    const client = getClient();
    if (!client) {
      void vscode.window.showErrorMessage(
        "The Epher language server is not running; the script run needs it.",
      );
      return;
    }
    try {
      const report = await client.sendRequest("epher/run", {
        textDocument: { uri: uri.toString() },
      });
      pane.show(uri, report);
    } catch (err) {
      void vscode.window.showErrorMessage(
        `Epher run failed: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
  };

  context.subscriptions.push(
    vscode.commands.registerCommand("epher.runScript", async (uri?: vscode.Uri) => {
      // The CodeLens hands in the document's uri; the command palette
      // path falls back to the active editor. Untitled epher documents
      // run too: the server tracks them by uri string.
      const target = uri ?? vscode.window.activeTextEditor?.document.uri;
      if (!target) {
        return;
      }
      await run(target);
    }),
    vscode.languages.registerCodeLensProvider({ language: "epher" }, {
      provideCodeLenses(document) {
        const top = new vscode.Range(0, 0, 0, 0);
        return [
          new vscode.CodeLens(top, {
            title: "$(play) Run script",
            command: "epher.runScript",
            arguments: [document.uri],
          }),
        ];
      },
    }),
  );
  return pane;
}
