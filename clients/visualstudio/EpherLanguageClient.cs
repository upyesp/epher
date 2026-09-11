// The epher language client (ADR-0066): a thin shell that starts the
// shared epher-lsp binary over stdio and leaves every language question
// to it. Visual Studio discovers the MEF export when a .epher file
// opens (the content type wiring in EpherContentType.cs), calls
// OnLoadedAsync, and ActivateAsync hands it the server's streams.
//
// The download-and-spawn shape mirrors clients/vscode/src/download.ts;
// what differs is only the platform: Visual Studio runs on Windows, so
// this client always resolves the Windows zip.

using System;
using System.Collections.Generic;
using System.ComponentModel.Composition;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio.LanguageServer.Client;
using Microsoft.VisualStudio.Threading;
using Microsoft.VisualStudio.Utilities;

namespace Epher.VisualStudio
{
    [ContentType("epher")]
    [Export(typeof(ILanguageClient))]
    public class EpherLanguageClient : ILanguageClient
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

            // First use downloads the shared server (a few MB) and
            // caches it under LocalApplicationData; a failure throws,
            // and Visual Studio reports exceptions from this method in
            // an InfoBar. Editing keeps working through the TextMate
            // baseline either way.
            var command = await ServerDownload.EnsureServerAsync();

            var info = new ProcessStartInfo
            {
                FileName = command,
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
