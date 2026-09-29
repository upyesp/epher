# Neovim captures (clients/nvim/images/)

Taken 2026-09-29 on the build box, display `:96` (Xvfb 1920x1200x24),
with **Neovim 0.11.4** (`nvim-linux-x86_64` tarball at
`/tmp/nvim-linux-x86_64/bin/nvim`) and the client's pinned server,
`target/release/epher-lsp` built from this tree (0.5.57).

Terminal-authentic: nvim inside an xterm window, xterm's own frame and
font handling, nvim's default dark colorscheme (nvim OSC-11-probes the
terminal background; the xterm was given a dark `-bg` so the probe picks
the dark default). Launch line:

```sh
LANG=C.UTF-8 xterm -fa 'DejaVu Sans Mono' -fs 13 \
  -geometry 113x41+2+2 -bg '#101010' \
  -title 'nvim ~/nvim-demo/demo.epher' \
  -e "env LANG=C.UTF-8 PATH=/tmp/nvim-rig-bin:/home/pete/code/epher/target/release:$PATH \
      /tmp/nvim-linux-x86_64/bin/nvim -u ~/.config/nvim-lua-demo/init.lua \
      ~/nvim-demo/demo.epher"
```

The 113x41 cell grid at this font size is a 1247x906 client area;
`scrot -u` window captures were resized to exactly 1244x900 (a <1%
downscale; every deliverable is 1244x900). Captures were taken with
`scrot -u` (focused window, no decorations) after `xdotool` input.

## init.lua used (`~/.config/nvim-lua-demo/init.lua`)

```lua
-- Demo config for the epher Neovim captures (clients/nvim/images/).
-- The minimal setup from clients/nvim/README.md, verbatim paths.
vim.opt.rtp:append("/home/pete/code/epher/clients/vim")
vim.opt.rtp:append("/home/pete/code/epher/clients/nvim")

require("epher").setup({
  cmd = { "/home/pete/code/epher/target/release/epher-lsp" },
})

-- nvim 0.11 renders completion item documentation in a floating
-- window when 'completeopt' carries "popup"; the client sets no
-- completion options of its own, so the demo opts in here to show
-- the docs pane the server provides.
vim.opt.completeopt:append("popup")
```

No client code was modified. nvim 0.11 takes the
`vim.lsp.config`/`vim.lsp.enable` branch in `lua/epher.lua`; inline
answers are nvim's native inlay hints (`vim.lsp.inlay_hint`), enabled
by the client's `FileType` autocmd. The input script is the canonical
`demo.epher` from `docs/research/capture-rig.md`, unmodified.

## What each shot shows

- **editor.png** - the hero. `demo.epher` open, the server attached,
  every answerable line carrying its inline answer (inlay hints):
  `= 6371 km`, `= 12742000 m`, `= 40030173.592 m`, `= 24873.5966904
  mile`, `= 3 m`, `= 5 m`, `= 4 m`, `= 1`; `graph sin(x)` has no
  inline answer (it produces the SVG in the results pane instead).
  Default dark colorscheme, default statusline, no popups.
- **hover.png** - cursor on `height` (`12,1`), `K` hover popup:
  `height: a variable in this document` and the current value `= 4 m`.
- **completion.png** - insert mode on a fresh line after the script,
  `sqrt` typed; `<C-x><C-o>` (nvim's `vim.lsp.omnifunc`) opened the
  completion popup. The item docs render in the popup's right column;
  `sqrt` is selected showing `Function sqrt(q): square root; negative
  reals fall back to complex; quantities need even dimen...`. Prefix
  `s` was used to get a menu (a unique prefix completes silently), then
  the selection was moved down to `sqrt`.
- **results.png** - `:EpherRun`: the client's results split (`:vsplit`,
  the default `pane = "split"`) with the transcript
  `L2 ... L18 graph: sin(x)` and the `Graphs (1, ...)` section naming
  the written SVG under `~/.cache/nvim/epher/runs/`. A graph row
  reopens the SVG with `<CR>`; the capture is the pane right after the
  run, focus in the results window. The graph itself is an SVG opened
  with the system viewer, not a text graph - the shared `epher/run`
  response has no text rendering, so this is the closest authentic
  surface for text-first nvim (the same one VS Code's pane uses for
  the transcript).
- **demo.gif** - the earth/miles/discriminant section typed live with
  `xdotool type --delay 40`, one frame captured after each line's
  answer landed, from an empty buffer (the results buffer was closed
  with `:bd!` first). 11 frames, `convert -delay 50 -loop 0`.

## Quirks (rig, not client)

- **Locale matters.** The box's default `LANG=C` makes xterm render
  the server's Unicode (en-dash, superscripts, Greek) as latin-1
  mojibake in the popup. xterm must be started with `LANG=C.UTF-8` in
  its own environment; the first UTF-8 attempt set it only on the nvim
  child, which is not enough.
- **No `xdg-open` on this box.** `:EpherRun` hands every written graph
  SVG to the system viewer; with no viewer present it warns. A no-op
  `xdg-open` shim (`/tmp/nvim-rig-bin/xdg-open`, exits 0) was put
  first in `PATH` so the run path stays quiet; the SVG is still
  written by the client and its path is what the shot shows.
- **Swap files.** Hard-killing nvim leaves a swap under
  `~/.local/state/nvim/swap/` (URL-encoded name); the next launch
  stops at the E325 prompt. The rig quits nvim cleanly (`:qa!`) and
  purges the swap dir; the E325 prompt ate early keystrokes in one
  aborted take.
- **Capture box runs parallel client tasks.** Two other captures (a
  Zed session and a Sublime Text session) were running the same
  `demo.epher` on other displays. `/tmp` paths with generic names are
  shared across tasks; one intermediate `results.png` was converted
  from another task's file after a path collision, was caught in the
  final inspection, and was re-converted from the verified capture.
  All final deliverables were visually verified individually.
- **xterm repaint lag.** Under rapid `xdotool` bursts, a `scrot`
  sometimes catches the pre-update frame; settles with a `Ctrl+L`
  redraw plus a two-snapshot check. All captures here were
  double-snapped (or re-snapped until stable) before leaving the rig.
- `:EpherRun`'s `:vsplit` semantics put the results window on the
  left and the script on the right (vim's `:vsplit` new-window side);
  that is the client's real layout, kept as captured.
