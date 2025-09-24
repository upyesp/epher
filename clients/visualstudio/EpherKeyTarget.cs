// The F5/Ctrl+F5 key path (ADR-0069): the same keys the VS Code
// extension answers run the script over a .epher document in Visual
// Studio too. VS resolves a keystroke to the command it is bound to —
// the shell's own Debug.Start / Debug.StartWithoutDebugging — and then
// dispatches that command down the chain, so the key needs no command
// and no <KeyBinding> of ours. This target registers itself at the
// FRONT of the shell's command chain through
// IVsRegisterPriorityCommandTarget and answers exactly those two
// commands before the debugger ever sees them, and only when the
// active text view is a .epher file.
//
// The <KeyBinding> attempt this replaces failed silently: VS only
// honors a key binding after it has queried the bound command, and a
// lazily loaded package's command stays invisible until its first
// query — so the first F5 on an epher file fell through to nothing
// (the 0.5.49 field report).
//
// Pass-through is the whole safety story: any other command group,
// any other command id, and those two debug commands over a non-epher
// document return OLECMDERR_E_NOTSUPPORTED, which sends the shell on
// to the next target in the chain — Start Debugging keeps working
// everywhere else. The shell routes every command through here, so
// the hot path is one Guid comparison and nothing else.

using System;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.OLE.Interop;

namespace Epher.VisualStudio
{
    internal sealed class EpherKeyTarget : IOleCommandTarget
    {
        // The shell's standard command set and the two debug commands
        // on it: VSStd97CmdID.Start = 295 (cmdidStart, Debug.Start, F5)
        // and VSStd97CmdID.StartNoDebug = 368 (cmdidStartNoDebug,
        // Debug.StartWithoutDebugging, Ctrl+F5), both verified against
        // the merged interop assembly and stdidcmd.h, not memory. The
        // set guid is VSConstants.GUID_VSStandardCommandSet97 =
        // {5EFC7975-14BC-11CF-9B2B-00AA00573819} (the initializer in
        // the interop and stdidcmd.h agree; it is not the ...57EF19
        // guid that keeps showing up in second-hand notes).
        private static readonly Guid StandardCommandSet97 = VSConstants.GUID_VSStandardCommandSet97;

        private const uint DebugStart = (uint)VSConstants.VSStd97CmdID.Start;

        private const uint DebugStartWithoutDebugging = (uint)VSConstants.VSStd97CmdID.StartNoDebug;

        // OLECMDERR_E_NOTSUPPORTED (docobj.h): OLE_E_LAST + 1 = 0x80040100
        // — the HRESULT that tells the shell to keep walking the command
        // chain. The merged interop carries no managed constant for it
        // (verified against the assembly), so it lives here.
        private const int NotSupported = unchecked((int)0x80040100);

        // supported + enabled; visible needs no bit — the OLECMDF enum
        // has no VISIBLE flag, visibility is the absence of
        // OLECMDF_INVISIBLE.
        private const uint SupportedAndEnabled =
            (uint)(OLECMDF.OLECMDF_SUPPORTED | OLECMDF.OLECMDF_ENABLED);

        private readonly EpherPackage package;

        internal EpherKeyTarget(EpherPackage package)
        {
            this.package = package;
        }

        // Menu and toolbar rendering ask this; the keystroke itself
        // goes straight to Exec.
        public int QueryStatus(ref Guid pguidCmdGroup, uint cCmds, OLECMD[] prgCmds, IntPtr pCmdText)
        {
            if (pguidCmdGroup == StandardCommandSet97 && cCmds == 1 && prgCmds != null && prgCmds.Length > 0)
            {
                if (IsEpherKey(prgCmds[0].cmdID) && EpherPackage.OverEpherScript())
                {
                    prgCmds[0].cmdf = SupportedAndEnabled;
                    return VSConstants.S_OK;
                }
            }
            return NotSupported;
        }

        public int Exec(ref Guid pguidCmdGroup, uint nCmdID, uint nCmdexecopt, IntPtr pvaIn, IntPtr pvaOut)
        {
            if (pguidCmdGroup == StandardCommandSet97 && IsEpherKey(nCmdID) && EpherPackage.OverEpherScript())
            {
                // Exec arrives on the UI thread; the run leaves through
                // the same JoinableTask door the Tools-menu command
                // uses (EpherPackage.RunActiveScript — one run path,
                // no duplicated run logic).
                this.package.RunActiveScript();
                return VSConstants.S_OK;
            }
            return NotSupported;
        }

        private static bool IsEpherKey(uint cmdId)
        {
            return cmdId == DebugStart || cmdId == DebugStartWithoutDebugging;
        }
    }
}
