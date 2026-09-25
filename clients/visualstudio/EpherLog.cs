// The diagnostics line (ADR-0069): every decision the run and answer
// paths make lands in the ActivityLog and the "epher" Output pane, so
// a field report can be read, not guessed. Deliberately package-free:
// the command filter and the answer tagger live in the MEF world,
// where no package instance exists to own the pane, and the pane is
// created on first use instead.

using System;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.Shell.Interop;

namespace Epher.VisualStudio
{
    internal static class EpherLog
    {
        // Fixed so every session reuses one pane (GuidList carries the
        // package-side copy of the same guid).
        private static readonly Guid PaneGuid =
            new Guid("0F420F6C-EB1B-4A0D-99AB-A37CE1DF6A19");

        private static bool paneCreated;

        public static void Write(string message)
        {
            _ = ThreadHelper.JoinableTaskFactory.RunAsync(async () =>
            {
                await ThreadHelper.JoinableTaskFactory.SwitchToMainThreadAsync();
                try
                {
                    var provider = ServiceProvider.GlobalProvider;
                    if (provider == null)
                    {
                        return;
                    }

                    try
                    {
                        (provider.GetService(typeof(SVsActivityLog)) as IVsActivityLog)?
                            .LogEntry(
                                (uint)__ACTIVITYLOG_ENTRYTYPE.ALE_INFORMATION,
                                "epher",
                                message);
                    }
                    catch
                    {
                        // The activity log failing is never worth
                        // breaking a keystroke over.
                    }

                    var outputWindow = provider.GetService(typeof(SVsOutputWindow)) as IVsOutputWindow;
                    if (outputWindow == null)
                    {
                        return;
                    }
                    if (!paneCreated)
                    {
                        var paneGuid = PaneGuid;
                        outputWindow.CreatePane(ref paneGuid, "epher", 1, 0);
                        paneCreated = true;
                    }
                    var paneId = PaneGuid;
                    outputWindow.GetPane(ref paneId, out var pane);
                    pane?.OutputString(message + Environment.NewLine);
                }
                catch
                {
                    // Same best-effort story: diagnostics never take the
                    // feature down with them.
                }
            });
        }
    }
}
