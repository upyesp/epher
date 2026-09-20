import * as vscode from "vscode";

// The run-only debug adapter: what F5 and Ctrl+F5 do on a .epher file.
//
// epher has nothing to debug — statements evaluate eagerly, in order,
// and the answers are the whole transcript — so a debug start is
// answered with the same whole-file run the results pane uses
// (ADR-0069): the language server's `epher/run`, streamed as DAP
// output events. The alternative VS Code offers without a debugger
// contribution ("find a debugging extension in the marketplace") sent
// users to a store that has no such extension, so the adapter exists
// to make the honest reading of F5 — "run it" — just work.
//
// The adapter is inline (no child process, no new dependencies) and
// identical on desktop and web, where the language server is the wasm
// build shipped inside the vsix. Only the DAP surface a run needs is
// implemented: initialize, launch, configurationDone, output,
// terminated, and polite refusals for everything a stepper would ask.

// The one client surface the adapter needs, same as the results pane
// (results.ts owns the `epher/run` request and exports the type).
import type { RunClient } from "./results";

/** A DAP message, typed loosely: the inline transport passes plain
 *  objects, and the run-only adapter touches a small, stable subset. */
type DapMessage = {
  seq?: number;
  type?: "request" | "response" | "event";
  command?: string;
  arguments?: { program?: string; breakpoints?: { line?: number }[] };
  request_seq?: number;
  success?: boolean;
  message?: string;
  event?: string;
  body?: unknown;
};

const REPL_HINT =
  "epher has nothing to pause or inspect: statements run in order and " +
  "the answers are the transcript. Edit the script and run again, or " +
  "use \u2018Epher: Run Script\u2019 for the results pane.";

class EpherRunSession implements vscode.DebugAdapter {
  private readonly messages = new vscode.EventEmitter<vscode.DebugProtocolMessage>();
  readonly onDidSendMessage = this.messages.event;

  private outgoingSeq = 0;
  private program: string | undefined;
  /** Set when the session ends before the run does; late output dies. */
  private ended = false;

  constructor(private readonly getClient: () => RunClient | undefined) {}

  handleMessage(message: vscode.DebugProtocolMessage): void {
    const msg = message as DapMessage;
    if (msg.type === "request") {
      this.onRequest(msg);
    }
    // Responses and events are not expected: the adapter never sends
    // requests, and VS Code posts no events over this transport.
  }

  dispose(): void {
    this.ended = true;
    this.messages.dispose();
  }

  private onRequest(request: DapMessage): void {
    switch (request.command) {
      case "initialize":
        this.respond(request, {
          supportsConfigurationDoneRequest: true,
          supportsTerminateRequest: true,
          supportsRestartRequest: false,
          supportsTerminateDebuggee: false,
          supportsEvaluateForHovers: false,
          supportsSetVariableRequest: false,
          supportsBreakpointLocationsRequest: false,
          supportsConditionalBreakpoints: false,
          exceptionBreakpointFilters: [],
        });
        this.event("initialized");
        break;
      case "launch":
        this.program = request.arguments?.program;
        this.respond(request);
        break;
      case "setBreakpoints":
      case "setFunctionBreakpoints":
        // Accepted and refused per breakpoint, so the editor's gutter
        // markers stay honest instead of silently pretending.
        this.respond(request, {
          breakpoints: (request.arguments?.breakpoints ?? []).map((breakpoint) => ({
            verified: false,
            line: breakpoint.line,
            message: "epher runs whole scripts; there is nothing to stop at",
          })),
        });
        break;
      case "setExceptionBreakpoints":
        this.respond(request, { breakpoints: [] });
        break;
      case "configurationDone":
        this.respond(request);
        void this.run();
        break;
      case "threads":
        this.respond(request, { threads: [{ id: 1, name: "epher" }] });
        break;
      case "evaluate":
        this.respond(request, { result: REPL_HINT, variablesReference: 0 });
        break;
      case "disconnect":
      case "terminate":
        this.respond(request);
        this.finish();
        break;
      default:
        this.fail(request, `not supported by the epher run adapter: ${request.command ?? "?"}`);
    }
  }

  /** The whole point: one `epher/run`, printed like the calculator
   *  prints. Mirrors the results pane's semantics in results.ts. */
  private async run(): Promise<void> {
    const client = this.getClient();
    if (!client) {
      this.output("The Epher language server is not running; the script run needs it.\n", "stderr");
      this.event("terminated");
      return;
    }
    if (!this.program) {
      this.output("No program to run: the launch configuration needs a \u2018program\u2019.\n", "stderr");
      this.event("terminated");
      return;
    }
    this.output(`Running ${this.program}\n`, "console");
    try {
      const report = await client.sendRequest("epher/run", {
        textDocument: { uri: toUriString(this.program) },
      });
      let errors = 0;
      for (const statement of report.statements) {
        this.output(`${statement.source}\n`);
        if (statement.error) {
          errors += 1;
          this.output(`${statement.display ?? "error"}\n`, "stderr");
        } else if (statement.display !== null) {
          this.output(`= ${statement.display}\n`);
        } else {
          this.output("skipped\n", "console");
        }
      }
      const total = report.statements.length;
      if (total === 0) {
        this.output("Nothing to run.\n", "console");
      } else {
        this.output(
          `${total} statement${total === 1 ? "" : "s"}`
            + (errors ? `, ${errors} with error${errors === 1 ? "" : "s"}` : "")
            + "\n",
          "console",
        );
      }
      if (report.svgs.length) {
        this.output(
          `${report.svgs.length} graph${report.svgs.length === 1 ? "" : "s"} produced`
            + " \u2014 \u2018Epher: Run Script\u2019 renders them beside the editor.\n",
          "console",
        );
      }
    } catch (err) {
      this.output(`Epher run failed: ${err instanceof Error ? err.message : String(err)}\n`, "stderr");
    }
    this.finish();
  }

  /** Ends the session exactly once, whether the run completed, failed,
   *  or the user pressed stop while it was in flight. */
  private finish(): void {
    if (!this.ended) {
      this.ended = true;
      this.event("terminated");
    }
  }

  private send(message: DapMessage): void {
    message.seq = ++this.outgoingSeq;
    this.messages.fire(message as vscode.DebugProtocolMessage);
  }

  private respond(request: DapMessage, body?: unknown): void {
    this.send({
      type: "response",
      request_seq: request.seq,
      command: request.command,
      success: true,
      body,
    });
  }

  private fail(request: DapMessage, message: string): void {
    this.send({
      type: "response",
      request_seq: request.seq,
      command: request.command,
      success: false,
      message,
    });
  }

  private event(name: string, body?: unknown): void {
    this.send({ type: "event", event: name, body });
  }

  private output(text: string, category = "stdout"): void {
    if (!this.ended) {
      this.event("output", { category, output: text });
    }
  }
}

/** A program argument arrives either as a filesystem path (a
 *  hand-written launch.json) or as a uri string (the synthesized
 *  configurations below); the server tracks documents by uri string. */
function toUriString(program: string): string {
  if (/^[a-z][a-z0-9+.-]*:/i.test(program)) {
    return program;
  }
  return vscode.Uri.file(program).toString();
}

/** Wires the debugger in. Both host entries call this next to
 *  registerRun; the pieces:
 *
 *  - the descriptor factory is what makes `type: "epher"` resolvable,
 *  - the Initial provider is the F5-just-works path: with no
 *    launch.json, VS Code hands over an empty configuration and a
 *    returned configuration starts immediately, no picker,
 *  - the Dynamic provider offers "Run epher script" in the Run and
 *    Debug dropdown (and in the Add Configuration flow). */
export function registerDebug(
  context: vscode.ExtensionContext,
  getClient: () => RunClient | undefined,
): void {
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory("epher", {
      createDebugAdapterDescriptor: () =>
        new vscode.DebugAdapterInlineImplementation(new EpherRunSession(getClient)),
    }),
    vscode.debug.registerDebugConfigurationProvider("epher", {
      resolveDebugConfiguration(_folder, config) {
        if (config?.type) {
          return config;
        }
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== "epher") {
          return config;
        }
        return {
          type: "epher",
          request: "launch",
          name: "Run epher script",
          program: editor.document.uri.toString(true),
        };
      },
    }, vscode.DebugConfigurationProviderTriggerKind.Initial),
    vscode.debug.registerDebugConfigurationProvider("epher", {
      provideDebugConfigurations() {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== "epher") {
          return [];
        }
        return [{
          type: "epher",
          request: "launch",
          name: "Run epher script",
          program: editor.document.uri.toString(true),
        }];
      },
    }, vscode.DebugConfigurationProviderTriggerKind.Dynamic),
  );
}
