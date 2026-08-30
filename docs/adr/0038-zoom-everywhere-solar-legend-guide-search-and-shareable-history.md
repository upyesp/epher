# ADR-0038: zoom on every tile, the solar legend, guide search, shareable history, and the keypad's dead keys

Date: 2026-08-30

## Status

Accepted (amends ADR-0031's zoom scale, ADR-0034's TUI-only zoom, and
ADR-0016's `clear`/`history` keys; amends ADR-0035 with the underline
rule for text-styled controls)

## Context

A review round surfaced a cluster of gaps, mostly around the graph
tiles:

1. The TUI could zoom (wheel and drag pan, ADR-0034) but the web app -
   and therefore the desktop app, which shares its UI - could not: no
   wheel, no pinch, and the 3D fine-control zoom slider spanned only
   ±2× (`2^-z`, ADR-0031). A user could not zoom in to inspect detail
   anywhere except a terminal.
2. The zoom slider's range had no useful meaning at its ends. -1 and +1
   should say something: -1 fits every object in the view, +1 isolates a
   single one.
3. The solar system pane had no legend - unlike the 2D plot, whose
   legend checkboxes hide each curve (ADR-0015 amendment). Eleven bodies
   with no way to isolate one.
4. The points-of-interest list below a 2D plot had no way to leave the
   page: copying meant selecting text by hand.
5. The in-app guide (ADR-0018) was scroll-only; finding a topic meant
   reading the table of contents and hoping.
6. History items could be re-run (ADR-0027) but not sent anywhere. On
   phones, sharing a calculation with someone is the gesture the
   platform exists for.
7. The keypad's `clear` and `history` keys (ADR-0016) inserted tokens
   that **no frontend implemented**: submitting them errored with
   `unknown name: clear` in the CLI, the TUI, and the app alike. The
   keys promised commands that never shipped.

## Decision

**Zoom (every graph tile).** The fine-control zoom slider now spans two
decades each way: the render window scales by `10^(-2z)`, so slider -1
widens the window 100× (every object fits, however scattered) and +1
narrows it 100× (a single object fills the pane). `0` is unchanged: the
fit view.

On top of the slider, wheel and pinch zoom every tile:

- **2D plot** - the wheel zooms around the pointer; a two-finger pinch
  zooms around the fingers' midpoint; a zoom slider (the same -1..1
  control the 3D panes have) sits with the graph options. The zoom sets
  an explicit x window: cartesian curves **re-sample over the window**
  (120 points across what is visible, so deep zoom stays smooth - the
  ADR-0006 seam), param and polar curves keep their parameter samples
  and are clipped by a new SVG clip path. The y range stays data-fit
  inside the window (`geometry_in` in the core). The trace cursor maps
  through the same window.
- **3D panes (surfaces and solar)** - the wheel and a pinch scale the
  camera distance directly, past the slider's ±1 display range. One
  finger still orbits; a second finger suspends the orbit and pinches.

Gestures move in steps of at most 5× per event and the span clamps at
nine decades from the fit window - deep enough that float sampling, not
the clamp, ends the journey. A fresh graph, `graph clear`, and Clear
graph re-fit. The desktop app inherits all of it through the shared
webview; the TUI keeps its ADR-0034 gestures.

**Solar legend.** The solar pane renders the same legend the 2D plot
has: one entry per body, swatch in the body's color, checkbox to hide
its orbit, trail, and dot. Hidden bodies stay out of the pane's
accessible name and the SVG export. A fresh scene resets the legend.

**Points-of-interest copy.** The list heading gains a copy button on
its right: one click puts every listed point on the clipboard, one
line each, exactly as displayed. The icon is the platform's own copy
glyph (the rounded SF-style sheets on Apple devices, the squared mark
elsewhere); the confirmation lands in the answer region like Copy SVG.

**Guide search.** A search box at the top of the in-app guide scans
every heading, paragraph, and list entry; matches appear as chapter +
snippet results, and jumping scrolls to the hit and flashes it
(opacity only, safe under reduced motion). Enter jumps to the first
hit; Escape clears.

**Shareable history.** Every history item gains a share icon to its
left, again in the platform's own artwork. It opens the OS share sheet
(`navigator.share`) with "Look at this in the epher app" and a link to
the app carrying the line's expression as `?expr=` - the same contents
a click loads into the entry (`history_expression`, answer suffix
stripped). Where the share API is missing (desktop browsers, the
desktop shell) the link lands on the clipboard instead. Opening a
`?expr=` link stages the expression into the entry - keyboard focus on
desktop only, per the ADR-0035 rule - and consumes the parameter so a
reload starts clean.

**The keypad's dead keys, revived.** `clear` and `history` are now
real commands in the app and the TUI: `clear` empties the plot (the
Clear graph button's exact behavior, plus the result line), `history`
opens/focuses the history list. The CLI is unchanged - its plot state
is one-shot and its history is the file. The keys finally do what
ADR-0016's keypad promised.

**Underline for text-styled controls.** Buttons that read as plain
text - Clear graph, Copy SVG, Clear history, Close, the Calculator and
Graph pane tabs - carry an underline, so "you can act on this" no
longer rides on color alone (WCAG 1.4.1). Recorded in the ADR-0035
amendment.

## Consequences

- The core's zoom contract changes: `View3D::with_offsets` /
  `with_spin_phase` scale by `10^(-2z)` (was `2^-z`), and
  `with_camera`'s floor drops to 0.01 - a guard for the projection,
  not a limit for the user.
- The web renderer gains `geometry_in` (window geometry), a plot clip
  path, wheel/pinch listeners on both graph components, and a
  `window` prop - all reusable seams.
- Sharing works only over https or localhost (the web share API's
  origin rule); the clipboard fallback covers the rest, including the
  desktop shell.
- The solar legend and zoom windows are per-session, like the curve
  legend - nothing new persists.

## Amendment (2026-08-30): pinch zoom on the 2D plot owns the touch gesture

The 2D plot's `touch-action` moved from `pan-x` to `none` so a pinch
inside the plot scales the view instead of the page. The pane strip
stays swipeable everywhere around the plot - toolbar, margins, pane
edges - and the 3D plot already kept full capture (ADR-0035).

## Amendment (2026-08-30): the review round - parity, polish, and one real bug

The follow-up review round kept every frontend in step and fixed what
it caught:

- **The spin loop's stale cells (the "ghost animation").** The spin
  loop (ADR-0032) reads the rotation sliders' live cells, but a fresh
  3D graph or solar scene reset only the slider states - a non-zero
  slider kept spinning every later scene with the sliders showing 0.
  The resets now clear the cells too.
- **The solar legend keeps the frame.** The legend now renders as a
  compact two/three-column strip **above** the plot, and the scene's
  viewBox comes from the **full** scene (`solar_view_box` +
  `solar_parts_in` in the core): hiding a body through the legend no
  longer rescales, jumps, or collapses the view.
- **Share text.** The shared message is "Checkout this in the epher
  app:" (localized), and the clipboard fallback carries the message
  and the link together.
- **The POI copy icon leads.** It sits **left** of the heading, and
  the TUI gained the same gesture: a Graph → "Copy points of interest"
  item writing the list through OSC 52.
- **Help above Settings.** The desktop menu rail and the TUI menu bar
  both place Help before Settings (the mobile hamburger already did).
- **The keypad reshuffle** recorded in the ADR-0016 amendment (newline
  key in, command keys out).
- **The TUI catches up**: guide search (`/` opens the query strip,
  Enter jumps to hits) mirrors the web overlay's search box, and the
  TUI's wheel zoom, orbit, and orthographic camera were confirmed
  current - they ride on the same core the app uses.
