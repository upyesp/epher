import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { ensureServer } from "./download";

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const channel = vscode.window.createOutputChannel("Epher");
  context.subscriptions.push(channel);

  // First run downloads the shared server for this platform and
  // caches it; a failure leaves editing to the TextMate baseline and
  // says so once.
  let command: string;
  try {
    command = await ensureServer(context, channel);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    channel.appendLine(`language server unavailable: ${message}`);
    void vscode.window.showErrorMessage(
      "The Epher language server could not be downloaded; see the Epher output channel for details.",
    );
    return;
  }

  const serverOptions: ServerOptions = { command, args: [] };
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
  await client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
