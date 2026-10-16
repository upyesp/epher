// The document observer: the middle layer that sits in the language
// client's message path and watches the document sync go by. The
// framework names each open script with its own wire URI and pushes
// the full text on every change (the server negotiates no incremental
// sync, so full text is the LSP default); the inline-answer tagger
// needs exactly those two facts — the URI spelling the server keys
// the document under, and the text it now holds — and the only sound
// way to get them is to observe the real traffic instead of guessing
// the spelling and duplicating the document (the 0.5.51.2 field
// report: duplicated documents, and hover gone dark).
//
// The layer is pure observation: CanHandle is true for the three
// document-sync notifications only, and every one of them is
// forwarded to the framework untouched, so the server sees exactly
// what it would have seen without us.

using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Microsoft.VisualStudio.LanguageServer.Client;
using Newtonsoft.Json.Linq;
using StreamJsonRpc;

namespace Epher.VisualStudio
{
    internal sealed class EpherDocumentObserver : ILanguageClientMiddleLayer
    {
        public bool CanHandle(string methodName)
        {
            return methodName == "textDocument/didOpen"
                || methodName == "textDocument/didChange"
                || methodName == "textDocument/didClose";
        }

        public async Task HandleNotificationAsync(
            string methodName, JToken methodParam, Func<JToken, Task> sendNotification)
        {
            try
            {
                switch (methodName)
                {
                    case "textDocument/didOpen":
                        EpherDocumentStore.RecordOpen(methodParam);
                        break;
                    case "textDocument/didChange":
                        EpherDocumentStore.RecordChange(methodParam);
                        break;
                    case "textDocument/didClose":
                        EpherDocumentStore.RecordClose(methodParam);
                        break;
                }
            }
            catch
            {
                // Observation must never break the message path: a
                // malformed sync costs the answers, not the editor.
            }

            await sendNotification(methodParam);
        }

        public Task<JToken> HandleRequestAsync(
            string methodName, JToken methodParam, Func<JToken, Task<JToken>> sendRequest)
        {
            // No request observation: requests ride the framework as if
            // this layer were not there.
            return sendRequest(methodParam);
        }
    }

    // The watched state, keyed by canonical local path so the editor
    // side (which knows a file path, not the wire spelling) can find
    // its document.
    internal static class EpherDocumentStore
    {
        private sealed class Entry
        {
            public string WireUri;
            public string Text;
        }

        private static readonly object Gate = new object();
        private static readonly Dictionary<string, Entry> Documents =
            new Dictionary<string, Entry>(StringComparer.OrdinalIgnoreCase);

        public static bool TryGet(string localPath, out string wireUri, out string text)
        {
            lock (Gate)
            {
                if (Documents.TryGetValue(CanonicalKey(localPath), out var entry))
                {
                    wireUri = entry.WireUri;
                    text = entry.Text;
                    return true;
                }
            }
            wireUri = null;
            text = null;
            return false;
        }

        internal static void RecordOpen(JToken parameters)
        {
            var uri = parameters?["textDocument"]?["uri"]?.ToString();
            var text = parameters?["textDocument"]?["text"]?.ToString();
            if (uri == null)
            {
                return;
            }
            lock (Gate)
            {
                Documents[CanonicalKey(uri)] = new Entry { WireUri = uri, Text = text ?? "" };
            }
        }

        internal static void RecordChange(JToken parameters)
        {
            var uri = parameters?["textDocument"]?["uri"]?.ToString();
            if (uri == null)
            {
                return;
            }
            // The server negotiates no incremental sync, so the last
            // content change carries the whole document. A change that
            // does not (a future framework change) leaves the stored
            // text alone rather than storing a fragment.
            var changes = parameters["contentChanges"];
            var last = changes?.Last;
            var text = last?["text"]?.ToString();
            lock (Gate)
            {
                if (text != null)
                {
                    if (Documents.TryGetValue(CanonicalKey(uri), out var entry))
                    {
                        entry.Text = text;
                    }
                    else
                    {
                        Documents[CanonicalKey(uri)] = new Entry { WireUri = uri, Text = text };
                    }
                }
            }
        }

        internal static void RecordClose(JToken parameters)
        {
            var uri = parameters?["textDocument"]?["uri"]?.ToString();
            if (uri == null)
            {
                return;
            }
            lock (Gate)
            {
                Documents.Remove(CanonicalKey(uri));
            }
        }

        // One spelling per file: the wire URI through Uri.LocalPath,
        // case-folded (Windows paths answer either case).
        private static string CanonicalKey(string uriOrPath)
        {
            try
            {
                if (uriOrPath.StartsWith("file:", StringComparison.OrdinalIgnoreCase))
                {
                    return new Uri(uriOrPath).LocalPath.ToLowerInvariant();
                }
            }
            catch
            {
                // fall through to the raw form
            }
            return uriOrPath.ToLowerInvariant();
        }
    }
}
