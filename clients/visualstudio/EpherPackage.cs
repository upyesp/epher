// The command surface (ADR-0069): a minimal VSPackage that owns the
// "Run Epher Script" button and the shell plumbing around the run —
// status bar, the "epher" Output pane, and the internal browser the
// results report opens in. A plain Package, not an AsyncPackage: the
// handler only reads the active view and hands the server round trip
// to a background task, so nothing here needs the async initialization
// contract, and plain Package is the oldest, most documented surface.
//
// The package itself stays dormant: CreatePkgDef turns the attributes
// below into the Packages and Menus registry entries at build time
// (GeneratePkgDefFile, and the VsPackage asset in the vsix manifest),
// while the hand-authored Epher.pkgdef keeps carrying the grammar
// entries. VS loads the package the first time the command runs.

using System;
using System.ComponentModel.Design;
using System.Diagnostics;
using System.Runtime.InteropServices;
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

        // The Output window pane run output echoes to; fixed so every
        // run reuses the same pane instead of growing new ones.
        public const string OutputPaneGuidString = "0F420F6C-EB1B-4A0D-99AB-A37CE1DF6A19";
    }

    // The ids must match Epher.vsct's IDSymbol values.
    internal static class CommandIds
    {
        public const int RunScript = 0x0100;
    }

    [PackageRegistration(UseManagedResourcesOnly = true)]
    [ProvideMenuResource("Menus.ctmenu", 1)]
    [Guid(GuidList.PackageGuidString)]
    public sealed class EpherPackage : Package
    {
        protected override void Initialize()
        {
            base.Initialize();

            var commandService = GetService(typeof(IMenuCommandService)) as OleMenuCommandService;
            if (commandService == null)
            {
                return;
            }

            var runCommandId = new CommandID(new Guid(GuidList.CommandSetGuidString), CommandIds.RunScript);
            var runCommand = new OleMenuCommand(RunActiveScript, runCommandId);
            runCommand.BeforeQueryStatus += OnBeforeQueryStatus;
            commandService.AddCommand(runCommand);
        }

        // Enabled only over a .epher document. The package loads lazily,
        // so VS serves the .ctmenu's static (enabled) state until the
        // first invocation loads the package; from then on this runs and
        // greys the command out over other file types.
        private void OnBeforeQueryStatus(object sender, EventArgs e)
        {
            var command = (OleMenuCommand)sender;
            string path;
            command.Enabled = ActiveEpherBuffer(out path) != null;
        }

        private void RunActiveScript(object sender, EventArgs e)
        {
            // The command handler runs on the UI thread; RunAsync hands
            // the work to a JoinableTask, whose awaits resume on the
            // main thread — so the shell calls after the server round
            // trip stay on the thread they need.
            ThreadHelper.JoinableTaskFactory.RunAsync(RunActiveScriptAsync);
        }

        private async Task RunActiveScriptAsync()
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

                var report = await EpherRun.RunAsync(path, text);
                EchoStatementErrors(report);
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
        private static IVsTextLines ActiveEpherBuffer(out string path)
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

            buffer.GetFileName(out path);
            if (string.IsNullOrEmpty(path) || !path.EndsWith(".epher", StringComparison.OrdinalIgnoreCase))
            {
                path = null;
                return null;
            }

            return buffer;
        }

        // Statement errors echo to the Output pane: the results tab
        // shows the output with the errors inline, and the pane keeps a
        // plain-text transcript of what broke.
        private void EchoStatementErrors(RunReport report)
        {
            if (report == null || report.Statements == null)
            {
                return;
            }

            foreach (var statement in report.Statements)
            {
                if (!statement.Error)
                {
                    continue;
                }
                GetOutputPane(new Guid(GuidList.OutputPaneGuidString), "epher")
                    .OutputString("line " + statement.Line + ": " + (statement.Display ?? "error") + Environment.NewLine);
            }
        }

        // The results open in the shell's internal browser, which lands
        // them as a dockable tab in the editor well like any document.
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
        private void ShowRunError(Exception ex)
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

            try
            {
                var pane = GetOutputPane(new Guid(GuidList.OutputPaneGuidString), "epher");
                pane.Activate();
                pane.OutputString("the run failed: " + ex.Message + Environment.NewLine);
                pane.OutputString(ex.ToString() + Environment.NewLine + Environment.NewLine);
            }
            catch
            {
                // Same story: shell reporting is best effort.
            }
        }
    }
}
