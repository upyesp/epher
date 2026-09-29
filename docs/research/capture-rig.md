# The capture rig (build box)

How to take authentic IDE screenshots and GIFs on this headless Debian 12
box. Proven live on 2026-09-29 with Zed rendering through software
Vulkan.

## The display

Xvnc (TigerVNC) or Xvfb both work now that Mesa is new enough. A server
is usually already running on :99 at 1920x1200x24:

    DISPLAY=:99 xdotool search --onlyvisible --name "."   # windows?

Start your own if needed (each capture task should use its own display
number to avoid clobbering parallel tasks):

    Xvfb :98 -screen 0 1920x1200x24 -nolisten tcp >/tmp/xvfb98.log 2>&1 &

Do NOT run xcompmgr (it is unnecessary and changes edges).

## Hard rules

- NEVER `pkill -f <string>` for anything whose name appears in your own
  command line: pkill matches your own shell and kills it (exit 143,
  everything silently dies). Use `pkill -x zed` (exact process name).
- Mesa must be >= 25.0.7 (installed from bookworm-backports). Old Mesa
  22 renders software-Vulkan apps as black windows.
- Zed needs `ZED_ALLOW_EMULATED_GPU=1` to skip the Unsupported GPU nag.
- The epher-cli and epher-lsp binaries build with
  `cargo build --release -p epher-cli -p epher-lsp` (version 0.5.57,
  matching the clients' pinned server version).
- Wait for the language server to attach before shooting: inline
  answers must be visible on every line, or the shot is a lie.

## Deliverables (all IDEs, matching the VS Code listing exactly)

Into `clients/<ide>/images/`:

- `editor.png` - the hero: demo.epher open, every line answered
- `hover.png`  - the signature/value popup visible on a user symbol
- `completion.png` - the completion popup open with documentation
- `results.png` - the run/results surface after executing the script
  (where the IDE/client has one; otherwise the closest authentic
  capability shot - say which in NOTES.md)
- `demo.gif` - the demo typed live, answers appearing; ~10-12 frames

Every file EXACTLY 1244x900 (match clients/vscode/images/editor.png).
GIF: `convert -delay 50 -loop 0 frame*.png demo.gif`, 12 frames-ish.

Capture window-only (scrot -u or import -window <id>), resize to
1244x900 if the window is bigger. IDE dark theme, default font size.
The IDE chrome must be the IDE's own (no others' shots, no fakes).

Input: `xdotool type --delay 40` and `xdotool key`. Between frames:
sleep, then `import -window <id> frameNN.png`.

## demo.epher (canonical, one per capture)

    // the earth, measured
    const radius = 6371 km
    const diameter = 2 * radius
    circumference = pi * diameter

    // the same numbers in miles
    circumference in mile

    // a right triangle
    const legs = 3 m
    const hypotenuse = 5 m
    height = sqrt(hypotenuse^2 - legs^2)

    // the discriminant
    def disc(a, b, c) = b^2 - 4*a*c
    disc(1, -5, 6)

    graph sin(x)

## Per-IDE notes (grows as captures land)

- **Zed**: installed at /home/pete/.local/bin/zed (stable, 0.2xx).
  Runs on :99 with `ZED_ALLOW_EMULATED_GPU=1 DISPLAY=:99 zed
  --foreground <file>`. The extension is in clients/zed (a Rust
  extension; it downloads epher-lsp v0.5.57 from GitHub releases on
  first activation - needs network, takes a minute). Install it as a
  dev extension (see `zed --help` for the dev-extension flag).
- **Sublime Text 4**: no install yet. The client is clients/sublime
  (epher.py on the LSP package + LSP-epher.sublime-settings + shared
  tmLanguage). LSP installs headless as an LSP.sublime-package from
  the sublimelsp/LSP GitHub releases into the ST Installed Packages
  dir. Results surface: epher.py's run command writes a results view.
