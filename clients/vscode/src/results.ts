import * as vscode from "vscode";

// The results pane (ADR-0069, amended): the language server's
// `epher/run` evaluates the whole script and the pane shows the
// output shaped by the result pane's presentation contract
// (ADR-0078): setup echo dim, labeled results at full weight, the
// answer emphasized, failures their own tier, matrices as grids,
// long lists folded behind a button; plus every plot the run
// produced as inline SVG. The verbatim display travels beside the
// presentation: copy always sends the transcript verbatim, and a
// server without a presentation (never shipped apart from this
// extension, but harmless) falls back to plain text rows.

export interface PaneGridData {
  prefix: string;
  cells: string[][];
  widths: number[];
  suffix: string;
}

export interface PaneListData {
  prefix: string;
  elements: string[];
  hidden: number;
  suffix: string;
}

export interface PaneLineData {
  text: string;
  kind: "setup" | "labeled" | "error";
  verdict?: boolean;
  unit?: string;
  grid?: PaneGridData;
  list?: PaneListData;
}

export interface PanePresentation {
  lines: PaneLineData[];
  answer: boolean;
}

export interface RunLine {
  line: number;
  source: string;
  display: string | null;
  error: boolean;
  presentation?: PanePresentation;
}

export interface RunReport {
  statements: RunLine[];
  svgs: string[];
  answerLine?: number | null;
}

/** The pane surface the run-only debug adapter (debug.ts) shares:
 *  every run lands here, whichever way it started, so the pane is
 *  the one place results and graphs render. */
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

// One display line's body (ADR-0078 R3/R6): a matrix renders as a
// right-aligned table, a folded list shows the first twenty elements
// behind a "+n more" button, a verdict wears its glyph, a trailing
// unit dims. `text` is already the pane's display text (prose drops
// the `= ` marker); the verbatim display stays on the row for copy.
function paneLineHtml(pane: PaneLineData): string {
  const glyph = pane.verdict === undefined ? "" : pane.verdict
    ? `<span class="v-glyph v-true" aria-hidden="true">✓</span>`
    : `<span class="v-glyph v-false" aria-hidden="true">✗</span>`;
  let body: string;
  if (pane.grid) {
    const head = escapeHtml(pane.grid.prefix);
    const rows = pane.grid.cells
      .map((row) => `<tr>${row.map((cell) => `<td>${escapeHtml(cell)}</td>`).join("")}</tr>`)
      .join("");
    const tail = escapeHtml(pane.grid.suffix);
    body = `${head}<table class="grid">${rows}</table>${tail}`;
  } else if (pane.list) {
    const shown = pane.list.elements.slice(
      0, pane.list.hidden > 0 ? pane.list.elements.length - pane.list.hidden : undefined,
    );
    const rest = pane.list.elements.slice(shown.length);
    const head = escapeHtml(pane.list.prefix);
    const tail = escapeHtml(pane.list.suffix);
    const more = pane.list.hidden > 0
      ? `<button type="button" class="more">+${pane.list.hidden} more</button>` +
        `<span class="rest" hidden>, ${rest.map(escapeHtml).join(", ")}</span>`
      : "";
    body = `${head}{ ${shown.map(escapeHtml).join(", ")}${more} }${tail}`;
  } else if (pane.unit && pane.text.endsWith(` ${pane.unit}`)) {
    const stripped = pane.text.slice(0, pane.text.length - pane.unit.length - 1);
    body = `${escapeHtml(stripped)}<span class="unit"> ${escapeHtml(pane.unit)}</span>`;
  } else {
    body = escapeHtml(pane.text);
  }
  return `<div class="pline pline-${pane.kind}">${glyph}<span class="line-body">${body}</span></div>`;
}

// One statement row: the pane's presentation when the server sent
// one, the verbatim display otherwise. The row keeps `data-line` for
// click-to-reveal and `title` for the verbatim text on hover.
function rowHtml(statement: RunLine, verbatimIndex: number | null): string {
  const title = statement.display === null ? "" : ` title="${escapeHtml(statement.display)}"`;
  const copy = verbatimIndex === null
    ? ""
    : `<button type="button" class="copy-btn" data-copy="${verbatimIndex}" title="Copy verbatim">⧉</button>`;
  if (!statement.presentation) {
    const answer = statement.error
      ? `<span class="error">${escapeHtml(statement.display ?? "error")}</span>`
      : `<span class="answer">${escapeHtml(statement.display ?? "")}</span>`;
    return `<div class="output" data-line="${statement.line}"${title}>${answer}${copy}</div>`;
  }
  const classes = ["output"];
  if (statement.presentation.answer) { classes.push("final"); }
  if (statement.error) { classes.push("error-row"); }
  const lines = statement.presentation.lines.map(paneLineHtml).join("");
  return `<div class="${classes.join(" ")}" data-line="${statement.line}"${title}>${lines}${copy}</div>`;
}

// The SVG documents the engine renders are self-contained and
// generated (never user input); inline them and let CSS size them to
// the pane. Only statements with output are rows: answers and errors,
// not the script re-read.
function reportHtml(report: RunReport, nonce: string): string {
  const verbatim: string[] = [];
  const rows = report.statements
    .filter((statement) => statement.display !== null)
    .map((statement) => {
      const index = verbatim.length;
      verbatim.push(statement.display ?? "");
      return rowHtml(statement, index);
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
  .output { cursor: pointer; padding: 2px 8px; white-space: pre-wrap; position: relative; }
  .output:hover { background: var(--vscode-list-hoverBackground); }
  .answer { color: var(--vscode-symbolIcon-functionForeground, #c8c8c8); }
  .error { color: var(--vscode-errorForeground); }
  /* The pane's tiers (ADR-0078): setup echo dims, the failure tier
     (ADR-0079) reads as a failure, the answer carries the weight. */
  .pline-setup { color: var(--vscode-descriptionForeground); }
  .pline-error { color: var(--vscode-errorForeground); font-weight: 500; }
  .output.final { font-weight: 600; }
  .output.final .line-body { font-size: 1.15em; }
  .v-glyph { display: inline-block; width: 1.6ch; }
  .v-true { color: var(--vscode-charts-green, green); }
  .v-false { color: var(--vscode-charts-red, red); }
  .unit { color: var(--vscode-descriptionForeground); }
  table.grid { display: inline-table; border-collapse: collapse; }
  table.grid td { text-align: right; padding: 0 10px 0 0; white-space: pre; }
  button.more {
    background: none; border: none; color: var(--vscode-descriptionForeground);
    cursor: pointer; font: inherit; padding: 0 4px; text-decoration: underline;
  }
  .copy-btn {
    visibility: hidden; background: none; border: none; cursor: pointer;
    color: var(--vscode-descriptionForeground); font-size: 0.9em;
    position: absolute; right: 4px; top: 2px;
  }
  .output:hover .copy-btn, .copy-btn:focus-visible { visibility: visible; }
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
  const verbatim = ${JSON.stringify(verbatim)};
  document.addEventListener("click", (event) => {
    const copy = event.target.closest(".copy-btn");
    if (copy) {
      event.stopPropagation();
      vscode.postMessage({ kind: "copy", text: verbatim[Number(copy.dataset.copy)] });
      return;
    }
    const more = event.target.closest(".more");
    if (more) {
      const rest = more.nextElementSibling;
      if (rest) { rest.hidden = false; }
      more.remove();
      return;
    }
    const row = event.target.closest(".output");
    if (row) { vscode.postMessage({ kind: "reveal", line: Number(row.dataset.line) }); }
  });
  // The pane opens on the answer (ADR-0078 R5), like the web and
  // terminal panes scroll to it.
  const final = document.querySelector(".output.final");
  if (final) { final.scrollIntoView({ block: "center" }); }
</script>
</body>
</html>`;
}

export function registerRun(
  context: vscode.ExtensionContext,
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
      (message: { kind: string; line?: number; text?: string }) => {
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
        // Copy always sends the transcript verbatim (ADR-0078's first
        // rule): the pane's shaped display never feeds a copy path.
        if (message.kind === "copy" && typeof message.text === "string") {
          void vscode.env.clipboard.writeText(message.text);
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

  context.subscriptions.push(
    vscode.commands.registerCommand("epher.runScript", async (uri?: vscode.Uri) => {
      // The CodeLens hands in the document's uri; the command palette
      // path falls back to the active editor. Untitled epher documents
      // run too: the server tracks them by uri string. Every start
      // takes the same road from here: the run-only debug session,
      // whose adapter sends `epher/run`, prints the transcript in the
      // Debug Console, and opens this pane beside the editor (see
      // debug.ts). Play button, Ctrl+Enter, F5, and Ctrl+F5 are then
      // one behaviour, not two.
      const target = uri ?? vscode.window.activeTextEditor?.document.uri;
      if (!target) {
        return;
      }
      await vscode.debug.startDebugging(undefined, {
        type: "epher",
        request: "launch",
        name: "Run epher script",
        program: target.toString(true),
      });
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
