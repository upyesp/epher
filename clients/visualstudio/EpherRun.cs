// The script run (ADR-0069): the command talks to the language server
// itself, one short-lived epher-lsp process per run, and transports
// the server's own `epher/run` request — execution lives in the server
// and this client stays a thin shell, exactly like the language-client
// session is. A fresh process per run keeps the run hermetic (its own
// shell state) and keeps it off the long-lived session the editor's
// inline features ride on.
//
// The download-and-spawn shape mirrors EpherLanguageClient.cs; what
// differs is the conversation: the run drives the LSP handshake by
// hand over JSON-RPC (StreamJsonRpc's default Content-Length framing
// is the same wire format the server speaks).

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;
using StreamJsonRpc;

namespace Epher.VisualStudio
{
    // One statement's line of the transcript, verbatim field names from
    // the server's run_request (crates/lsp): line, source, display,
    // error — display carries the answer or the error text, and is
    // null when the statement prints nothing.
    public sealed class RunLine
    {
        [JsonProperty("line")]
        public int Line;

        [JsonProperty("source")]
        public string Source;

        [JsonProperty("display")]
        public string Display;

        [JsonProperty("error")]
        public bool Error;
    }

    // The `epher/run` result: the per-statement transcript plus every
    // plot the run produced as self-contained SVG documents.
    public sealed class RunReport
    {
        [JsonProperty("statements")]
        public List<RunLine> Statements;

        [JsonProperty("svgs")]
        public List<string> Svgs;
    }

    internal static class EpherRun
    {
        // Spawn a server, run the script, tear everything down. The
        // handshake is the standard one — initialize, initialized,
        // didOpen with the buffer's exact text so the run reflects what
        // the editor showed, then `epher/run` — followed by the
        // shutdown and exit that close a well-behaved LSP session.
        public static async Task<RunReport> RunAsync(string scriptPath, string scriptText)
        {
            // First use shares the language client's download cache: the
            // same matching server for this extension version
            // (ServerDownload, ADR-0066).
            var serverPath = await ServerDownload.EnsureServerAsync();

            var start = new ProcessStartInfo
            {
                FileName = serverPath,
                Arguments = string.Empty,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true,
            };

            var process = new Process { StartInfo = start };
            if (!process.Start())
            {
                process.Dispose();
                throw new InvalidOperationException("failed to start " + serverPath);
            }

            try
            {
                using (var rpc = JsonRpc.Attach(process.StandardInput.BaseStream, process.StandardOutput.BaseStream))
                {
                    // stderr is redirected but nothing reads it by hand:
                    // the async drain keeps the server's log lines from
                    // filling the pipe and blocking the run (the same
                    // reason the JetBrains one-shot inherits stderr).
                    process.ErrorDataReceived += (sender, eventArgs) => { };
                    process.BeginErrorReadLine();

                    var documentUri = new Uri(scriptPath).AbsoluteUri;
                    var rootUri = new Uri(Path.GetDirectoryName(scriptPath)).AbsoluteUri;

                    // The run session initializes like any editor
                    // session, rooted at the script's directory.
                    await rpc.InvokeAsync<object>("initialize", new
                    {
                        processId = (string)null,
                        rootUri = rootUri,
                        capabilities = new { },
                    });
                    await rpc.NotifyAsync("initialized", new { });

                    // didOpen is a notification: no response waits, so
                    // it must not go through InvokeAsync. The buffer text
                    // travels whole; the server runs exactly this.
                    await rpc.NotifyAsync("textDocument/didOpen", new
                    {
                        textDocument = new
                        {
                            uri = documentUri,
                            languageId = "epher",
                            version = 1,
                            text = scriptText,
                        },
                    });

                    var report = await rpc.InvokeAsync<RunReport>("epher/run", new
                    {
                        textDocument = new { uri = documentUri },
                    });

                    await rpc.InvokeAsync<object>("shutdown", null);
                    try
                    {
                        await rpc.NotifyAsync("exit", new { });
                    }
                    catch
                    {
                        // The server exits on shutdown; a notification
                        // racing that exit fails harmlessly.
                    }

                    return report;
                }
            }
            finally
            {
                try
                {
                    if (!process.HasExited)
                    {
                        process.Kill();
                    }
                }
                catch
                {
                    // Already gone: nothing to clean up.
                }
                process.Dispose();
            }
        }

        // The results report: a self-contained HTML file under
        // %TEMP%\epher, overwritten per run, opened in the shell's
        // internal browser. The port of clients/vscode/src/results.ts
        // reportHtml(): only statements with output are rows, every row
        // anchors to its line, every SVG inlines. No CSS variables — a
        // fixed light palette plus a dark override for browsers that
        // report a dark scheme.
        public static string WriteResultsHtml(string scriptPath, RunReport report)
        {
            var dir = Path.Combine(Path.GetTempPath(), "epher");
            Directory.CreateDirectory(dir);
            var name = Path.GetFileNameWithoutExtension(scriptPath);
            var file = Path.Combine(dir, name + ".results.html");
            File.WriteAllText(file, ReportHtml(name, report), new UTF8Encoding(false));
            return file;
        }

        private static string EscapeHtml(string text)
        {
            return text
                .Replace("&", "&amp;")
                .Replace("<", "&lt;")
                .Replace(">", "&gt;")
                .Replace("\"", "&quot;");
        }

        private static string ReportHtml(string scriptName, RunReport report)
        {
            report = report ?? new RunReport();
            var statements = report.Statements ?? new List<RunLine>();
            var svgs = report.Svgs ?? new List<string>();

            // The pane is the script's output, not a re-reading of the
            // script: statements that print nothing are not rows.
            var rows = new List<string>();
            foreach (var statement in statements)
            {
                if (statement.Display == null)
                {
                    continue;
                }
                var span = statement.Error
                    ? "<span class=\"error\">" + EscapeHtml(statement.Display ?? "error") + "</span>"
                    : "<span class=\"answer\">" + EscapeHtml(statement.Display) + "</span>";
                rows.Add("<div class=\"output\" data-line=\"" + statement.Line + "\">" + span + "</div>");
            }
            var rowHtml = string.Join("\n", rows);

            var graphs = new List<string>();
            foreach (var svg in svgs)
            {
                graphs.Add("<div class=\"graph\">" + svg + "</div>");
            }
            // The SVG documents the engine renders are self-contained
            // and generated (never user input); inline them whole and
            // let CSS size them to the window.
            var graphSection = svgs.Count > 0
                ? "<h2>Graphs</h2>\n" + string.Join("\n", graphs)
                : "";

            // "No output." only when there was neither a row nor a graph.
            var body = (rowHtml.Length > 0 || svgs.Count > 0) ? rowHtml : "<p class=\"empty\">No output.</p>";

            var html = new StringBuilder();
            html.AppendLine("<!DOCTYPE html>");
            html.AppendLine("<html>");
            html.AppendLine("<head>");
            html.AppendLine("<meta charset=\"utf-8\" />");
            html.AppendLine("<title>epher results — " + EscapeHtml(scriptName) + "</title>");
            html.AppendLine("<style>");
            html.AppendLine("  body { font-family: Consolas, 'Courier New', monospace; color: #1f1f1f; background: #ffffff; padding: 8px 14px; }");
            html.AppendLine("  h2 { font-size: 1.05em; margin: 14px 0 6px; }");
            html.AppendLine("  .output { padding: 2px 8px; white-space: pre-wrap; }");
            html.AppendLine("  .output:hover { background: #f2f2f2; }");
            html.AppendLine("  .answer { color: #0b5cad; }");
            html.AppendLine("  .error { color: #a4262c; }");
            html.AppendLine("  .graph { margin: 10px 0; }");
            html.AppendLine("  .graph svg { width: 100%; height: auto; background: #ffffff; border-radius: 4px; }");
            html.AppendLine("  .empty { color: #616161; font-style: italic; }");
            html.AppendLine("  @media (prefers-color-scheme: dark) {");
            html.AppendLine("    body { color: #d4d4d4; background: #1e1e1e; }");
            html.AppendLine("    .output:hover { background: #2a2d2e; }");
            html.AppendLine("    .answer { color: #4fc1ff; }");
            html.AppendLine("    .error { color: #f48771; }");
            html.AppendLine("    .empty { color: #9d9d9d; }");
            html.AppendLine("    .graph svg { border: 1px solid #3c3c3c; }");
            html.AppendLine("  }");
            html.AppendLine("</style>");
            html.AppendLine("</head>");
            html.AppendLine("<body>");
            html.AppendLine(body);
            html.AppendLine(graphSection);
            html.AppendLine("</body>");
            html.AppendLine("</html>");
            return html.ToString();
        }
    }
}
