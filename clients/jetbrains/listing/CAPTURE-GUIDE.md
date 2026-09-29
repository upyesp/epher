# Capture guide: the JetBrains Marketplace screenshots (Linux Mint VM)

Four screenshots, all of the demo project that `setup.sh` prepares.
There is no animated GIF: the listing uses four static shots, matching
the pattern of .ignore (3 gallery images) and IdeaVim (text-only). The
JetBrains Marketplace stores these in the plugin's **Media** section,
where their listing docs put the minimum recommended size at
**1200 x 760** (1280 x 800 ideal).

**Size the window to clear that floor natively.** Raise the VM display
before shooting (for example 1600 x 1000 or larger) and maximize the
IDE, so the captured window is at least 1200 x 760 by itself. The
captures that shipped in 2026-09 were 901 x 568, below the floor, and
`make-media.sh` scales that set up to 1280 wide as a documented
fallback: it satisfies the marketplace, but a native capture is
crisper, and scaling is what the note here exists to avoid next time.

**Make every shot at the same IDE window size**, and capture the IDE
window only (no desktop, no other windows). Uniformity matters more
than any particular size; the four files must share identical
dimensions.

Take them in this order; each builds on the previous state.

## 0. Prepare

    ./setup.sh                      # once; downloads the IDE (~1.4 GB)
    "$HOME/ideaIU-captures/bin/idea.sh" "$HOME/epher-captures" &

Sign in (or start the free trial) when asked. Open `demo.epher` and
wait: the first open downloads the language server (a few MB, one
time), and then every statement shows its inline answer. Do not start
capturing until every line of `demo.epher` has its answer.

Dark theme (the default), editor font size 14 or larger (File,
Settings, Editor, Font), and the Project tree visible on the left: the
JetBrains look should be unmistakable.

Capture the window with Mint's Screenshot tool (choose "the window
that is under the cursor"), or:

    gnome-screenshot -w editor.png      # captures the focused window
    scrot -u editor.png                 # same, the current window

## 1. editor.png — the hero

What: `demo.epher` fully open, every line's answer inline, the
`circumference in mile` line showing `= 24,902.8 mile`.

- Click into the editor so it has focus, make sure no tooltip, popup,
  or notification banner is showing, and capture the IDE window.

## 2. hover.png — the signature

What: the mouse hovering `disc` on its `def` line (or the
`disc(1, -5, 6)` call), with the signature popup fully visible.

- Hover the name, count two seconds, then capture without moving.

## 3. completion.png — the catalog

What: the completion popup open at the end of `demo.epher`, rooted at
a fresh line reading `sq`, with `sqrt` highlighted and its
documentation pane showing the catalog entry.

- Click at the end of the last line, press Enter, type `sq`, wait a
  beat for the popup (Ctrl+Space forces it), capture with the popup
  and its documentation pane open.
- Press Escape afterwards and delete the line.

## 4. results.png — the run

What: the results tool window on the right with the transcript and the
`sin(x)` plot, after Tools, Run Epher Script.

- Run it once via Tools, Run Epher Script so the tool window exists.
- Capture the whole IDE window with the results pane open and the
  plot fully scrolled into view.

## Naming and hand-off

Save the four PNGs as `editor.png`, `hover.png`, `completion.png`,
`results.png` in one folder, then copy them to the build box:

    scp editor.png hover.png completion.png results.png \
        pete@192.168.10.114:/home/pete/code/epher/clients/jetbrains/listing/

They get uploaded to the plugin's Media section on
plugins.jetbrains.com once the listing exists.
