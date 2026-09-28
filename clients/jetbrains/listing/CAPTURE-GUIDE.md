# Capture guide: the JetBrains Marketplace screenshots and GIF

Five captures, all of the demo project this folder's `setup.ps1`
prepares. Sizes follow the VS IDE captures: **2055 x 1370** for the
four screenshots (uniform, no browser chrome), and the GIF at the same
crop. The JetBrains Marketplace recommends images 600-800 px wide in
the description text; ours go in the **Media** gallery instead, where
larger images are fine and zoomable.

Take them in this order; each builds on the previous state.

## 0. Prepare

Run `setup.ps1` (see that file), then launch IntelliJ IDEA with the
demo project it created. Wait for the blue progress line at the bottom
to finish. The first time `demo.epher` opens, the plugin downloads the
language server (a few MB, a one-time pause); the inline answers
appear once it attaches. Do not start capturing until every statement
in `demo.epher` shows its inline answer.

Dark theme ("Dark" / the default), editor font size 14 or larger
(Settings, Editor, Font, Size), and the project tree visible on the
left: the JetBrains look should be unmistakable.

## 1. editor.png — the hero

What: `demo.epher` fully open, every line's answer inline, the
`circumference in mile` line showing `= 24,902.8 mile`.

- Open `demo.epher`, click into the editor, then press Ctrl+Shift+F12
  if you want more editor room (but keep the project tree visible).
- Make sure no tooltip, popup, or notification banner is showing.
- Capture the whole IDE window (Alt+PrtScn captures the focused
  window, or use Snipping Tool with a window snip).

## 2. hover.png — the signature

What: the mouse hovering `disc` on its `def` line (or the
`disc(1, -5, 6)` call), with the signature popup fully visible.

- Hover the name, count two seconds, then capture without moving.

## 3. completion.png — the catalog

What: the completion popup open in `demo.epher`, rooted at a fresh
line reading `sq`, with `sqrt` highlighted and its documentation pane
showing the catalog entry.

- Click at the end of the last line, press Enter, type `sq`, wait a
  beat for the popup (Ctrl+Space forces it), capture with the popup
  and its documentation pane open.
- Press Escape afterwards and delete the line.

## 4. results.png — the run

What: the results tool window on the right with the transcript and the
`sin(x)` plot, after Tools, Run Epher Script.

- Run it once via Tools, Run Epher Script so the tool window exists.
- Capture the whole window with the results pane open and the plot
  fully scrolled into view.

## 5. demo.gif — typing, live

What: a fresh line at the bottom of `demo.epher` being typed, the
inline answer appearing the moment the statement completes.

- Script (about 8-10 seconds at normal typing speed):
  1. Click at the end of the file, press Enter twice.
  2. Type: `60 mile/hr in m/s` — pause half a second after the `/`,
     pause again after `m/s`, and hold one full second after pressing
     Enter so the viewer sees the converted answer land.
- Record with the Xbox Game Bar (Win+G, capture the window) or
  OBS at 2055x1370, then export as GIF (15 fps is plenty; ezyzip.com,
  gifski, or ffmpeg: `ffmpeg -i in.mp4 -vf "fps=15,scale=1400:-1" out.gif`).
  Aim for under 8 MB.
- Press Ctrl+Z enough times to remove the typed line afterwards.

## Naming and hand-off

Save the four PNGs and the GIF into this folder as
`editor.png`, `hover.png`, `completion.png`, `results.png`,
`demo.gif`, then scp them to the build box:

    scp pete@192.168.10.114:/home/pete/code/epher/clients/jetbrains/listing/*.png .
    scp pete@192.168.10.114:/home/pete/code/epher/clients/jetbrains/listing/demo.gif .

(or copy them the other way from the Windows side). They get uploaded
to the plugin's Media section on plugins.jetbrains.com once the
listing exists.
