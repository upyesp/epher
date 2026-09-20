import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/browser";
import { registerDebug } from "./debug";
import { registerRun } from "./results";
import { wasmServerOptions } from "./wasmServer";

// The web entry (ADR-0066 amendment): the same server build as the
// desktop entry, compiled to wasm32-wasip1-threads and shipped inside
// the vsix. No download, no spawn; the browser cannot do either.
// ms-vscode.wasm-wasi-core runs the module in its own workers and
// @vscode/wasm-wasi-lsp bridges the WASI pipes to LSP
// MessageTransports, which the browser client accepts from an async
// ServerOptions factory. If any of that fails, editing degrades to
// the TextMate baseline, the same contract as desktop.
let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const channel = vscode.window.createOutputChannel("Epher", { log: true });
  context.subscriptions.push(channel);

  const serverOptions: ServerOptions = wasmServerOptions(context, channel);
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
  // Registered before the start attempt: the debug start exists even
  // when the server fails, so F5 reports the dead server cleanly
  // instead of regressing to the marketplace dialog.
  registerDebug(context, () => client);
  try {
    await client.start();
    registerRun(context, () => client);
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
