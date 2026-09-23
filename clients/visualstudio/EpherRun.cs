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
// hand over the server's stdio, with Content-Length framing written
// and read byte-exactly here. The JSON is JavaScriptSerializer's
// (System.Web.Extensions, a GAC assembly in every .NET Framework
// install): nothing NuGet-sourced rides the vsix for this, because a
// bundled StreamJsonRpc failed to load inside the VS process (the
// 0.5.48 field reports) — a framework-only transport cannot.

using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using System.Web.Script.Serialization;

namespace Epher.VisualStudio
{
    // One statement's line of the transcript, verbatim field names from
    // the server's run_request (crates/lsp): line, source, display,
    // error — display carries the answer or the error text (the "= 2"
    // voice included), and is null when the statement prints nothing.
    // The wire names are matched by hand in ParseReport; no serializer
    // attributes.
    public sealed class RunLine
    {
        public int Line;

        public string Source;

        public string Display;

        public bool Error;
    }

    // The `epher/run` result: the per-statement transcript plus every
    // plot the run produced as self-contained SVG documents.
    public sealed class RunReport
    {
        public List<RunLine> Statements;

        public List<string> Svgs;
    }

    internal static class EpherRun
    {
        // Spawn a server, run the script, tear everything down. The
        // handshake is the standard one — initialize, initialized,
        // didOpen with the buffer's exact text so the run reflects what
        // the editor showed, then `epher/run` — followed by the
        // shutdown and exit that close a well-behaved LSP session.
        public static Task<RunReport> RunAsync(string scriptPath, string scriptText)
        {
            // The hand-rolled session blocks on its reads and writes:
            // keep that work off the thread the command starts on (the
            // UI thread), exactly as the async transport did.
            return Task.Run(() => Run(scriptPath, scriptText));
        }

        private static RunReport Run(string scriptPath, string scriptText)
        {
            // First use shares the language client's download cache: the
            // same matching server for this extension version
            // (ServerDownload, ADR-0066). A blocking wait is safe here:
            // this runs on a thread-pool thread, with no context for
            // the download's own awaits to need back.
            var serverPath = ServerDownload.EnsureServerAsync().GetAwaiter().GetResult();

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
                var session = new LspSession(process);
                try
                {
                    return session.Converse(scriptPath, scriptText);
                }
                catch (Exception ex)
                {
                    // Every run failure names what broke and, when the
                    // server said anything on stderr, ends with its last
                    // lines — future field reports carry the evidence
                    // with them.
                    throw new InvalidOperationException(ex.Message + session.StderrSuffix(), ex);
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

        // The transport: one LSP session over the process's stdio with
        // Content-Length framing, the same wire format the server
        // speaks. Requests and notifications serialize from
        // Dictionary<string, object> graphs; responses deserialize the
        // same way and are picked apart through the As* helpers below,
        // because JavaScriptSerializer hands back
        // Dictionary<string, object>, sequences (object[]), and numbers
        // as int or decimal.
        private sealed class LspSession
        {
            // The stderr lines kept for diagnostics: the server logs a
            // line or two per session, and the tail is the part that
            // names a crash.
            private const int StderrLines = 8;

            private readonly Stream input;   // the server's stdout
            private readonly Stream output;  // the server's stdin
            private readonly JavaScriptSerializer json;
            private readonly object stderrLock = new object();
            private readonly List<string> stderrLines = new List<string>();
            private readonly object writeLock = new object();
            private int nextId;

            internal LspSession(Process process)
            {
                this.input = process.StandardOutput.BaseStream;
                this.output = process.StandardInput.BaseStream;
                this.json = new JavaScriptSerializer();
                // The run result carries whole SVG documents; the 2 MB
                // default would reject a graph-heavy script.
                this.json.MaxJsonLength = int.MaxValue;

                // stderr is redirected and must be drained even though
                // nothing parses it: an unread pipe fills and blocks
                // the server mid-run. The drain runs on a background
                // thread (BeginErrorReadLine); the handler keeps the
                // last few lines for the failure message.
                process.ErrorDataReceived += this.OnStderrLine;
                process.BeginErrorReadLine();
            }

            // The conversation: initialize, initialized, didOpen,
            // `epher/run`, shutdown, exit — the handshake the run has
            // always driven, hand-framed now.
            internal RunReport Converse(string scriptPath, string scriptText)
            {
                var documentUri = new Uri(scriptPath).AbsoluteUri;
                var rootUri = new Uri(Path.GetDirectoryName(scriptPath)).AbsoluteUri;

                // The run session initializes like any editor session,
                // rooted at the script's directory.
                Request("initialize", new Dictionary<string, object>
                {
                    { "processId", null },
                    { "rootUri", rootUri },
                    { "capabilities", new Dictionary<string, object>() },
                });
                Notify("initialized", new Dictionary<string, object>());

                // didOpen carries the buffer's exact text so the run
                // reflects what the editor showed; the server runs
                // exactly this.
                Notify("textDocument/didOpen", new Dictionary<string, object>
                {
                    {
                        "textDocument",
                        new Dictionary<string, object>
                        {
                            { "uri", documentUri },
                            { "languageId", "epher" },
                            { "version", 1 },
                            { "text", scriptText },
                        }
                    },
                });

                var response = Request("epher/run", new Dictionary<string, object>
                {
                    { "textDocument", new Dictionary<string, object> { { "uri", documentUri } } },
                });

                Request("shutdown", null);
                try
                {
                    Notify("exit", new Dictionary<string, object>());
                }
                catch
                {
                    // The server exits on shutdown; a notification
                    // racing that exit fails harmlessly.
                }

                return ParseReport(response);
            }

            internal Dictionary<string, object> Request(string method, Dictionary<string, object> parameters)
            {
                nextId += 1;
                SendMessage(nextId, method, parameters);
                return WaitForResponse(nextId, method);
            }

            internal void Notify(string method, Dictionary<string, object> parameters)
            {
                SendMessage(null, method, parameters);
            }

            // The failure-message suffix: "; epher-lsp said: ..." with
            // the tail, or nothing when the server never spoke.
            internal string StderrSuffix()
            {
                lock (stderrLock)
                {
                    if (stderrLines.Count == 0)
                    {
                        return "";
                    }
                    return "; epher-lsp said: " + string.Join(" | ", stderrLines);
                }
            }

            private void OnStderrLine(object sender, DataReceivedEventArgs e)
            {
                if (e.Data == null)
                {
                    return; // end of stream, not a line
                }
                lock (stderrLock)
                {
                    stderrLines.Add(e.Data);
                    if (stderrLines.Count > StderrLines)
                    {
                        stderrLines.RemoveAt(0);
                    }
                }
            }

            private void SendMessage(int? id, string method, Dictionary<string, object> parameters)
            {
                var message = new Dictionary<string, object> { { "jsonrpc", "2.0" } };
                if (id.HasValue)
                {
                    message["id"] = id.Value;
                }
                message["method"] = method;
                if (parameters != null)
                {
                    message["params"] = parameters;
                }

                var payload = Encoding.UTF8.GetBytes(json.Serialize(message));
                var header = Encoding.ASCII.GetBytes("Content-Length: " + payload.Length + "\r\n\r\n");
                lock (writeLock)
                {
                    output.Write(header, 0, header.Length);
                    output.Write(payload, 0, payload.Length);
                    output.Flush();
                }
            }

            // Read framed messages until the response with our id shows
            // up. Everything else the server sends — today only
            // publishDiagnostics — is drained, because the run
            // conversation reads strictly in order.
            private Dictionary<string, object> WaitForResponse(int id, string method)
            {
                while (true)
                {
                    var fields = json.Deserialize<Dictionary<string, object>>(ReadMessage());
                    if (fields == null)
                    {
                        continue;
                    }
                    object methodValue;
                    if (fields.TryGetValue("method", out methodValue) && methodValue is string)
                    {
                        continue; // a server notification: not ours
                    }
                    object idValue;
                    if (!fields.TryGetValue("id", out idValue) || AsInt(idValue) != id)
                    {
                        continue; // not a response, or a stale one
                    }
                    object errorValue;
                    if (fields.TryGetValue("error", out errorValue) && errorValue != null)
                    {
                        throw new InvalidOperationException(method + " failed: " + ErrorText(errorValue));
                    }
                    return fields;
                }
            }

            // One framed message: ASCII headers through the blank line,
            // then exactly Content-Length bytes of UTF-8 JSON. The
            // header loop reads the raw stream byte by byte — a
            // buffered reader would steal bytes from the body.
            private string ReadMessage()
            {
                var contentLength = -1;
                while (true)
                {
                    var line = ReadHeaderLine();
                    if (line.Length == 0)
                    {
                        break;
                    }
                    const string lengthPrefix = "Content-Length:";
                    if (line.StartsWith(lengthPrefix, StringComparison.OrdinalIgnoreCase))
                    {
                        contentLength = int.Parse(
                            line.Substring(lengthPrefix.Length).Trim(),
                            CultureInfo.InvariantCulture);
                    }
                    // Any other header (Content-Type) is irrelevant here.
                }
                if (contentLength < 0)
                {
                    throw new EndOfStreamException("epher-lsp closed the stream before any message");
                }

                var payload = new byte[contentLength];
                var read = 0;
                while (read < contentLength)
                {
                    var count = input.Read(payload, read, contentLength - read);
                    if (count <= 0)
                    {
                        throw new EndOfStreamException("epher-lsp closed the stream mid-message");
                    }
                    read += count;
                }
                return Encoding.UTF8.GetString(payload);
            }

            private string ReadHeaderLine()
            {
                var line = new StringBuilder();
                while (true)
                {
                    var byteValue = input.ReadByte();
                    if (byteValue < 0)
                    {
                        throw new EndOfStreamException("epher-lsp closed the stream mid-header");
                    }
                    if (byteValue == '\n')
                    {
                        break;
                    }
                    if (byteValue != '\r')
                    {
                        line.Append((char)byteValue);
                    }
                }
                return line.ToString();
            }

            private static string ErrorText(object errorValue)
            {
                var error = errorValue as Dictionary<string, object>;
                if (error != null)
                {
                    object messageValue;
                    if (error.TryGetValue("message", out messageValue))
                    {
                        var message = messageValue as string;
                        if (message != null)
                        {
                            return message;
                        }
                    }
                }
                return "no detail";
            }

            // The `epher/run` result, picked apart by hand: statements
            // in server order, displays verbatim (the "= " voice stays —
            // the report shows the transcript as the server wrote it).
            private static RunReport ParseReport(Dictionary<string, object> response)
            {
                var report = new RunReport();
                object resultValue;
                if (response == null || !response.TryGetValue("result", out resultValue))
                {
                    return report;
                }
                var result = resultValue as Dictionary<string, object>;
                if (result == null)
                {
                    return report;
                }

                var statements = new List<RunLine>();
                object value;
                if (result.TryGetValue("statements", out value))
                {
                    foreach (var entry in AsList(value))
                    {
                        var fields = entry as Dictionary<string, object>;
                        if (fields == null)
                        {
                            continue;
                        }
                        var line = new RunLine();
                        object fieldValue;
                        if (fields.TryGetValue("line", out fieldValue))
                        {
                            line.Line = AsInt(fieldValue);
                        }
                        if (fields.TryGetValue("source", out fieldValue))
                        {
                            line.Source = fieldValue as string;
                        }
                        if (fields.TryGetValue("display", out fieldValue))
                        {
                            line.Display = fieldValue as string;
                        }
                        if (fields.TryGetValue("error", out fieldValue))
                        {
                            line.Error = AsBool(fieldValue);
                        }
                        statements.Add(line);
                    }
                }
                report.Statements = statements;

                var svgs = new List<string>();
                if (result.TryGetValue("svgs", out value))
                {
                    foreach (var entry in AsList(value))
                    {
                        var svg = entry as string;
                        if (svg != null)
                        {
                            svgs.Add(svg);
                        }
                    }
                }
                report.Svgs = svgs;
                return report;
            }

            // JavaScriptSerializer decodes JSON arrays as object[]
            // (readable as any IEnumerable) and JSON numbers as int or
            // decimal; every field read goes through one of these so
            // the wire's shapes are handled in exactly one place.
            private static List<object> AsList(object value)
            {
                var items = new List<object>();
                if (value == null || value is string)
                {
                    return items;
                }
                var sequence = value as IEnumerable;
                if (sequence == null)
                {
                    return items;
                }
                foreach (var item in sequence)
                {
                    items.Add(item);
                }
                return items;
            }

            private static int AsInt(object value)
            {
                if (value is int)
                {
                    return (int)value;
                }
                if (value is long)
                {
                    return (int)(long)value;
                }
                if (value is decimal)
                {
                    return (int)(decimal)value;
                }
                if (value is double)
                {
                    return (int)(double)value;
                }
                return 0;
            }

            private static bool AsBool(object value)
            {
                return value is bool && (bool)value;
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
