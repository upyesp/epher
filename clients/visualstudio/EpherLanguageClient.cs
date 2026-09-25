// The epher language client (ADR-0066): a thin shell that spawns the
// epher-lsp binary that ships inside this extension and leaves every
// language question to it. Visual Studio discovers the MEF export
// when a .epher file opens (the content type wiring in
// EpherContentType.cs), calls OnLoadedAsync, and ActivateAsync hands
// it the server's streams.
//
// The server is bundled, not downloaded: the vsix carries
// server\epher-lsp.exe, built from the same commit as the extension,
// so the two cannot drift (the ADR-0066 amendment that retired the
// downloader for Visual Studio). If the binary is somehow missing,
// this throws and Visual Studio reports it in an InfoBar; editing
// keeps working through the TextMate baseline either way, the same
// contract as every other transport failure.
//
// The custom-message half of the interface (ILanguageClientCustomMessage2)
// serves the inline answers: the middle layer observes the document
// sync the framework sends (didOpen / didChange / didClose), learning
// the exact wire URI and latest text of every open script, and
// AttachForCustomMessageAsync hangs onto the session channel that the
// inlay-hint tagger asks on (EpherInlayHints.cs). The middle layer
// changes nothing: every message is forwarded untouched, so hover,
// diagnostics, and completion behave exactly as the framework's own.

using System;
using System.Collections.Generic;
using System.ComponentModel.Composition;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio.LanguageServer.Client;
using Microsoft.VisualStudio.Threading;
using Microsoft.VisualStudio.Utilities;
using Newtonsoft.Json.Linq;
using StreamJsonRpc;

namespace Epher.VisualStudio
{
    [ContentType("epher")]
    [Export(typeof(ILanguageClient))]
    public class EpherLanguageClient : ILanguageClient, ILanguageClientCustomMessage2
    {
        public string Name => "Epher";

        // No settings of our own (the server takes none); the docs make
        // null the way to say so.
        public IEnumerable<string> ConfigurationSections => null;

        public object InitializationOptions => null;

        public IEnumerable<string> FilesToWatch => null;

        public bool ShowNotificationOnInitializeFailed => true;

        public event AsyncEventHandler<EventArgs> StartAsync;

        // Never invoked: the server lives as long as Visual Studio does.
        // The interface declares the event, the docs' own sample does
        // not raise it.
        public event AsyncEventHandler<EventArgs> StopAsync;

        // The activation sequence from the docs: invoking StartAsync
        // here is what makes Visual Studio call ActivateAsync.
        public Task OnLoadedAsync()
        {
            return StartAsync.InvokeAsync(this, EventArgs.Empty);
        }

        public async Task<Connection> ActivateAsync(CancellationToken token)
        {
            await Task.Yield();

            var serverPath = BundledServerPath();
            if (!File.Exists(serverPath))
            {
                throw new FileNotFoundException(
                    "the bundled language server is missing from the extension folder: "
                    + serverPath);
            }

            var info = new ProcessStartInfo
            {
                FileName = serverPath,
                Arguments = string.Empty,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true,
            };

            var process = new Process { StartInfo = info };
            if (process.Start())
            {
                return new Connection(process.StandardOutput.BaseStream, process.StandardInput.BaseStream);
            }

            return null;
        }

        // The server rides the vsix: server\epher-lsp.exe beside this
        // assembly (the MEF parts load from the extension folder in
        // place, so Location names it).
        internal static string BundledServerPath()
        {
            var folder = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);
            return Path.Combine(folder ?? ".", "server", "epher-lsp.exe");
        }

        public object MiddleLayer => new EpherDocumentObserver();

        public object CustomMessageTarget => null;

        public Task AttachForCustomMessageAsync(JsonRpc customMessageRpc)
        {
            EpherInlayHintChannel.Attach(customMessageRpc);
            return Task.CompletedTask;
        }

        public Task OnServerInitializedAsync()
        {
            return Task.CompletedTask;
        }

        public Task OnServerInitializeFailedAsync(Exception e)
        {
            return Task.CompletedTask;
        }

        public Task<InitializationFailureContext> OnServerInitializeFailedAsync(
            ILanguageClientInitializationInfo initializationState)
        {
            // The default notification (ShowNotificationOnInitializeFailed)
            // already tells the user; nothing extra to add.
            return Task.FromResult<InitializationFailureContext>(null);
        }
    }
}
