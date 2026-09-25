// The standard-commands hook (ADR-0069): a view-level command filter
// that claims the shell's own Debug.Start (F5) and
// Debug.StartWithoutDebugging (Ctrl+F5) over a .epher document — the
// commands the native Debug menu, the toolbar play button, and the
// keyboard all speak. Nothing is rebound: the menu items light up
// because this filter's QueryStatus answers SUPPORTED | ENABLED for
// exactly those two commands while a script is active, and Exec runs
// it; over every other file type the whole filter is a pass-through
// to the next target, so debugging keeps working untouched.
//
// Why this layer and not the shell-level one: the priority command
// target (EpherKeyTarget, still registered as the outside-the-editor
// backup) sits in the shell's chain, and the key-path diagnostics
// proved the shell never asks it about keystroke-resolved Debug.Start
// (0.5.50/0.5.51 field data: registered, zero queries). The view's
// command chain is different — it is the active-object chain OLE
// command routing walks first, and AddCommandFilter is its documented
// extension point. The key processor (EpherKeyProcessor) sits even
// earlier, on the raw keystroke; the two cannot double-run, because a
// key the processor marks handled never becomes a command.

using System;
using System.ComponentModel.Composition;
using System.Runtime.InteropServices;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Editor;
using Microsoft.VisualStudio.OLE.Interop;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.Text.Editor;
using Microsoft.VisualStudio.TextManager.Interop;
using Microsoft.VisualStudio.Utilities;

namespace Epher.VisualStudio
{
    [Export(typeof(IVsTextViewCreationListener))]
    [ContentType("epher")]
    [TextViewRole(PredefinedTextViewRoles.Editable)]
    internal sealed class EpherViewFilterProvider : IVsTextViewCreationListener
    {
        // Every .epher view gets the filter at the front of its
        // command chain; the view hands back the target that used to
        // be first, and everything this filter does not claim rides
        // there unchanged.
        public void VsTextViewCreated(IVsTextView textView)
        {
            var filter = new EpherViewFilter();
            IOleCommandTarget next;
            var added = textView.AddCommandFilter(filter, out next);
            filter.Next = next;
            // The field-diagnostic line that decides between "the
            // filter is in the chain and the shell never asks" and
            // "the filter never joined": without it the two failures
            // are indistinguishable from the outside.
            EpherLog.Write(added == VSConstants.S_OK
                ? "the command filter joined the view chain"
                : "the command filter was REFUSED by the view (0x"
                    + added.ToString("X8") + ")");
        }
    }

    internal sealed class EpherViewFilter : IOleCommandTarget
    {
        // The OLECMDERR_E_NOTSUPPORTED constant the merged interop
        // carries nowhere (verified against the assembly): "not mine",
        // for the rare case the chain has no next target to forward
        // to.
        private const int NotSupported = unchecked((int)0x80040100);

        // The target this filter displaced in the view's chain: the
        // pass-through for every command and every query this filter
        // does not claim.
        internal IOleCommandTarget Next { get; set; }

        private static readonly Guid StandardCommandSet97 =
            VSConstants.GUID_VSStandardCommandSet97;

        public int QueryStatus(
            ref Guid pguidCmdGroup, uint cCmds, OLECMD[] prgCmds, IntPtr pCmdText)
        {
            if (pguidCmdGroup == StandardCommandSet97
                && cCmds == 1
                && prgCmds != null
                && prgCmds.Length > 0
                && IsRunKey(prgCmds[0].cmdID)
                && EpherPackage.OverEpherScript())
            {
                // Claimed: the native Debug menu item, the toolbar play
                // button, and the key binding all read this state and
                // light up over a script.
                prgCmds[0].cmdf =
                    (uint)(OLECMDF.OLECMDF_SUPPORTED | OLECMDF.OLECMDF_ENABLED);
                return VSConstants.S_OK;
            }

            return Forward(ref pguidCmdGroup, cCmds, prgCmds, pCmdText);
        }

        public int Exec(
            ref Guid pguidCmdGroup, uint nCmdID, uint nCmdexecopt,
            IntPtr pvaIn, IntPtr pvaOut)
        {
            if (pguidCmdGroup == StandardCommandSet97
                && IsRunKey(nCmdID)
                && EpherPackage.OverEpherScript())
            {
                // The same one run path the Tools command uses; the
                // report lands in the results pane either way.
                EpherLog.Write(
                    "Debug.Start/StartNoDebug (" + nCmdID
                    + ") claimed by the epher view filter over a .epher document — running the script");
                EpherPackage.RunActiveScript();
                return VSConstants.S_OK;
            }

            return Forward(ref pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut);
        }

        private int Forward(
            ref Guid pguidCmdGroup, uint nCmdID, uint nCmdexecopt,
            IntPtr pvaIn, IntPtr pvaOut)
        {
            var next = this.Next;
            return next != null
                ? next.Exec(ref pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut)
                : NotSupported;
        }

        private int Forward(
            ref Guid pguidCmdGroup, uint cCmds, OLECMD[] prgCmds, IntPtr pCmdText)
        {
            var next = this.Next;
            return next != null
                ? next.QueryStatus(ref pguidCmdGroup, cCmds, prgCmds, pCmdText)
                : NotSupported;
        }

        // VSStd97CmdID.Start (F5, Debug.Start) and
        // VSStd97CmdID.StartNoDebug (Ctrl+F5, Start Without Debugging),
        // the pair the key target answers too.
        private static bool IsRunKey(uint cmdId)
        {
            return cmdId == (uint)VSConstants.VSStd97CmdID.Start
                || cmdId == (uint)VSConstants.VSStd97CmdID.StartNoDebug;
        }
    }
}
