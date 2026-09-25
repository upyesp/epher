// The command surface (ADR-0069): the "Run Epher Script" button,
// placed in the Tools menu and the code-window context menu, plus the
// shell plumbing around the run — the results report in the IDE's
// internal browser and the full transcript echo in the "epher" Output
// pane. An AsyncPackage, per the platform's load-time guidance, and a
// lazily loaded one: nothing here registers auto-load. The standard
// F5/Ctrl+F5/play-button path does not live in the package at all —
// it is the view command filter (EpherViewFilter.cs), a MEF piece the
// editor loads with the first .epher view — so the package only ever
// loads when a human picks the menu item.

using System;
using System.ComponentModel.Design;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.Shell.Interop;
using Microsoft.VisualStudio.TextManager.Interop;

namespace Epher.VisualStudio
{
    internal static class GuidList
    {
        // Fixed forever once shipped: they are part of the registry the
        // installer merges, and Epher.vsct's symbols pair with the
        // command set below.
        public const string PackageGuidString = "672BDABC-CE2C-4E12-A10C-107148209915";
        public const string CommandSetGuidString = "2F8AD3D6-EC4C-4A64-8984-68F686E0F058";

        // The Output window pane the transcript echoes to; fixed so
        // every run reuses the same pane instead of growing new ones
        // (EpherLog carries the same guid for the MEF-side writers).
        public const string OutputPaneGuidString = "0F420F6C-EB1B-4A0D-99AB-A37CE1DF6A19";
    }

    // The ids must match Epher.vsct's IDSymbol values.
    internal static class CommandIds
    {
        public const int RunScript = 0x0100;
    }

    [PackageRegistration(UseManagedResourcesOnly = true, AllowsBackgroundLoading = true)]
    [ProvideMenuResource("Menus.ctmenu", 1)]
    [Guid(GuidList.PackageGuidString)]
    public sealed class EpherPackage : AsyncPackage
    {
        protected override async Task InitializeAsync(
            CancellationToken cancellationToken, IProgress<ServiceProgressData> progress)
        {
            await base.InitializeAsync(cancellationToken, progress);

            // Menu commands join the UI thread: the OleMenuCommandService
            // is a UI-thread service.
            await this.JoinableTaskFactory.SwitchToMainThreadAsync(cancellationToken);

            var commandService = await this.GetServiceAsync(typeof(IMenuCommandService)) as OleMenuCommandService;
            if (commandService == null)
            {
                return;
            }

            var runCommandId = new CommandID(new Guid(GuidList.CommandSetGuidString), CommandIds.RunScript);
            var runCommand = new OleMenuCommand(RunActiveScript, runCommandId);
            runCommand.BeforeQueryStatus += OnBeforeQueryStatus;
            commandService.AddCommand(runCommand);
        }

        protected override void Dispose(bool disposing)
        {
            base.Dispose(disposing);
        }

        // Enabled (and visible) only over a .epher document: this drives
        // the Tools-menu and code-window context-menu items. The
        // standard F5/Ctrl+F5/play path does not ride on this command;
        // it is the view command filter's QueryStatus (EpherViewFilter).
        private void OnBeforeQueryStatus(object sender, EventArgs e)
        {
            var command = (OleMenuCommand)sender;
            string path;
            bool overEpher = ActiveEpherBuffer(out path) != null;
            command.Visible = overEpher;
            command.Enabled = overEpher;
        }

        private void RunActiveScript(object sender, EventArgs e)
        {
            RunActiveScript();
        }

        // The one run path: the menu command and the view command
        // filter both come through here. The caller runs on the UI
        // thread; RunAsync hands the work to a JoinableTask, whose
        // awaits resume on the main thread — so the shell calls after
        // the server round trip stay on the thread they need.
        internal static void RunActiveScript()
        {
            ThreadHelper.JoinableTaskFactory.RunAsync(RunActiveScriptAsync);
        }

        private static async Task RunActiveScriptAsync()
        {
            try
            {
                // The buffer text travels whole: the run reflects exactly
                // what the editor showed, saved or not.
                string path;
                var buffer = ActiveEpherBuffer(out path);
                if (buffer == null)
                {
                    return;
                }

                int lastLine;
                int lastColumn;
                buffer.GetLastLineIndex(out lastLine, out lastColumn);
                string text;
                buffer.GetLineText(0, 0, lastLine, lastColumn, out text);

                var report = await EpherRun.RunAsync(
                    EpherLanguageClient.BundledServerPath(), path, text);
                EchoTranscript(report);
                var resultsPath = EpherRun.WriteResultsHtml(path, report);
                NavigateToResults(resultsPath);
            }
            catch (Exception ex)
            {
                ShowRunError(ex);
            }
        }

        // The active document through the text manager, no DTE
        // dependency: the active view's buffer knows both its file name
        // and its text. Untitled buffers have no file name, and a run
        // needs a path to anchor the results report to. Returns null for
        // anything that is not a .epher file.
        internal static IVsTextLines ActiveEpherBuffer(out string path)
        {
            path = null;

            var textManager = ServiceProvider.GlobalProvider.GetService(typeof(SVsTextManager)) as IVsTextManager;
            if (textManager == null)
            {
                return null;
            }

            IVsTextView view;
            if (textManager.GetActiveView(0, null, out view) != VSConstants.S_OK || view == null)
            {
                return null;
            }

            IVsTextLines buffer;
            if (view.GetBuffer(out buffer) != VSConstants.S_OK || buffer == null)
            {
                return null;
            }

            // The buffer's file name comes from its persistence
            // interface, GetCurFile: the merged 17.x interop carries no
            // GetFileName on the text interfaces at all (verified
            // against the assembly, not the stale docs). Untitled
            // buffers answer an empty name, which the .epher gate
            // below rejects.
            uint formatIndex;
            ((IPersistFileFormat)buffer).GetCurFile(out path, out formatIndex);
            if (string.IsNullOrEmpty(path) || !path.EndsWith(".epher", StringComparison.OrdinalIgnoreCase))
            {
                path = null;
                return null;
            }

            return buffer;
        }

        // The gate every run claim shares — the menu item's
        // BeforeQueryStatus, the view filter's QueryStatus and Exec.
        internal static bool OverEpherScript()
        {
            string path;
            return ActiveEpherBuffer(out path) != null;
        }

        // The whole transcript echoes to the "epher" Output pane, not
        // only the errors: the pane is the plain-text home of the run
        // beside the results report.
        private static void EchoTranscript(RunReport report)
        {
            if (report == null || report.Statements == null)
            {
                return;
            }

            foreach (var statement in report.Statements)
            {
                if (statement.Display == null)
                {
                    continue;
                }
                EpherLog.Write(
                    "line " + statement.Line + ": "
                    + (statement.Error ? "error: " : "") + statement.Display);
            }
        }

        // The results open in the shell's internal browser, which lands
        // them as a dockable tab in the editor well like any document
        // (ADR-0069's Visual Studio shape; a true tool window remains
        // the named follow-up).
        private static void NavigateToResults(string resultsPath)
        {
            var browsingService = ServiceProvider.GlobalProvider.GetService(typeof(SVsWebBrowsingService)) as IVsWebBrowsingService;
            if (browsingService != null)
            {
                // VSNWB_ForceNew: each run gets its own tab rather than
                // hijacking whatever the browser frame last showed.
                IVsWindowFrame frame;
                browsingService.Navigate(new Uri(resultsPath).AbsoluteUri, (uint)__VSWBNAVIGATEFLAGS.VSNWB_ForceNew, out frame);
                return;
            }

            // No internal browser service (unusual shell flavors): the
            // system browser is the fallback.
            Process.Start(resultsPath);
        }

        // A failed run is a user-facing event: the status bar for the
        // glance, the Output pane for the full message.
        private static void ShowRunError(Exception ex)
        {
            try
            {
                var statusbar = ServiceProvider.GlobalProvider.GetService(typeof(SVsStatusbar)) as IVsStatusbar;
                if (statusbar != null)
                {
                    statusbar.SetText("epher: the run failed — " + ex.Message + " (see the epher Output pane)");
                }
            }
            catch
            {
                // The status bar failing is no reason to crash the
                // command; the pane below still tries.
            }

            EpherLog.Write("the run failed: " + ex.Message);
            EpherLog.Write(ex.ToString());
        }
    }
}
