# Capturing the Visual Studio listing assets

These shots must come from **Visual Studio 2022 on Windows** running the
epher extension, the same way the VS Code listing's captures came from
real VSCodium. Do not reuse the VSCodium shots: the chrome, the theme,
and the menus must be unmistakably Visual Studio.

## One-time setup

1. Windows 10 or 11 with Visual Studio 2022 (17.x), any edition.
2. Install the extension: download `epher-visualstudio.vsix` from the
   v0.5.56 release page and double-click it (the VSIX Installer runs;
   restart Visual Studio if it was open).
3. Default dark theme (Visual Studio's own default: the shots should
   look like a fresh install), editor zoom 100%, default Consolas font.
4. Close every tool window (Solution Explorer, Error List, Output, Team
   Explorer) and dismiss any notification badges on the title bar.
5. Open `demo.epher` from this folder: File > Open > File. Every
   statement in it is real epher syntax, verified against the 0.5.56
   engine: the answers the shots need appear inline.

Optional: `setup.ps1` sizes the Visual Studio window to exactly
1244x900 at the top-left of the screen, to match the other listings'
frame size. Capturing the window itself (Snipping Tool's window mode)
gives the same result without the helper.

## The five shots (PNG) and one animation (GIF)

Name files exactly like the VS Code listing: `editor.png`,
`hover.png`, `completion.png`, `results.png`, `demo.gif`.

1. **editor.png** (the hero): the whole Visual Studio window with
   `demo.epher` open. Every statement must show its inline answer
   (the `= 6371 km` hints). If hints are missing: Visual Studio 17.x
   has an inlay-hints switch under Tools > Options > Text Editor >
   General; make sure it is enabled, then close and reopen the file.
   (The extension renders its answers through Visual Studio's inlay
   hint support.)
2. **hover.png**: the mouse over `legs` on the triangle line, with the
   hover signature (`legs: 3 m`) visible. Keep the tooltip fully inside
   the window.
3. **completion.png**: on the empty line at the end of the file, type
   `sq` and press Ctrl+Space, capturing with the completion list
   showing `sqrt` highlighted and its signature in the doc pane beside
   the list. (Delete the typed text after the shot, or undo it.)
4. **results.png**: after running (Debug > Start Debugging, or F5, or
   Tools > Run Epher Script): the results pane with the transcript and
   the 2D sine graph, plus the editor beside it. The graph must be
   fully visible.
5. **demo.gif** (the animation): in a copy of the file with only the
   comment line, type the three earth statements slowly (`const radius
   = 6371 km`, `const diameter = 2 * radius`, `circumference = pi *
   diameter`), pausing a beat
   after each so the inline answer appears, then press F5 and let the
   results pane open with the graph. Roughly ten seconds. ScreenToGif
   (free, portable) or ShareX both work: capture the window, 10 to 15
   frames per second, export at 800 pixels wide, keep it under 4 MB.

## Delivering them

The marketplace portal upload (the publish step) takes the images
directly, so the fastest path is: capture, then upload at publish time.
If you want them in the repo next to the VSCodium set instead, drop
the five files into this folder and say so; they will ride the next
train's commit.
