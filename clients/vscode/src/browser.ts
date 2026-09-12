import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/browser";
import { Wasm } from "@vscode/wasm-wasi/v1";
import { createStdioOptions, startServer } from "@vscode/wasm-wasi-lsp";

// The web entry (ADR-0066 amendment): the same server, compiled to
// wasm32-wasip1-threads and shipped inside the vsix. No download, no
// spawn — the browser cannot do either. wasm-wasi-core runs the module
// in its own workers and @vscode/wasm-wasi-lsp bridges the WASI pipes
// to LSP MessageTransports, which the browser client accepts from an
// async ServerOptions factory. If any of that fails (the dependency is
// missing, threads are unavailable), editing degrades to the TextMate
// baseline — the same contract as the desktop download failing.
let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const channel = vscode.window.createOutputChannel("Epher", { log: true });
  context.subscriptions.push(channel);

  const serverOptions: ServerOptions = async () => {
    const wasm = await Wasm.load();
    // The module rides the extension; workspace.fs reads it whether
    // the host is desktop or web. Same commit as the extension, so
    // the version skew the desktop downloader guards against cannot
    // happen here.
    const module = await wasm.compile(
      vscode.Uri.joinPath(context.extensionUri, "server", "epher-lsp.wasm"),
    );
    // Shared memory sized for script evaluation: 16 MiB to start, up
    // to 128 MiB. wasi-threads (the debounce) needs it shared.
    const process = await wasm.createProcess(
      "epher-lsp",
      module,
      { initial: 256, maximum: 2048, shared: true },
      {
        stdio: createStdioOptions(),
        mountPoints: [{ kind: "workspaceFolder" }],
      },
    );
    const decoder = new TextDecoder("utf-8");
    process.stderr?.onData((data) => channel.append(decoder.decode(data)));
    return startServer(process);
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ language: "epher" }],
    outputChannel: channel,
  };
  client = new LanguageClient(
    "epherLanguageServer",
    "Epher Language Server",
    serverOptions,
    clientOptions,
  );
  try {
    await client.start();
  } catch (err) {
    // The server is the whole live feature set; without it the
    // declarative contributions still edit fine. Say so once.
    client = undefined;
    const message = err instanceof Error ? err.message : String(err);
    channel.appendLine(`language server failed to start: ${message}`);
    void vscode.window.showErrorMessage(
      "The Epher language server could not start in the browser; highlighting and snippets still work. See the Epher output channel for details.",
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
