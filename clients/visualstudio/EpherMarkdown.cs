// The markdown half of ADR-0088: epher code fences inside a markdown
// document are a script. The language server owns the understanding:
// it scans the client's text for `epher`-tagged fences, projects them
// to one virtual script, and serves every feature in markdown
// coordinates (crates/lsp/src/markdown.rs). This file only answers
// the activation questions the editor side cannot delegate: should
// the run command light up over this buffer, and what languageId
// names the document's kind on the wire.
//
// .epher files always run. A .md/.markdown buffer runs when it
// carries at least one epher fence with content, so prose-only
// markdown never sees the run command or the debug keys.
//
// The fence grammar here mirrors the server's exactly (a deliberate
// copy, not a dependency, the same shape the VS Code family's
// runnableDocument carries): up to three leading spaces, three or
// more of one fence character (backtick or tilde), the info string's
// first word equal to `epher` case-insensitively, a backtick info
// string may not contain a backtick, a closing fence is the same
// character at least as long with nothing else on the line, and an
// unclosed fence runs to end of file. Only the *does an epher block
// exist* answer is needed, so no offsets are kept.

using System;
using System.Text.RegularExpressions;

namespace Epher.VisualStudio
{
    internal static class EpherMarkdown
    {
        private static readonly Regex FenceLine = new Regex(
            @"^[ ]{0,3}(`{3,}|~{3,})(.*)$",
            RegexOptions.Compiled | RegexOptions.CultureInvariant);

        private static readonly Regex CloseLine = new Regex(
            @"^[ ]{0,3}(`{3,}|~{3,})[ \t]*$",
            RegexOptions.Compiled | RegexOptions.CultureInvariant);

        internal static bool IsMarkdownPath(string path)
        {
            return path.EndsWith(".md", StringComparison.OrdinalIgnoreCase)
                || path.EndsWith(".markdown", StringComparison.OrdinalIgnoreCase);
        }

        // The wire languageId for a document path: "markdown" puts the
        // server in its markdown projection, "epher" is the script.
        internal static string LanguageIdFor(string path)
        {
            return IsMarkdownPath(path) ? "markdown" : "epher";
        }

        // Whether the text carries one epher fence with at least one
        // content line, the smallest projection the server would
        // build. Fence lines inside another (non-epher) fence are
        // content, not fence lines, so the walk tracks the enclosing
        // fence; and an epher fence never closed runs its content to
        // end of file.
        internal static bool HasEpherFence(string text)
        {
            if (text == null)
            {
                return false;
            }

            string fenceChar = null;
            int fenceLen = 0;
            bool inEpher = false;
            bool epherHasContent = false;

            foreach (var line in SplitLines(text))
            {
                if (fenceChar == null)
                {
                    var open = FenceLine.Match(line);
                    if (!open.Success)
                    {
                        continue;
                    }

                    var marker = open.Groups[1].Value;
                    var info = open.Groups[2].Value;
                    var ch = marker[0];
                    if (ch == '`' && info.Contains("`"))
                    {
                        continue;
                    }

                    fenceChar = ch.ToString();
                    fenceLen = marker.Length;
                    inEpher = FirstWord(info).Equals("epher", StringComparison.OrdinalIgnoreCase);
                    epherHasContent = false;
                }
                else if (IsClose(line, fenceChar[0], fenceLen))
                {
                    if (inEpher && epherHasContent)
                    {
                        return true;
                    }

                    fenceChar = null;
                    fenceLen = 0;
                    inEpher = false;
                }
                else if (inEpher)
                {
                    epherHasContent = true;
                }
            }

            return inEpher && epherHasContent;
        }

        // Whether a line closes the fence: at least as long a run of
        // the same fence character, nothing else on the line. (The
        // regex admits a different-character run only through the
        // marker[0] check below.)
        private static bool IsClose(string line, char fenceChar, int fenceLen)
        {
            var match = CloseLine.Match(line);
            if (!match.Success)
            {
                return false;
            }

            var marker = match.Groups[1].Value;
            return marker[0] == fenceChar && marker.Length >= fenceLen;
        }

        private static string FirstWord(string info)
        {
            var trimmed = info.Trim();
            var space = trimmed.IndexOfAny(new[] { ' ', '\t' });
            return space < 0 ? trimmed : trimmed.Substring(0, space);
        }

        // \r\n, \r, and \n all end a line, the same set the server's
        // line scan and the editor both treat as breaks.
        private static System.Collections.Generic.IEnumerable<string> SplitLines(string text)
        {
            return Regex.Split(text, "\r\n|\r|\n");
        }
    }
}
