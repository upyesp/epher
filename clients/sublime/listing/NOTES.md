# Sublime Text 4 capture notes (2026-09-29)

## Versions

- Sublime Text 4, build **4215** (official Linux x64 tarball from
  `https://download.sublimetext.com/sublime_text_build_4215_x64.tar.xz`,
  resolved as the newest amd64 build in
  `https://download.sublimetext.com/apt/stable/Packages`). The tarball
  unpacks to `sublime_text/` (not a bare binary), so the rig used
  `/tmp/st4/sublime_text/sublime_text`.
- LSP package **4070-2.14.0** (latest sublimelsp/LSP release).
- epher-lsp **0.5.57** (the clients' pinned version; built locally at
  `target/release/epher-lsp`, already present - not rebuilt).
- Xvfb :98 at 1920x1200x24 + openbox (decorations disabled for
  `sublime_text` via a custom rc.xml) so every visible pixel of chrome
  is Sublime's own; window sized to exactly 1244x900 with wmctrl.

## LSP install method that worked (fully headless)

GitHub releases of sublimelsp/LSP have **no assets** (checked the last
100 releases), so `LSP.sublime-package` was built from the release
tag's zipball and Package Control's libraries were fetched by hand:

1. `codeload.github.com/sublimelsp/LSP/zip/refs/tags/4070-2.14.0`,
   top-level dir stripped, rezipped as
   `~/.config/sublime-text/Installed Packages/LSP.sublime-package`.
2. LSP `dependencies.json` (for ST >= 4096) needs five libraries.
   Package Control's registry
   (`raw.githubusercontent.com/packagecontrol/channel/main/repository.json`,
   the `$schema`-4.0.0 *libraries* registry) resolves them to PyPI
   wheels / GitHub release wheels; all were downloaded and extracted
   into `~/.config/sublime-text/Lib/python314/` (ST 4215 runs modern
   plugins on its python 3.14 host):
   - mdpopups 5.1.2 - wheel from the
     `facelessuser/sublime-markdown-popups` GitHub release; it vendors
     jinja2/markupsafe/pygments/etc. under `mdpopups.third_party`,
     so nothing else is needed for popup rendering;
   - bracex 3.0.1, wcmatch 11.0.1, typing_extensions 4.16.0 - plain
     `py3-none-any` wheels from PyPI;
   - orjson 3.12.0 - `cp314-cp314-manylinux_2_17_x86_64` wheel from
     PyPI.
3. The package folder is installed as
   `~/.config/sublime-text/Packages/LSP-epher/` - **not** `Packages/epher`:
   the LSP package resolves the client config at
   `Packages/LSP-epher/LSP-epher.sublime-settings`, so the folder name
   must carry the `LSP-` prefix (clients/sublime/README.md documents
   this; it is load-bearing).
4. One extra file the repo copy lacks: `.python-version` containing
   `3.8`, same as real LSP-* packages (e.g. LSP-pyright). Without it
   the package's plugin loads on the legacy python 3.3 host where
   `from LSP.plugin import ...` fails (LSP itself runs on the modern
   host). With it, the console shows `reloading plugin
   LSP-epher.epher` + `plugins loaded` and the run command works.
5. `LSP-epher.sublime-settings` points `command` at
   `/home/pete/code/epher/target/release/epher-lsp`.

## Results command (exact)

- Command palette entry: **"epher: run this script"**
  (`LSP-epher.sublime-commands`), backed by the
  `lsp_epher_run` text command in plugin.py (ADR-0069 `epher/run`).
- Key chord ctrl+c ctrl+c (Linux/Windows) from
  the keymap the review then removed (see the review section).
- The results view ("epher run results", scratch, plain text) lists
  one row per statement (`L2 = 6371 km` ...) plus a Graphs section;
  the sin(x) graph was written to
  `~/.cache/sublime-text/Cache/epher/runs/run-<ts>-1.svg` and handed
  to `xdg-open` (no viewer on this headless box, so it errors into
  the console - harmless, the SVG file itself is written first).

## Quirks

- **Inlay hints (the inline answers) default off in the LSP package**
  (`"show_inlay_hints": false` in its sublime-package.json). Enable
  once via `Packages/User/LSP.sublime-settings`:
  `{ "show_inlay_hints": true }`. That alone was not enough on this
  rig: hints only rendered after running **LSP: Toggle Inlay Hints**
  once per window (the per-window `lsp_show_inlay_hints` flag drives
  the phantom set; the automatic refresh triggers did not fire at
  attach). editor/hover/completion/results all show the answers.
- In this LSP build (4070-2.14.0) newly typed lines do **not** get
  hints from typing or from ctrl+s either; the toggle command (or
  reopening the file) refreshes them. The demo.gif's final frame was
  taken after one console-invoked
  `window.run_command("lsp_toggle_inlay_hints", {"enable": True})`.
  The earlier lines' phantoms survive buffer clears and re-expand
  when identical text is retyped, which is why mid-gif frames already
  show answers on lines 1-4.
- The completion popup is plain ST auto-complete: docs render as the
  truncated italic detail column (`sqrt(q): square root; negative
  reals fal...`); there is no separate docs pane. Authentic UI, not
  a rig artifact.
- `xdotool type` swallows a leading `-` (parsed as an option) - use
  `xdotool type -- '-5, 6)'`.
- With a completion popup open, `xdotool key Return` accepts the
  selected completion instead of inserting a newline - send Escape
  first when scripting typing.
- `import -window <id>` fails with `Resource temporarily unavailable`
  on this GTK window; captures are `import -window root` cropped to
  the window rect (the window is undecorated and sits at +10+10, so
  the crop is pixel-identical to the window).
- Resizing the ST window with `xdotool windowsize` alone enlarges the
  X window but GTK never reflows (black band); `wmctrl -r ... -e`
  (EWMH, WM-driven) resizes correctly.
- ST's `on_hover` needs real motion events: teleporting the pointer
  with a single `xdotool mousemove` did not trigger hover popups;
  stepping the pointer in 2-3 moves does.
- No license/evaluation nag ever appeared during the session; no
  welcome tab either (launching with a file argument). Had one shown,
  Escape/dismiss was the plan; nothing to work around.

## Deliverables (clients/sublime/images/, all exactly 1244x900)

- `editor.png` - hero: canonical demo.epher, dark theme (ST default
  Mariana), every statement line carrying its inline answer.
- `hover.png` - hover on `disc` (line 16): popup with
  `disc(a, b, c): user-defined function` + `def disc(a, b, c) = b^2 - 4*a*c`.
- `completion.png` - `sq` typed at end of file: popup with sqrt /
  september_equinox / chisq_gof and their signature docs.
- `results.png` - the `epher run results` view after running
  "epher: run this script": L2..L18 transcript + graph SVG path.
- `demo.gif` - 10 frames, 50ms delay, loop: buffer cleared, 6 lines
  retyped live (radius -> diameter -> circumference -> in mile ->
  def disc -> disc(1, -5, 6)) with a frame after each, last frame
  with all inline answers.

## Review round (2026-09-29): what the LSP team's review changed

rchl, the LSP for Sublime Text maintainer, reviewed the channel PR
and asked for six things; all are in, and the package repo carries
them as `v0.5.58`:

- **`LspPlugin`, not `AbstractPlugin`**: `plugin.py` (renamed from
  `epher.py`, matching the peer packages) subclasses `LSP.plugin.
  LspPlugin` and registers with `EpherPlugin.register()/.unregister()`.
  The session name and the settings file now both come from the
  package name, so the explicit `name()` and `session_name = "epher"`
  are gone. The command overrides `is_enabled()` so the palette entry
  stays usable from the results view, which has no session.
- **No keymap**: the three `Default (<platform>).sublime-keymap` files
  are deleted. The bindings were ctrl+c ctrl+c (super+c super+c on
  macOS), which shadowed Copy. The README suggests `ctrl+alt+r`
  instead, verified unbound in ST 4215's default keymaps for Linux,
  macOS and Windows.
- **Settings shape**: the `"comment"` key became `//` comments,
  `priority_selector` (only meaningful with several servers on one
  view) is gone, and `schemes` gained `buffer`: epher-lsp keys its
  documents by URI and never reads a file path, so unsaved tabs are
  fully served.
- **Command naming**: the palette entry is `LSP-epher: Run Script`,
  naming the package like every other LSP helper package.
- **README truth pass**: the LSP package is documented as a manual
  install (Package Control cannot install one package from another;
  its dependencies are Python libraries), and semantic highlighting is
  documented with its real requirements: `"semantic_highlighting":
  true`, the color scheme rule for custom schemes, and the known LSP
  limitations.
- **Version**: `v0.5.57` was the pre-review revision and stays
  untouched; the reviewed revision is tagged `v0.5.58`, because
  Package Control reads tags and a listed release must be reviewed
  code. The next epher train re-syncs the package through
  `.github/workflows/sublime-publish.yml` as usual.
