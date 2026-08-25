# ADR-0032: Dark Windows Launch, Slider Ends, 3D Spin Controls, Vertical Icon Rail, and the Bare Hero Command

Date: 2026-08-25

Status: Accepted

## Context

Five findings from using v0.4.19:

1. **Windows still flashes light on launch.** v0.4.19 shipped
   `--default-background-color=141416` as a WebView2 browser argument, but
   the window still opens white and turns dark a moment later. Linux shows
   no flash.
2. **Sliders never reach their ends.** Every `<input type="range">` in the
   app stops visibly short of both ends, so the user can't tell where the
   minimum and maximum are.
3. **The 3D rotation sliders are static.** A non-zero horizontal or
   vertical rotation value re-poses the plot once. Users expect a
   non-zero rotation to *rotate* — the horizontal slider spinning the
   graph around the vertical axis, the vertical slider around the
   horizontal axis, stopping at zero — with the mouse and arrow keys
   still available.
4. **The horizontal menu bar wastes a row.** A vertical icon rail on the
   left is the requested shape; top-level names become icons (Settings =
   a gear), the dropdowns stay words. "Open" should read "Load".
5. **The website hero shows a shell prompt.** The terminal card reads
   `$ graph3d x ^ 2 - y ^ 2`; the user wants the bare command.

## Analysis

### Windows light flash

Two layers paint on Windows: the tao window and the WebView2 surface.

- **The browser argument was invalid.** WebView2 parses
  `--default-background-color` as an AARRGGBB hex value (the format its
  own transparency documentation uses, e.g. `00000000`). The six-digit
  `141416` v0.4.19 passed is not a valid color, so WebView2 silently kept
  the white default — the fix did exactly nothing.
- **The window-level `backgroundColor` is a no-op on Windows.** Tao
  creates the window class with the default (white) background brush and
  only stores the configured color; unlike the Linux (GTK) path it never
  applies it. That is why Linux has never flashed: its window background
  is dark from the first frame. (v0.4.3's blank-window bug taught us to
  keep the tauri window config off Windows; nothing changed there — the
  color was simply never reaching the surface at all.)
- **The reliable mechanism is hidden-until-loaded.** `visible: false` on
  the Windows window plus a show at the first-paint signal means the very
  first presented frame is the already rendered page — the shell's inline
  CSS is dark from its first paint, and the boot-fallback (v0.4.2) is
  dark too. Even if WebView2 composited a white first frame, nobody can
  see it. The signal is the frontend's existing `init` IPC call, which
  fires right after Yew mounts (tauri exposes no page-load hook on the
  window handle in 2.11.x — the builder has one, the created window does
  not, and a WebView event-loop route doesn't exist either); the Rust
  `init` command shows and focuses the window on Windows. The
  boot-fallback script invokes `init` too when it takes over, so a
  broken wasm boot shows the dark fallback window instead of nothing.

Decision: keep the window hidden on Windows until the page load event,
show it then, and correct the browser argument to `FF141416` (opaque
dark, AARRGGBB) so the webview's own pre-CSS background is dark as well.
Linux/macOS keep their working `backgroundColor` overlays and visible
windows.

### Slider ends

The generic text-entry rule `input, textarea { padding: 0.75rem; border:
2px solid … }` also matched `type="range"` (and `type="checkbox"`). The
12px side padding and the border inset the thumb's travel, so the
minimum and maximum landed visibly inside the ends.

Decision: range and checkbox inputs take no text-input chrome — `padding:
0; border: none`. The thumb now travels edge to edge; the input's own
ends are the slider's min and max.

### Spinning rotation sliders

The fine-control sliders (ADR-0031) map onto `View3D::with_offsets`, a
static re-pose: horizontal adds a fixed yaw (h × π), vertical a fixed,
clamped pitch (v × 0.8, ±1.4), zoom scales the camera.

A spin is different in kind, not degree:

- It is *continuous*: while the slider is non-zero the pose advances. The
  advance must accumulate somewhere the orbit base cannot disturb —
  storing it in the base view would let a drag clamp the pitch back into
  ±1.4 and visibly snap a pose that had tumbled past the pole.
- It must *cross the poles*: "rotate around the horizontal axis" is a
  full revolution, so the vertical spin cannot honor the static pitch
  clamp. The projection is a pure spherical camera (yaw, then pitch,
  then a perspective divide); sine/cosine keep the pose continuous
  through the poles and the painter's-order renderer handles the camera
  passing under the plot. So the spin phase lives beside the orbit base:
  effective pose = base + phase (no clamp) + zoom offset.
- It must *stop at zero*: the phase freezes where the spin stopped, the
  sliders return to 0, and only a fresh 3D graph or Clear graph resets
  the phase — the same lifecycle as the slider values themselves.
- It is *motion*: under `prefers-reduced-motion` the spin loop never
  runs and the sliders keep their v0.4.19 static-offset meaning (WCAG
  2.3.3, consistent with the parameter play button's one-step behavior).

Decision: `View3D::with_spin_phase(yaw, pitch, zoom)` in the core — the
orbit base plus an unclamped accumulated rotation and the static zoom.
The web frontend keeps a live `(yaw, pitch)` phase cell advanced by a
single spawned loop at ~30fps while either rotation slider is non-zero
(full deflection ≈ one revolution per six seconds, dt-clamped so a
backgrounded tab cannot jump); the loop and the render both consult
`prefers-reduced-motion`. Orbit gestures still mutate the base, so the
mouse and arrow keys work during and after a spin. The TUI's Settings
rows keep their static ±0.1 nudges — they are adjustment rows, not
sliders, and a terminal has no continuous-render loop to host a spin
(the t-parameter play loop paces itself at 120ms and re-runs the
sampler; a spin there would thrash the grid).

### Vertical icon rail

The menubar becomes the app's left rail at the desktop breakpoint: the
topbar column carries it as a grid column beside the panes (`display:
grid` on `.epher`, `auto / minmax(0, 1fr)` columns), the bar itself
runs top-down as 44px square icon buttons — inline lucide-style stroke
SVGs in `currentColor` (file, pencil, gear, question circle), each with
the menu name as `aria-label` and native tooltip, `role="menubar"` with
`aria-orientation="vertical"`, dropdowns opening to the right with the
same word labels and keyboard reach as before. Mobile is untouched: the
rail is hidden below 880px and the hamburger remains the mobile menu.

The labels "Open history…" and "Open script…" become "Load history…"
and "Load script…" in all eight locales (the TUI's open prompt and its
failure message become "Load file:" / "Could not load the file" too —
the same words everywhere the file pickers are named).

### Hero code block

The terminal card in the landing hero shows the bare command,
`graph3d x ^ 2 - y ^ 2`, without a shell prompt; the `.terminal-dollar`
styling is removed with it.

## Decision

- Windows launches hidden and shows on the frontend's first-mount `init`
  call; the WebView2 background argument is the valid AARRGGBB
  `FF141416`.
- Range and checkbox inputs take no padding or border: the slider ends
  are the min and max.
- The horizontal and vertical rotation sliders spin the plot while
  non-zero via a core `View3D::with_spin_phase` and a single web spin
  loop; zero freezes the pose; orbit gestures keep working; reduced
  motion falls back to the static offsets.
- The desktop menu bar is a vertical icon rail on the left (dropdowns
  still words, opening rightward); mobile keeps the hamburger. "Open"
  becomes "Load" across the app and its eight locales.
- The hero terminal card shows `graph3d x ^ 2 - y ^ 2` with no prompt.

## Consequences

- The Windows first frame is dark twice over (hidden-until-loaded plus
  the corrected webview background); verified by the shipped binary
  carrying `--default-background-color=FF141416` and a merged window
  config with `visible: false` (the exe embeds the merged config).
- Slider thumbs travel edge to edge; the checkbox styling loses the
  padded box it accidentally inherited.
- The 3D plot spins at slider-driven speed and can tumble past the
  poles; the static pitch clamp no longer bounds what a spin can reach
  (it still bounds orbit drags and the reduced-motion offsets). Copy SVG
  exports the current spin pose, like any other pose.
- The rail trades one horizontal row for a 53px column — a net width
  win at every window size; icon-only top levels rely on aria-labels
  and tooltips for their names, and the menu names remain in the
  dropdowns, the mobile hamburger, and the TUI.
- One more rAF-paced loop runs while a rotation slider is non-zero;
  at zero it sleeps. The site hero now shows a copy-pasteable command
  that needs no prompt stripping.
