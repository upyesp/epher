# Eclipse captures (`clients/eclipse/images/`) — how they were taken (2026-09-29)

All five deliverables are exactly **1244x900**, window-only captures of
Eclipse on display `:97` (Xvfb 1920x1200x24 + openbox, no compositor).
Nothing was scaled, composited, or retouched: the shell was resized to
exactly 1244x900 at the top-left of the screen and every shot is a
straight `import -window root -crop 1244x900+2+2` (the +2+2 accounts for
openbox's 2px gravity offset; the crop is the whole Eclipse window).

## IDE version and what was installed

- **Eclipse IDE for Eclipse Committers 2026-09** (`buildId
  4.41.0.20260903-0719`), linux-gtk-x86_64 tarball from
  `https://www.eclipse.org/downloads/packages/release/2026-09/r`,
  extracted to `/tmp/eclipse-captures/eclipse`. The IDE itself runs on
  the bundle's own JustJ **Java 25.0.4.1** (see `.metadata/.log`).
- **LSP4E 0.19.15.202608181341** and **TM4E 0.18.0.202608200919** are
  already part of the Committers package (both visible under
  `eclipse/plugins/` before the first launch), so **no p2 director
  install was performed** and no IU ids were needed. The README's
  "install LSP4E/TM4E if missing" fallback was not exercised.
- The epher bundle was built from `io.github.upyesp.epher.eclipse/`:
  `javac -nowarn -cp '/tmp/eclipse-captures/eclipse/plugins/*'` with
  Temurin **21.0.12.1+1** (`/tmp/jdk-21`, the bundle targets JavaSE-21)
  against the target platform's `ProcessStreamConnectionProvider`.
  **No source or import changes were needed** — the one class
  (`EpherConnectionProvider`) compiles as-is against LSP4E 0.19.15.
  The staged jar has `META-INF/MANIFEST.MF` (Bundle-Version bumped
  `0.0.0 -> 0.5.57`), `plugin.xml`, `syntaxes/epher.tmLanguage.json`
  and `language-configuration.json` from the VS Code client (the
  shared grammar), and the compiled class; dropped into
  `eclipse/dropins/io.github.upyesp.epher.eclipse.jar`.

## Launch and theme

- Xvfb `:97` + `openbox`; the Eclipse launcher was started with
  `PATH=/home/pete/code/epher/target/release:$JAVA_HOME/bin:$PATH` so
  the LSP4E server definition resolves `epher-lsp` from PATH. The
  server attached as soon as `demo.epher` opened — the Language Servers
  view during results.png shows `epher language server` with
  `PID 81860` and command line
  `/home/pete/code/epher/target/release/epher-lsp`, and inlay hints
  appear on every valued line (server 0.5.57 from this tree, not
  rebuilt — the binary already existed).
- Dark theme was pre-seeded before the first launch via the workspace
  preference
  `.metadata/.plugins/org.eclipse.core.runtime/.settings/org.eclipse.e4.ui.css.swt.theme.prefs`
  (`themeid=org.eclipse.e4.ui.css.theme.e4_dark`) and verified in
  *Window > Preferences > General > Appearance* (Theme: Dark). On
  this GTK rig the SWT trim (menubar/toolbar) only turns dark when
  the session also carries a dark **XSettings**: `xsettingsd` on `:97`
  with `Net/ThemeName "Adwaita-dark"` and
  `Gtk/ApplicationPreferDarkTheme "true"` (config scoped via the
  Eclipse process's `XDG_CONFIG_HOME`; `GTK_THEME=Adwaita:dark` alone
  was not enough). With that, the whole frame is Eclipse's own dark
  theme.

## What each deliverable shows

- `editor.png` — hero: canonical `demo.epher` (from the rig doc) open
  in Eclipse's Generic (LSP4E) editor, dark theme, inline answers
  (inlay hints) on every valued line: `6371 km`, `12742000 m`,
  `40030173.592 m`, `24873.5966904 mile`, `3 m`, `5 m`, `4 m`, `1`.
- `hover.png` — hover on `circumference` (line 7): popup
  "circumference: a variable in this document / = 40030173.592 m".
- `completion.png` — end of file, `sq` typed, Ctrl+Space: proposal
  list (`sigma_sb`, `satphen`, ...) with the documentation panel
  ("sigma_sb: Stefan-Boltzmann constant (W/(m²·K⁴))") and the
  incomplete-`sq` diagnostic marker on the line.
- `results.png` — **there is no run surface**: the bundle is a content
  type + LSP4E server definition + TM4E grammar, with no launch
  configuration or run command, so this is the closest authentic
  capability shot (rig doc's fallback). One frame showing both:
  *General > Content Types* preference page expanded to
  **Text > epher script** (file association `*.epher`, associated
  editors Generic Code Editor / Plain Text Editor) with, below the
  dialog, the **Language Servers view** listing the running
  `epher language server` (`epher-lsp`, `PID 81860`,
  `/home/pete/code/epher/target/release/epher-lsp`,
  `io.github.upyesp.epher.server`) and the answered `demo.epher`
  editor on the left.
- `demo.gif` — 12 frames, `convert -delay 50 -loop 0`: empty editor →
  the canonical demo's six opening lines typed live one per frame
  (answers appearing as each line lands) → settle → hovers on
  `radius`, `diameter`, `circumference` → final full view.

## Quirks worth knowing (rig lessons)

- **`pkill -x eclipse` only kills the launcher wrapper.** The IDE is a
  `java` child that survives and keeps the workspace lock (the next
  start shows "Workspace Cannot Be Locked" naming the PID). Kill the
  java PID (from the dialog or `pgrep -P <launcher> -x java`).
- The shared `language-configuration.json`'s comment rule makes Enter
  on a comment line auto-insert `// ` (authentic TM4E behaviour). The
  GIF removes that prefix with BackSpace before typing the next line,
  so no frames show it.
- The auto-completion popup **swallows the Return key** (accepts a
  proposal instead of inserting a newline). When scripting input,
  send Escape before Return.
- This train's first launch also opens "Sponsor the Eclipse IDE" and
  an "Install Java 27 Support" tab; both were dismissed (`Don't Ask
  Again`) before shooting.
- SWT on Xvfb is slow; each language-server reaction was given 1.2–3 s
  before the frame was taken so no shot shows a half-arrived answer.
