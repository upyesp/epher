import * as vscode from "vscode";
import { MessageTransports } from "vscode-languageclient";
import { Wasm } from "@vscode/wasm-wasi/v1";
import { createStdioOptions, startServer } from "@vscode/wasm-wasi-lsp";

// The language server, launched from the wasm module shipped inside
// the vsix. Both entries share this: desktop (main) and web (browser)
// run the identical server build (ADR-0066 amendment) — no download,
// no per-platform binaries, no first-use wait. The module rides the
// extension, so server and extension versions can never drift.
//
// Threading: the module is wasm32-wasip1-threads; ms-vscode.wasm-wasi-
// core (an install-time dependency, builtin on the web) provides the
// WASI host and maps wasi.thread-spawn to workers on every desktop
// platform and to web workers in the browser.

export function wasmServerOptions(
  context: vscode.ExtensionContext,
  channel: vscode.OutputChannel,
): () => Promise<MessageTransports> {
  return async () => {
    const wasm = await Wasm.load();
    // workspace.fs reads the module whether the host is desktop or
    // web. Shared memory sized for script evaluation: 16 MiB to
    // start, up to 128 MiB. wasi-threads (the debounce) needs it
    // shared.
    const module = await wasm.compile(
      vscode.Uri.joinPath(context.extensionUri, "server", "epher-lsp.wasm"),
    );
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
    process.stderr?.onData((data: Uint8Array) => channel.append(decoder.decode(data)));
    return startServer(process);
  };
}
