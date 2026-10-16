// The inline answers (ADR-0069): the language server computes an
// answer for every statement that produces a value and serves it as
// LSP inlay hints — "= 42" at the end of the statement's line. The
// VSSDK language-client framework never asks for inlay hints (the
// field report that started this: hover worked, answers never came),
// so this side asks: the tagger below drives textDocument/inlayHint
// over the session channel (hung by the language client), naming the
// document by the exact wire URI the framework itself opened it with
// (learned by the middle layer, EpherDocumentObserver) and carrying
// the text the server already holds. No duplicate documents, no
// second sync: the answers read the same session every other feature
// reads.

using System;
using System.Collections.Generic;
using System.ComponentModel.Composition;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Threading;
using Microsoft.VisualStudio.Text;
using Microsoft.VisualStudio.Text.Editor;
using Microsoft.VisualStudio.Text.Tagging;
using Microsoft.VisualStudio.Utilities;
using StreamJsonRpc;

namespace Epher.VisualStudio
{
    // The shared session channel, hung by the language client when the
    // framework attaches it. Taggers register here; when the channel
    // arrives (it can arrive after a view opened), every registered
    // tagger asks once so answers appear without further editing.
    internal static class EpherInlayHintChannel
    {
        private static readonly object Gate = new object();
        private static JsonRpc rpc;
        private static readonly List<EpherInlayHintTagger> Waiting =
            new List<EpherInlayHintTagger>();

        public static JsonRpc Rpc
        {
            get { lock (Gate) { return rpc; } }
        }

        public static void Attach(JsonRpc channel)
        {
            EpherInlayHintTagger[] toPrime;
            lock (Gate)
            {
                rpc = channel;
                toPrime = Waiting.ToArray();
                Waiting.Clear();
            }
            // The one line of field evidence for the answers: without
            // it, a silent attach and a broken channel look identical.
            EpherLog.Write(
                "the inlay-hint channel attached — inline answers will render as they arrive");
            foreach (var tagger in toPrime)
            {
                tagger.RequestHints();
            }
        }

        // A tagger with no channel yet: held until one shows up.
        public static void Await(EpherInlayHintTagger tagger)
        {
            JsonRpc arrived;
            lock (Gate)
            {
                if (rpc == null)
                {
                    Waiting.Add(tagger);
                    return;
                }
                arrived = rpc;
            }
            tagger.RequestHints();
        }
    }

    // The wire shape, matched by field name to what lsp-types
    // serializes (position, label as a plain string). Newtonsoft, the
    // formatter StreamJsonRpc pairs with here, populates public
    // fields.
    internal sealed class InlayHintPosition
    {
        public int line;
        public int character;
    }

    internal sealed class InlayHintRange
    {
        public InlayHintPosition start;
        public InlayHintPosition end;
    }

    internal sealed class InlayHintDto
    {
        public InlayHintPosition position;
        public string label;
    }

    [Export(typeof(IViewTaggerProvider))]
    [ContentType("epher")]
    [TextViewRole(PredefinedTextViewRoles.Document)]
    [TagType(typeof(IntraTextAdornmentTag))]
    internal sealed class EpherInlayHintTaggerProvider : IViewTaggerProvider
    {
        // One tagger per view: the answers belong to the document, but
        // the adornment elements belong to one visual tree, so a
        // split-pane second view gets its own tagger (and its own
        // request cycle; the server recomputes per ask either way).
        public ITagger<T> CreateTagger<T>(ITextView textView, ITextBuffer buffer) where T : ITag
        {
            return textView.Properties.GetOrCreateSingletonProperty(
                () => new EpherInlayHintTagger(buffer)) as ITagger<T>;
        }
    }

    internal sealed class EpherInlayHintTagger : ITagger<IntraTextAdornmentTag>
    {
        // The gray of an answer: readable on both shells' themes, and
        // clearly not code.
        private static readonly Brush AnswerBrush = CreateAnswerBrush();

        private static Brush CreateAnswerBrush()
        {
            var brush = new SolidColorBrush(Color.FromRgb(0x8a, 0x8a, 0x8a));
            brush.Freeze();
            return brush;
        }

        private const int DebounceMilliseconds = 400;

        private readonly ITextBuffer buffer;
        private readonly Dispatcher dispatcher;
        private readonly DispatcherTimer debounce;
        private readonly string documentPath;
        private readonly object gate = new object();

        // (tracking span at the answer's position, the rendered text)
        private List<KeyValuePair<ITrackingSpan, TextBlock>> entries =
            new List<KeyValuePair<ITrackingSpan, TextBlock>>();

        // The snapshot version the in-flight request was made against:
        // a result that comes back stale triggers one quiet re-request.
        private int requestedAtVersion = -1;

        // One line of per-document evidence, on the first response that
        // carries anything.
        private bool announcedFirstAnswer;

        internal EpherInlayHintTagger(ITextBuffer buffer)
        {
            this.buffer = buffer;
            this.dispatcher = Dispatcher.CurrentDispatcher;

            ITextDocument document;
            this.buffer.Properties.TryGetProperty(typeof(ITextDocument), out document);
            // The content type guarantees an ITextDocument (files open
            // from disk; the run path already relies on it). A missing
            // one simply means no answers.
            this.documentPath = document?.FilePath;

            this.debounce = new DispatcherTimer(DispatcherPriority.ApplicationIdle)
            {
                Interval = TimeSpan.FromMilliseconds(DebounceMilliseconds),
            };
            this.debounce.Tick += (sender, e) =>
            {
                this.debounce.Stop();
                this.RequestHints();
            };

            this.buffer.Changed += this.OnBufferChanged;

            if (this.documentPath != null)
            {
                EpherInlayHintChannel.Await(this);
            }
        }

        private void OnBufferChanged(object sender, TextContentChangedEventArgs e)
        {
            // Answers trail the typing by the debounce gap, the same
            // shape the server applies to diagnostics.
            this.debounce.Stop();
            this.debounce.Start();
        }

        internal void RequestHints()
        {
            if (this.documentPath == null)
            {
                return;
            }

            // The document must be one the framework opened: its wire
            // URI and its text come from the observed sync, not from
            // this side. Before the session starts (or for a file the
            // framework has not synced yet) there is nothing to ask.
            if (!EpherDocumentStore.TryGet(this.documentPath, out var wireUri, out var text))
            {
                return;
            }

            var snapshot = this.buffer.CurrentSnapshot;
            var version = snapshot.Version.VersionNumber;
            lock (this.gate)
            {
                if (version == this.requestedAtVersion)
                {
                    return;
                }
                this.requestedAtVersion = version;
            }

            var lastLine = snapshot.GetLineFromPosition(snapshot.Length);
            var range = new InlayHintRange
            {
                start = new InlayHintPosition { line = 0, character = 0 },
                end = new InlayHintPosition
                {
                    line = snapshot.LineCount - 1,
                    character = lastLine.Length,
                },
            };

            Task.Run(async () =>
            {
                try
                {
                    var channel = EpherInlayHintChannel.Rpc;
                    if (channel == null)
                    {
                        return;
                    }

                    var hints = await channel.InvokeWithParameterObjectAsync<List<InlayHintDto>>(
                        "textDocument/inlayHint",
                        new
                        {
                            textDocument = new { uri = wireUri },
                            range,
                        });

                    // Editing during the round trip: throw this result
                    // away, the queued debounce asks again.
                    if (this.buffer.CurrentSnapshot.Version.VersionNumber != version)
                    {
                        this.dispatcher.BeginInvoke(
                            new Action(this.RequestHints),
                            DispatcherPriority.ApplicationIdle);
                        return;
                    }

                    this.dispatcher.BeginInvoke(
                        new Action(() => this.ApplyHints(snapshot, hints)),
                        DispatcherPriority.ApplicationIdle);
                }
                catch (Exception ex)
                {
                    // A failing answer channel must never surface as an
                    // error dialog on a keystroke: answers are a bonus
                    // on top of the editor, not a dependency of it. One
                    // line lands in the diagnostics pane so the failure
                    // is a fact, not a silence.
                    EpherLog.Write("the inlay-hint request failed: " + ex.Message);
                }
            });
        }

        // Build the adornments on the UI thread and swap them in as a
        // whole: readers of GetTags see either the old answers or the
        // new ones, never a mix.
        private void ApplyHints(ITextSnapshot snapshotAtRequest, List<InlayHintDto> hints)
        {
            var fresh = new List<KeyValuePair<ITrackingSpan, TextBlock>>();
            var count = 0;
            if (hints != null)
            {
                foreach (var hint in hints)
                {
                    if (hint == null || hint.position == null || hint.label == null)
                    {
                        continue;
                    }

                    // The answer belongs after its statement. A hint at
                    // column 0 of a following line is the same fact in
                    // the server's other spelling — render it at the end
                    // of the previous line, because an adornment at a
                    // line's first column reads as an indent that cannot
                    // be moved (the 0.5.51.2 field report).
                    var line = snapshotAtRequest.GetLineFromLineNumber(
                        Math.Max(0, Math.Min(hint.position.line, snapshotAtRequest.LineCount - 1)));
                    if (hint.position.character <= 0 && line.LineNumber > 0)
                    {
                        line = snapshotAtRequest.GetLineFromLineNumber(line.LineNumber - 1);
                    }
                    var position = Math.Min(line.Start + line.Length, snapshotAtRequest.Length);

                    var span = snapshotAtRequest.CreateTrackingSpan(
                        position, 0, SpanTrackingMode.EdgePositive);
                    fresh.Add(new KeyValuePair<ITrackingSpan, TextBlock>(
                        span, MakeAdornment(hint.label)));
                    count++;
                }
            }

            if (count > 0 && !this.announcedFirstAnswer)
            {
                this.announcedFirstAnswer = true;
                EpherLog.Write("inline answers live: " + count + " answer(s) on the current script");
            }

            lock (this.gate)
            {
                this.entries = fresh;
            }
            this.RaiseTagsChanged();
        }

        private static TextBlock MakeAdornment(string label)
        {
            // paddingLeft on the wire becomes the leading space here:
            // the answer sits one space after the statement's last
            // character.
            return new TextBlock
            {
                Text = " " + label,
                FontFamily = new FontFamily("Consolas"),
                FontSize = 11.5,
                Foreground = AnswerBrush,
                Opacity = 0.9,
            };
        }

        private void RaiseTagsChanged()
        {
            var handler = this.TagsChanged;
            if (handler != null)
            {
                handler(this, new SnapshotSpanEventArgs(
                    new SnapshotSpan(this.buffer.CurrentSnapshot, 0,
                        this.buffer.CurrentSnapshot.Length)));
            }
        }

        public event EventHandler<SnapshotSpanEventArgs> TagsChanged;

        public IEnumerable<ITagSpan<IntraTextAdornmentTag>> GetTags(
            NormalizedSnapshotSpanCollection spans)
        {
            var current = this.buffer.CurrentSnapshot;
            var result = new List<ITagSpan<IntraTextAdornmentTag>>();
            lock (this.gate)
            {
                foreach (var entry in this.entries)
                {
                    var span = entry.Key.GetSpan(current);
                    result.Add(new EpherAdornmentTagSpan(span, new IntraTextAdornmentTag(
                        entry.Value, null)));
                }
            }
            return result;
        }

        private sealed class EpherAdornmentTagSpan : ITagSpan<IntraTextAdornmentTag>
        {
            public EpherAdornmentTagSpan(SnapshotSpan span, IntraTextAdornmentTag tag)
            {
                this.Span = span;
                this.Tag = tag;
            }

            public SnapshotSpan Span { get; }

            public IntraTextAdornmentTag Tag { get; }
        }
    }
}
