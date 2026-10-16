// The F5/Ctrl+F5 key path (ADR-0069), the pre-translation layer. The
// field has now buried every command-routing attempt — 0.5.49's
// <KeyBinding> entries, 0.5.50's priority command target (zero
// queries, proven by the key-path diagnostics), 0.5.51's bindings on
// an auto-loaded package, and 0.5.52's view command filter (in the
// chain per the diagnostics, but the shell never hands keystroke
// -resolved Debug.Start down when Start Debugging is disabled shell
// -wide, as it is with no solution open) — because all of them wait
// for the shell to hand them a command. This layer does not wait: it
// is an editor key processor, the pre-translation point where the
// physical key is seen before anything turns it into a command — the
// same layer Vim-emulation extensions are built on. A captured key is
// marked handled, so no binding table, no enablement state, and no
// command chain is ever consulted; the view command filter
// (EpherViewFilter.cs) stays as the standard layer behind it, and the
// two cannot double-run, because a key the processor marks handled
// never becomes a command.
//
// Export note, paid for in the field: the editor instantiates key
// processors only through IKeyProcessorProvider. The 0.5.52.1 attempt
// exported the bare KeyProcessor class and the editor never created
// it (the diagnostics line named the absence); the provider form is
// the documented extension point.

using System;
using System.ComponentModel.Composition;
using System.Windows.Input;
using Microsoft.VisualStudio.Text.Editor;
using Microsoft.VisualStudio.Utilities;

namespace Epher.VisualStudio
{
    [Export(typeof(IKeyProcessorProvider))]
    [Name("epher key processor provider")]
    [ContentType("epher")]
    [TextViewRole(PredefinedTextViewRoles.Editable)]
    internal sealed class EpherKeyProcessorProvider : IKeyProcessorProvider
    {
        // The content type and role filters already narrowed this call
        // to .epher views; the per-view evidence line lands here, at
        // the moment the editor actually asks for the processor.
        public KeyProcessor GetAssociatedProcessor(IWpfTextView view)
        {
            EpherLog.Write("the epher key processor joined a .epher view");
            return new EpherKeyProcessor();
        }
    }

    internal sealed class EpherKeyProcessor : KeyProcessor
    {
        // Plain F5 and Ctrl+F5 only: Shift/Alt+F5 (Restart, Stop
        // Debugging in some profiles) and every other chord pass
        // through untouched. The OverEpherScript gate is the same one
        // the Tools command's enablement uses, so every surface agrees
        // on what counts as a script.
        public override void KeyDown(KeyEventArgs args)
        {
            if (args.Key != Key.F5 || args.Handled)
            {
                return;
            }

            var modifiers = args.KeyboardDevice.Modifiers;
            if (modifiers != ModifierKeys.None && modifiers != ModifierKeys.Control)
            {
                return;
            }

            if (!EpherPackage.OverEpherScript())
            {
                return;
            }

            args.Handled = true;

            EpherLog.Write(
                "F5" + (modifiers == ModifierKeys.Control ? "+Ctrl" : "")
                + " captured by the epher key processor over a .epher document — running the script");
            EpherPackage.RunActiveScript();
        }
    }
}
