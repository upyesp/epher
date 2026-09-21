import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { registerDebug } from "./debug";
import { registerRun, RunPane } from "./results";
import { wasmServerOptions } from "./wasmServer";

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  // { log: true }: vscode-languageclient 10 types the client's
  // outputChannel as LogOutputChannel, the channel doubles as the
  // client's structured log.
  const channel = vscode.window.createOutputChannel("Epher", { log: true });
  context.subscriptions.push(channel);

  // Desktop and web share one server: the wasm module in the vsix
  // (ADR-0066 amendment). No download, no per-platform binaries. If
  // the wasm stack fails, editing degrades to the TextMate baseline
  // and says so once.
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
  // instead of regressing to the marketplace dialog. The results pane
  // handle lands once registerRun runs; until then F5 runs fine and
  // only points at the play command for graphs.
  let pane: RunPane | undefined;
  registerDebug(context, () => client, () => pane);
  try {
    await client.start();
    pane = registerRun(context, () => client);
  } catch (err) {
    client = undefined;
    const message = err instanceof Error ? err.message : String(err);
    channel.appendLine(`language server failed to start: ${message}`);
    void vscode.window.showErrorMessage(
      "The Epher language server could not start; highlighting and snippets still work. See the Epher output channel for details.",
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
