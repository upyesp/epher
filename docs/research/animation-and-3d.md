# Animation and 3D graphing in competitive graphing calculators

**Goal:** establish best practice for two epher roadmap items — animated graphs
(parameter playback) and 3D surface graphing — from primary sources, to inform
an implementation in epher. epher today (ADR-0014): 2D multi-curve graphing with
parametric/polar samplers, points-of-interest analysis, trace, sliders over
session constants (web/desktop only), and tables, all projected through
`epher-core::graph` into an SVG renderer (web/desktop) and an ASCII renderer
(TUI), with WCAG 2.2 AA as a hard requirement. This note inventories exactly
how the competition documents animation and 3D, so epher can adopt the
converged model rather than invent one.

**Method (all claims verified, not assumed):** consulted only first-party
documentation and read every cited page in full before quoting or paraphrasing
it. *Desmos:* help-center article bodies fetched through the help center's own
Zendesk API (`help.desmos.com/api/v2/help_center/en-us/articles/<id>.json`) —
the HTML pages serve a Cloudflare interstitial to scripts, but the API returns
the same published article bodies, including the current versions of the pages
cited in `docs/research/graphing-features.md` ([3] sliders, [27] 3D). The
*Desmos 3D User Guide* article is a pointer to a Google Doc
(`docs.google.com/document/d/1jDJC0Zw7cB82SNEc04m5HGQHaXYaJK62iZM88ojNYwI`),
fetched in full via Google's text export and verified against the HTML.
*Desmos keyboard shortcuts:* the shortcuts pages (desmos.com/3dshortcuts etc.)
render client-side; the page's own published JS bundle contains the shortcut
table, which was extracted. *GeoGebra:* the official manual mirror
geogebra.github.io/docs/manual/en/ (source repo github.com/geogebra/manual), as
in the prior survey. *Wolfram:* reference.wolfram.com Animate, Manipulate, and
Plot3D pages read in full, including complete options tables. *TI:* the TI-84
Plus guidebook PDF (text-extracted) and the TI-Nspire™ Technology eGuide
webhelp at education.ti.com (the computer-software manual; the TI-Nspire CX II
*handheld* guidebook contains no 3D chapter — 3D Graphing lives in the CX
Premium Teacher Software). *NumWorks:* manual at numworks.com. All pages were
accessed and all claims extracted on **2026-08-17**.

---

## 1. Animation

### 1.1 Desmos: slider playback

Desmos's animation model is entirely slider-driven: "Click Play next to any
slider to animate through all of its values" [1]. The slider itself is a
parameter control with an interval and an optional step: "To adjust the
interval of your slider, click either of the values at the ends of the slider
bar and input your desired upper or lower bounds. By default, a slider can take
on any value between the upper and lower bound. However, by entering a
numerical step, you can limit the values to fixed interval steps counting up
from the lower bound" [1]. Default intervals: −10 to 10 in the 2D Graphing
Calculator and Geometry Tool, "an interval of −5 to −5 [sic] in the 3D Graphing
Calculator" [1]. Bounds may reference other variables ("dynamic bounds"): "If
you have defined a free variable in the calculator, you can use that variable
in the upper or lower bounds of a slider" [1].

Playback controls: "Click Animation Properties or click and hold Play to
control how the slider animates. You can choose to loop forwards and backwards,
repeat in one direction, play once, or play indefinitely. You can also speed up
or slow down your animation" [1]. So the loop modes are: **loop
(forwards-and-backwards), repeat (one direction), once, indefinite**, plus a
continuous speed control. The play control is per-slider, on the slider row
[1]. Shared variables make curves move together: "You can also use the same
variables in several expressions to plot curves that will change together" [1].
Sliders can drive points along curves: "For a function f(x), the point
(a,f(a)) will stay along the path of your curve", and the same for parametric
paths; in 3D "you can use sliders to help visualize different slices of conic
sections" [1].

Keyboard access to sliders is documented on the shortcuts pages: with a slider
focused, Left/Right Arrow decrease/increase the value, Page Down/Page Up
decrease/increase "by larger increment", Home/End jump to min/max [9]. A
dedicated **Slider Trace mode** (toggle with `S`) lets the arrow keys drive
the selected slider while reading the graph: Right Arrow/`L` and Left
Arrow/`J` increase/decrease, Page Up/Page Down larger steps, End/`N` and
Home/`U` max/min, Down Arrow/Tab/`K` and Up Arrow/Shift+Tab/`I` move between
sliders [9]. In audio-trace mode, playback speed has keyboard control
"Adjust Playback Speed (1 = slowest, 5 = fastest)" on Alt+1–0 (Option+1–0 on
Mac), and `A` announces active slider animations [9].

Beyond sliders, Desmos documents an event/repetition layer: **Actions** —
update rules written with `→` ("To increase a by 1: enter a → a + 1") that run
on click (clickable objects) or on a timer: "When you enable actions, you can
specify an action that will run repeatedly at a specified interval using the
ticker … Optionally, you can set the duration in milliseconds (ms) for the time
interval between ticks. Click Run to start the ticker. It will repeatedly run
the action(s) at the specified time interval" [2]. Actions are an opt-in
advanced feature enabled at the account level [2].

### 1.2 GeoGebra: slider animation and the animation panel

GeoGebra animates "free numbers and/or angles at the same time, but also
dependent points that are constrained on an object (segment, line, function,
curve, etc.)"; to be animated, "free numbers / angles need to be shown as
sliders in the Graphics View" [11]. Animation is toggled per object: "you need
to select Animation On in the Context Menu of that number, angle or point. In
order to stop the animation, you need to un-check Animation On in the same
context menu" [11]. Once any object is animating, "an animation button appears
in the lower left corner of the Graphics View. It allows you to either pause or
continue an animation" [11] — a global pause/continue control.

The animation behavior is configured "in the Properties Dialog on tab Slider":
**Speed** — "A speed of 1 means that the animation takes about 10 seconds to
run once through the interval of the slider" — and the **cycle repetition
modes** [11]:

- ⇔ **Oscillating** — "alternates between Decreasing and Increasing"
- ⇒ **Increasing** — "always increasing. After reaching the maximum value of
  the slider, it jumps back to the minimum value and continues the animation"
- ⇐ **Decreasing** — the mirror: always decreasing, jumps back to maximum
- ⇒ **Increasing (Once)** — "always increasing. After reaching the maximum
  value of the slider, it stops at this value and ends the animation"

Unlike the others, "once" is terminal — it stops by itself at the end [11].
GeoGebra documents that "while an automatic animation is activated, GeoGebra
remains fully functional. This allows you to make changes to your construction
while the animation is playing" [11] — animation does not block editing.

**Manual animation:** with the Move tool, click a free number/angle/point and
"press either the + or – key or the arrow keys on your keyboard. Keeping one of
these keys pressed allows you to produce manual animations"; the increment is
"the increment of the slider on tab Slider of the Properties Dialog" [11].
Keyboard step modifiers: "Shift + arrow key gives you a step width of 0.1
units", "Ctrl + arrow key … 10 units", "Alt + arrow key … 100 units"; a point
on a line can be moved along the line with `+`/`−` [11]. The GeoGebra
accessibility page adds the critical keyboard fact for playback: "**Space can
be used to activate a Button, toggle a Checkbox or start/stop a slider
animating**", and arrow keys move sliders and points once selected [16]. The
accessibility page also recommends, for motor-impaired users, "adding
'decrement' and 'increment' Buttons at each end of the slider" [16].

The Slider Tool dialog itself captures the parameter model: "The appearing
dialog window allows to specify the Name, Interval [min, max], and Increment of
the number or angle, as well as the Alignment, and its Speed and Animation
mode" [14]. Sliders can be positioned "absolute in the Graphics View … or
relative to the coordinate system" [14].

### 1.3 Wolfram: Animate and Manipulate

`Animate[expr,{u,umin,umax}]` "generates an animation of expr in which u varies
continuously from umin to umax"; `Animate[expr,{u,umin,umax,du}]` "takes u to
vary in steps du"; `Animate[expr,{u,{u1,u2,…}}]` "makes u take on discrete
values u1, u2, …"; multiple parameter ranges vary "all the variables u, v, …"
[18]. Initial value: `Animate[expr,{{u,u0},umin,umax}]` [18]. The same
specification grammar is the whole interface of `Manipulate`, whose control
forms map each spec to a widget: `{u,umin,umax}` → "manipulator (slider,
animator, etc.)", `{u,umin,umax,du}` → "discrete manipulator with step du",
`{{u,uinit},umin,umax,…}` → initial value, `{{u,uinit,ulbl},…}` → labeled
control, `{u,{u1,u2,…}}` → setter bar/popup menu [19].

Timing and direction, from the Animate options table [18]:

- `AnimationDirection` — Forward (default); example shows `Forward`,
  `Backward`, `ForwardBackward`
- `AnimationRate` — "the rate at which to take variables to vary"; explicit
  rate overrides duration
- `AnimationRepetitions` — "how many times to run before stopping", default
  `Infinity`
- `AnimationRunning` — default `True`: "By default Animate starts running when
  evaluated"; `AnimationRunning -> False` starts paused
- `DefaultDuration` — 5. (seconds): "When umax is finite, u is taken to vary at
  such a rate as to make the animation last for the time given by the setting
  for DefaultDuration"
- `RefreshRate` — "the default number of times per second to refresh"; the
  default step `du` "is determined by the setting for the RefreshRate option,
  and is negative if umin is larger than umax"
- `DisplayAllSteps` — force every discrete step to be displayed

Default control elements: "The following elements are included by default:
'ProgressSlider', 'PlayPauseButton', 'FasterSlowerButtons',
'DirectionButton'" [18] — i.e. a progress slider plus play/pause, speed, and
direction buttons; additional elements (`StepLeftButton`, `StepRightButton`,
`ResetButton`, `ResetPlayButton`, etc.) are opt-in via `AppearanceElements`
[18]. Unbounded ranges: `Animate[expr,{u,umin,Infinity}]` "makes an infinite
animation in which the value of u increases forever at a rate of one unit per
second" [18]. Animate is defined in terms of Manipulate: "Animate generates a
Manipulate object containing an Animator" [18].

Manipulate adds **autorun** over multiple variables: "By choosing Autorun from
the Manipulate menu, each variable is automatically run through" [19].
`AutorunSequencing` controls order and per-variable durations — "Specify a
different duration for each variable (default 5)";
`AutorunSequencing -> All` runs "through all variables simultaneously" [19].
`ContinuousAction` controls update granularity (continuous vs on-release vs
explicit Update button) [19]. Two practical notes from Animate's "Possible
Issues" are directly relevant to any implementation: "Fix PlotRange to stop
animations from jiggling" and "Use ImagePadding to make sure different labels
do not make the image size change" [18] — stabilize the viewport and the frame
across frames. Manipulate's Scope explicitly documents parameter-driven **3D**
animation: "You can interactively rotate 3D graphics while changing parameters:
Manipulate[Plot3D[Sin[x y + a], …], {a, 0, 1}]" [19].

### 1.4 TI and NumWorks

**TI-84 Plus** has no slider playback, but two adjacent mechanisms [21]:

1. **Animate/path graph styles.** The graph-style table defines `ë` Path — "A
   circular cursor traces the leading edge of the graph and draws a path" —
   and `ì` Animate — "A circular cursor traces the leading edge of the graph
   without drawing a path" [21]. These are draw-time animations of the
   sampling sweep, and parametric mode makes them temporal: the ballistics
   example says "To simulate the ball flying through the air, set graph style
   to ì (animate) for X1T and Y1T", and trace afterwards "follows the path of
   the ball over time. The values for X (distance), Y (height), and T (time)
   are displayed" [21]. So TI's parametric `t` is a *trace* parameter, not a
   playback parameter.
2. **Family of curves via lists:** "If you enter a list (Chapter 11) as an
   element in an expression, the TI-84 Plus plots the function for each value
   in the list, thereby graphing a family of curves"; `{2,4,6}sin(X)` graphs
   `2 sin(X)`, `4 sin(X)`, and `6 sin(X)` [21]. This is parameter
   enumeration, not animation.

Graph plotting itself is pausable: "While plotting a graph, you can pause or
stop graphing. Press Í to pause; then press Í to resume. Press É to stop; then
press s to redraw" [21].

**TI-Nspire** documents true parameter playback. The handheld guidebook lists,
in the Graphs/Scratchpad section, "Assign a variable in the expression to a
slider" as an analysis action [22]. The computer-software eGuide details the
slider: "A slider control lets you interactively adjust or animate the value
of a numeric variable. You can insert sliders in the Graphs, Geometry, Notes,
and Data & Statistics applications" [31]. Settings dialog fields: Value,
Minimum, Maximum, Step Size (the 3D example uses Value 3.8, Minimum 3.2,
Maximum 4.4, Step Size 0.1) [30]. Keyboard: "You can use the Tab key to move
the focus to a slider or to move from one slider to the next… When a slider has
the focus, you can use the arrow keys to change the value of the variable"
[31]. Playback: the slider's context menu is used "to start or stop its
animation"; in the animated-3D-graph example: "To animate the graph, display
the slider's context menu, and click Animate. (To stop, click Stop Animate from
the context menu.)" [29,30]. Separately, points on objects can be animated
directly: choose "either unidirectional or alternating animation", "Type a
value to set the animation speed. Any nonzero speed begins the animation. To
reverse the direction, enter a negative value", and global "Pause", "Play",
and "Reset" buttons control all page animations ("Resetting pauses all
animations and returns all animated points to the positions they occupied when
they were first animated") [32]. Direction is thus encoded as a **signed
speed** in Nspire. The 3D animation example also names the performance lever:
"Experiment with the x and y resolution to balance curve definition against
animation smoothness" [30].

**NumWorks** documents neither: the Grapher manual page covers functions,
curves, conics, inequalities, polar and parametric curves, tables, and
points-of-interest — with no slider, animation, or playback feature anywhere
on the page [33], and the manual's application list contains no 3D
application [34].

### 1.5 Best practice synthesis — animation

(All of §1.5 and §2.5 are analysis, not source claims; sources are the
enumerated primary pages.)

- **Parameter-driven playback is the universal model.** Every product that
  animates does so by varying a numeric parameter over a bounded interval:
  Desmos slider Play [1], GeoGebra Animation On for free numbers [11], Wolfram
  Animate/Manipulate `{u,umin,umax}` [17,18], TI-Nspire slider Animate
  [29,30]. The only outlier is TI-84's `ì` animate style, which animates the
  drawing sweep rather than a parameter [21]. For epher, animation = playback
  over the existing constants/sliders (ADR-0014), nothing new in the language.
- **The parameter spec is bounds + step + initial value.** Desmos
  interval+step [1]; GeoGebra Interval/Increment [14]; Wolfram
  `{u,umin,umax,du}` with `{{u,u0},…}` [17,18]; TI-Nspire Value/Minimum/
  Maximum/Step Size [30]. Bounds may be dynamic (Desmos [1]).
- **Speed, direction, and loop mode are the three playback controls, and the
  option sets converge.** Desmos: loop / repeat / once / indefinite + speed
  [1]. GeoGebra: oscillating / increasing / decreasing / increasing-once +
  speed (1 ≈ 10 s per interval sweep) [11]. Wolfram: Forward/Backward/
  ForwardBackward × AnimationRate × AnimationRepetitions (∞ default) with a
  default cycle duration (5 s) [18]. TI-Nspire: unidirectional/alternating +
  signed speed [32]. Recommended epher surface: direction ∈ {forward,
  backward, oscillate}, loop ∈ {repeat, once}, speed as cycle duration or
  multiplier — that is the intersection of all four models.
- **Playback must default to stopped and be pausable (WCAG 2.2.2).** Click-to-
  play is the norm (Desmos [1], Nspire context-menu Animate [30]); Wolfram
  inverts it by default (`AnimationRunning -> True`) but documents the paused
  start `AnimationRunning -> False` [18] — epher should adopt the paused
  default. GeoGebra's pause/continue button [11] and Nspire's global
  Pause/Play/Reset [32] show a global transport control alongside per-slider
  play. WCAG 2.2.2 (pause, stop, hide) applies to any moving/blinking/
  auto-updating content lasting more than five seconds.
- **Keyboard operability (WCAG 2.1.1) is solved and consistent:** arrow keys
  step, PageUp/PageDown coarse-step, Home/End jump to bounds (Desmos [9]);
  `+`/`−` and arrows with modifier multipliers (GeoGebra [11]); Tab focus +
  arrows (Nspire [31]); **Space as the universal play/stop key** (GeoGebra
  [16]). Desmos's Slider Trace mode (`S` to enter, arrows to drive, Tab/K/I
  between sliders) [9] is the strongest model — epher's TUI maps to it nearly
  verbatim.
- **Reduced motion (WCAG 2.3.3):** none of the five vendors' documentation
  read here mentions `prefers-reduced-motion` or a reduced-motion setting.
  This is a gap epher must close itself: honor
  `prefers-reduced-motion` (web) / a motion setting (TUI) by replacing smooth
  playback with manual stepping (the GeoGebra manual-animation model [11]) —
  every animation above has an equivalent stepwise form, which is why the
  feature degrades gracefully.
- **Performance: re-sample per parameter value; stabilize everything else.**
  The model everywhere is re-evaluating the plot for each parameter value as
  the slider/animator moves (Desmos re-plots per slider value [1]; Wolfram
  "Animate evaluates expr only for the specific literal values of u it
  requires" [18]; Nspire names the resolution/smoothness tradeoff explicitly
  [30]). Two Wolfram cautions are directly transferable: keep the viewport
  fixed across frames ("Fix PlotRange to stop animations from jiggling" [18])
  and keep label/frame geometry fixed so the image doesn't resize [18]. Wolfram
  also exposes the frame-rate knob (`RefreshRate` [18]); GeoGebra's guarantee
  that the app "remains fully functional" during playback [11] argues for
  non-blocking re-sampling. epher's re-sampling is already cheap
  (ADR-0014 sliders re-sample every curve per change); playback is just a
  timer driving the same path.
- **Beyond sliders**, the two documented extras worth knowing: ticker/event
  actions (Desmos Actions + ticker, ms intervals [2]) and multi-variable
  autorun sequencing with per-variable durations (Wolfram AutorunSequencing
  [19]). Both are deferrals for epher — sliders over session constants cover
  the teaching use cases (Desmos's "dancing curves" [1]).

---

## 2. 3D graphing

### 2.1 Desmos 3D

**Surface syntax.** The 3D calculator extends the 2D expression list to
three variables. A 2D expression stays a curve in the xy-plane until extended:
"If you type an expression in the 3D Calculator with only x and y variables,
you will see your curve graphed flat on the xy-plane. You'll also have the
option to check Extend to 3D. By checking this box, the calculator will plot
the entire surface of points that satisfies your expression for all z values"
[5]. Equivalently, "Any function of x and y will plot as a curve by default.
To view the function as a surface, check Extend to 3D or add z to the
equation. As soon as you add z, the calculator will display a surface that
represents every z value that satisfies the equation" [4]. So `z = f(x,y)`
style equations (the getting-started page graphs the plane `z=3` [3]) and
extended implicit 2D equations (a circle becomes a cylinder) are the two
surface forms. Points take `(x,y,z)` triples [3,5]. Inequalities with a
z-component plot as surfaces; curly-bracket restrictions limit which parts of
a surface render, including **slices**: "restricting x²+y² to the domain
{z=3} will graph the slice of the surface where z=3" [3], and the user guide
shows a slice driven by a slider variable a — sliders animate z-slices, a
documented 3D animation idiom [4].

**Parametric curves and surfaces.** Curves: "create a point where at least one
component is defined in terms of the parameter t … By default, parametric
curves are plotted for values of t in the interval [0,1], but it is possible
to adjust the domain manually using the parameter bounds beneath the
expression. You can either click or tab into the lower and upper bounds to
edit them" [4]. Surfaces: "To generate a surface, your expression should have
three coordinates defined in terms of the parameters u and v" [5]; the
cylinder example `(cos(u), sin(u), v)` defaults to "a cylinder with a height
of 1, with v in the interval [0,1]", adjustable to e.g. [0,3] via the
parameter bounds [4]. Other documented 3D objects: `triangle` (three points
only), `segment`, `sphere`, vectors and dot/cross products, and surfaces of
revolution via parametric templates [4,5]. Cylindrical and spherical
coordinates: equations in `r, θ, z` or `ρ, θ, φ` are recognized ("r = 2 will
graph a circle with radius of 2. To view the cylinder, check the Extend to
3D"; "ρ=2" graphs a sphere); spherical defaults are θ in [0,2π], φ in [0,π]
[7].

**Orbit/rotate interaction.** Pointer: "You can click and drag to rotate the
cube. If you click, drag, and then release the cube while your cursor is still
in motion, the graph will continue to rotate. To stop the rotation, click
anywhere on the screen" [3] — drag with inertia. Keyboard: "press Ctrl+Alt+P
(Windows) or Ctrl+Cmd+P (Mac) to focus the cube. Press an arrow key once to
rotate or tilt the cube in that direction. Hold down an arrow key to increase
the speed of rotation and set the cube in motion. To stop the rotation, press
any key" [4]. The shortcuts page confirms: "Rotate Cube in Given Direction
(when focused)" = arrow keys; "Spin Cube in Given Direction (when focused)" =
press and hold arrow keys [9]. Zoom: buttons, scroll, or Alt+Plus/Minus
[4]. Orientation presets: XY Orientation and Default Orientation buttons
animate the cube between the 3D view and the flat xy-plane [3,4,5]. Rotation
can be disabled ("Select Disable rotation to prevent the cube from rotating")
[6].

**Grid/axes/box.** The default view is "a cube containing the x-, y-, and
z-axes", rotated slightly to show two sides with the xy-plane tilted toward
the viewer [3,4]. Axes span −5 to 5 by default; bounds can be set for all
axes at once or individually, with Zoom Square (equalize axis ranges) and
Center Origin buttons [6]. The Axes & Grid toggle removes "everything inside
the cube"; XY plane, Numbers, and Labels can be toggled individually [6].
Settings are saved with the graph (except reverse contrast and translucent
surfaces) [6].

**Projection.** A Perspective slider sweeps between the two projection
conventions: "For no distortion, or an orthographic view, drag the slider all
the way to the left. For the most distortion, or a perspective view, drag the
slider all the way to the right" [6].

**Appearance.** Per-item color and style via the item's style menu [4].
Surfaces can be made **translucent**: "Select Translucent surfaces to view
inside your surfaces and visualize the intersection of different surfaces more
easily" [6] — the documented answer to seeing surface intersections. Lighting
is built-in and can be disabled: "Select Disable lighting to remove the built-
in light source and reflections. This option creates a flatter image and gives
you precise control over color" [6]. Coordinate-based **color maps** define
colors as functions of (x,y,z) — "define a color using the function
C = hsv(250z, 1, 1). Applying that color to a surface will create a gradient
hue based on the z-values of that surface" — height/heatmap encoding; they
"don't yet work for points or curves" [8].

**Points of interest / trace in 3D:** none of the Desmos 3D pages read
documents points-of-interest markers, a trace cursor, or an analysis menu in
3D; the documented equivalent of cross-section analysis is the restricted
z-slice with a slider [3,4] and translucent surfaces [6]. (Absence per the
pages read; not a verified negative.)

**Accessibility in 3D.** Desmos's accessibility page states: "All of our math
tools, except for the 3D Calculator, include a setting to enlarge the display"
[10]. Reverse contrast does exist in 3D but "inverts the colors of the
background, expression list, and buttons while maintaining the color of
points, lines, and surfaces" [10] — surface colors are deliberately
preserved. Braille mode (Nemeth and UEB) is available in all tools except the
Matrix calculator, and the 3D settings page lists Braille mode among 3D
settings [10,6]. Keyboard navigation of the cube (arrows, focus shortcut) is
documented first-class [4,9].

### 2.2 GeoGebra 3D View

The 3D Graphics View is "part of the 3D Graphics Perspective", documented as
"Three dimensional mathematical objects can be constructed and changed
dynamically" [12]. The view supports "points, vectors, lines, segments, rays,
polygons, and circles in a three-dimensional coordinate system" plus "surfaces,
planes, as well as geometric solids (pyramids, prisms, spheres, cylinders, and
cones)" [13].

**Surface syntax.** Direct input: "Enter f(x, y)=sin(x*y) in order to create
the corresponding surface" [13]. Solids via commands (e.g.
`Pyramid[A, B, C, D]`) [13]. Construction tools include a Sphere tool, plane
toolboxes, and geometric-solids toolbox [13]. Points are placed in 3D by
click-and-hold for x/y then dragging for z [13].

**Orbit/rotate interaction.** "You may rotate the coordinate system by using
the Rotate 3D Graphics View Tool and dragging the background of the 3D
Graphics View with your pointing device. Alternatively you can right-drag the
background" [13,15]. Continuous auto-rotation is a documented toggle: "If you
want to continue the rotation of the coordinate system when the mouse is
released, you may use the option Start Rotating the View and Stop Rotating the
View in the 3D Graphics View Style Bar" [13], with "Rotate back to default
view" and "Back to Default View" buttons [13]. Translation: Move Graphics
View tool, or hold Shift and drag; zoom via Zoom In/Out tools or the mouse
wheel [13]. A "View in front of" tool re-orients the camera to face a
selected object [13]. Keyboard: Page Up / Page Down move a selected object up
and down in 3D [13], and the shortcuts catalogue documents arrow keys moving
selected points in 3D (up/right/left/down), with X / End changing the
z-coordinate [17] — object movement is keyboard-driven, but view rotation
itself is pointer-driven (right-drag or the Rotate tool) [13,15]; GeoGebra
documents no arrow-key orbit.

**Grid/axes/box and projection.** The Style Bar toggles "the coordinate axes,
the xOy-plane, and a grid in the xOy-plane" (the 3D view's reference plane is
the xOy-plane, not a full cube) and includes a button to "**choose the type of
projection**" (orthographic/parallel vs perspective) [13]. The View menu
offers the same show/hide surface (see the prior survey's Views page [33 of
graphing-features.md]).

**Trace / points of interest in 3D:** the 3D Graphics View page documents no
trace cursor or points-of-interest analysis; what it documents is object
creation, view manipulation, and styling. (Absence per the pages read.)

**Accessibility in 3D.** The manual's accessibility page documents a
screen-reader hook specifically for the 3D view: "If you make a text object in
GeoGebra called altText, altText2, altText3D then it will be attached to
Graphics View 1, Graphics View 2, Graphics View 3D respectively" — an
author-provided alt-text convention for the 3D scene [16]. The same page's
general guidance applies to 3D work: dark-on-white, thick lines, avoid pure
red/green, and dash in addition to color for differentiation [16].

### 2.3 Wolfram Plot3D

**Syntax.** `Plot3D[f,{x,xmin,xmax},{y,ymin,ymax}]` "generates a
three-dimensional plot of f as a function of x and y"; `Plot3D[{f1,f2,…},…]`
plots several surfaces; `Plot3D[…,{x,y}∈reg]` plots over a geometric region
[20]. "Plot3D is also known as a surface plot or surface graph" [20].

**The mesh model.** "Plot3D evaluates f at values of x and y in the domain
being plotted over and connects the points {x,y,f[x,y]} to form a surface
showing how f varies with x and y" [20]. Sampling: "Plot3D initially
evaluates each function at a grid of equally spaced sample points specified by
PlotPoints. Then it uses an adaptive algorithm to choose additional sample
points, subdividing at most MaxRecursion times", with the caveat that "with
the finite number of sample points used, it is possible for Plot3D to miss
features in your functions" [20]. Discontinuity handling: "Gaps are left at
any point where the fi evaluate to anything other than real numbers" and "with
the default settings Exclusions->Automatic and ExclusionsStyle->None, Plot3D
breaks surfaces at discontinuity curves it detects. Exclusions->None joins
across discontinuities" [20].

**Mesh lines.** `Mesh` controls "how many mesh lines in each direction to
draw"; "The default setting MeshFunctions->{#1&,#2&} draws an x, y mesh on
each surface"; `Mesh -> All` "draws mesh lines to show all subdivisions it
makes" (visible adaptive-refinement grid) [20]. Mesh lines are the
documented way to reveal surface geometry independent of color, and surface
themes exist specifically for it ("DarkMesh", "GrayMesh", "LightMesh",
"ZMesh" — "vertically distributed mesh lines") [20].

**Viewpoint options.** From the options table: `ViewPoint` — "viewing
position", default `{1.3,-2.4,2.}`; `ViewVertical` — "direction to make
vertical", default `{0,0,1}`; `ViewAngle` — "angle of the field of view";
`Boxed` — "whether to draw the bounding box", default True; `BoxRatios` —
"bounding 3D box ratios", default `{1,1,0.4}`; `SphericalRegion` — "whether to
make the circumscribing sphere fit in the final display area"; `RotationAction`
— "how to render after interactive rotation", default `"Fit"` [20]. 3D
graphics rotate interactively in notebooks, and Manipulate explicitly
documents combining parameter animation with interactive 3D rotation [19].

**Depth and transparency.** A telling detail: "PlotStyle->None draws no
surface, so effectively does not eliminate hidden surfaces" [20] — hidden-
surface elimination is a rendering concern distinct from what is drawn.
`Filling` fills below a surface with `FillingStyle` default `Opacity[0.5]`
[20]. `ColorFunction` colors by height (e.g. `ColorFunction -> Function[{x, y,
z}, Hue[z]]`) [20]. Lighting is a Graphics3D option (default lighting, per
the options list's reference to Graphics3D options) [20].

**Output form.** "Plot3D returns Graphics3D[GraphicsComplex[data]]" [20] — the
surface is a graphics complex of the sampled grid, which is the Wolfram way of
saying the core emits geometry, not pixels.

### 2.4 TI-Nspire 3D graphs and NumWorks

TI-Nspire's 3D Graphing view is part of the CX Premium Teacher Software (and
the iPad app); the handheld guidebook has no 3D chapter [21,22]. The 3D view
"lets you create and explore three-dimensional graphs of: 3D functions of the
form z(x,y) [and] 3D parametric plots" [23]. The work area "Shows a 3D box
containing graphs that you define. Drag to rotate the box" [23]. Functions
are entered in an entry line defaulting to "z1(x,y)=" [23]; parametric
plots use parameters with settable "tmin, tmax, umin, and umax" [25].

**Orbit/rotate interaction (keyboard-first).** "Press R to activate the
Rotation tool. Press any of the four arrow keys to rotate the graph" [26].
Auto-rotation: "Auto rotation is equivalent to holding down the right arrow
key. Press A. The Auto Rotation icon appears, and the graph rotates" [26].
Orientation presets: "Press Z, Y, or X to view along the z, y, or x axis.
Press letter O to view from the default orientation" [26]. The camera model is
exposed numerically in Range Settings: eye θ¡ (default 35), eye φ¡ (default
160), and eye distance (default 11) — a spherical viewpoint parameterization
with documented defaults [29].

**Grid/axes/box.** View elements (3D box, axes, box end values, legend) can be
shown/hidden from the View menu [29]. Box attributes include tick labels, end
values, and axis arrows [29]. Ranges default to −5..5 per axis with per-axis
Aspect Ratio (default 1) and range settings [29].

**Projection.** Orthographic is the **default**, with perspective as an
option: "From the View menu, click Orthographic Projection or Perspective
View" [29] — the opposite default order from Desmos [6].

**Appearance.** Per-graph attributes: "format: surface+wire, surface only, or
wire only"; "x resolution (enter a value in range 2-200*, default=21)"; "y
resolution … default=21"; "transparency (enter a value in range 0-100,
default=30)" [28] — the wireframe/solid format and resolution knobs are
first-class settings, and default transparency 30 makes overlapping surfaces
readable. Colors: separate line and fill colors, plus Custom Plot Color with
"Top/bottom color, Vary color by height, or Vary color by steepness" [28] —
height and slope color encodings are documented features, alongside the
color-independent wire format [28].

**Trace in 3D:** Nspire documents a 3D trace of its own kind — **z Trace**
draws a trace plane at a z-value: "The z Trace icon and the trace plane
appear, along with a text line showing the current 'z=' trace value … hold
down Shift and press the up or down arrow key" to move it [27]. That is a
cross-section cursor, not a curve-following cursor; no points-of-interest
analysis in 3D is documented [22–29].

**Animation in 3D:** the documented pattern is the slider: define the surface
with a free variable, insert a slider (Value/Minimum/Maximum/Step Size), then
"To animate the graph, display the slider's context menu, and click Animate",
combining "manual or auto rotation with the slider animation", with the
explicit advice to "Experiment with the x and y resolution to balance curve
definition against animation smoothness" [30].

**NumWorks:** no 3D graphing — the manual's application list has no 3D
application [34], and the Grapher page's entire feature set is 2D [33].
(Absence per the pages read.)

### 2.5 Best practice synthesis — 3D

- **Surface-as-mesh with resolution control is the universal rendering model.**
  Wolfram states it outright — evaluate on a grid, "connects the points
  {x,y,f[x,y]} to form a surface", with PlotPoints + adaptive MaxRecursion
  refinement [20]; Nspire exposes the grid size directly (x/y resolution
  2–200, default 21) alongside wire vs surface format and default
  transparency 30 [28]; Desmos's surfaces are the point sets of their
  expressions with per-item styling [4,5]. Non-finite values must leave gaps
  and discontinuities must break the surface, not bridge it (Wolfram [20]) —
  the 3D analog of epher's existing no-false-asymptotes rule (ADR-0014).
  Adaptive refinement is a later optimization; a fixed grid with a
  user-facing resolution knob is the minimal viable mesh.
- **Projection: both conventions exist, defaults differ, so expose both.**
  Desmos defaults to perspective-ish with an explicit orthographic end of the
  Perspective slider [6]; Nspire defaults to orthographic with perspective as
  an option [29]; GeoGebra exposes the choice as a projection type [13];
  Wolfram parameterizes the camera directly (ViewPoint default {1.3,−2.4,2},
  ViewVertical, ViewAngle) [20]. The universal lesson is an explicit
  projection choice, not one hidden convention.
- **Depth ordering is the renderer's problem, and transparency is the
  documented intersection cue.** Wolfram's note that PlotStyle->None "does not
  eliminate hidden surfaces" [20] marks hidden-surface elimination as a
  rendering-layer responsibility. For a mesh surface, sorting mesh quads by
  depth and drawing back-to-front (painter's algorithm) is the standard
  cheap answer; translucent surfaces are how Desmos [3,6] and Nspire (default
  transparency 30) [28] make intersecting surfaces legible — depth cues that
  do not rely on color alone.
- **Orbit controls + keyboard equivalents converge on the same recipe.**
  Drag to orbit (Desmos [3], GeoGebra right-drag [13], Nspire drag [23]);
  inertia/spin as an optional continuation (Desmos release-in-motion [3],
  GeoGebra Start/Stop Rotating [13], Nspire A auto-rotate [26]); arrow keys
  rotate when the view is focused (Desmos [4,9], Nspire R+arrows [26]) —
  GeoGebra is the outlier: pointer-only orbit, keyboard reserved for object
  movement [13,15,17]; named orientation presets — default
  orientation, axis-aligned views, XY-plane view (Desmos [3,4], Nspire
  Z/Y/X/O [26], GeoGebra Back to Default View [13]); zoom via wheel/buttons/
  shortcuts in all three [4,13,26 — and 29's Range Settings]; and a numeric
  viewpoint parameterization is proven by Nspire's eye θ/φ/distance defaults
  [29] — which is exactly what epher-core would need to store a view state.
- **Axis reference and bounds:** a cube with visible edges (Desmos "3D cube"
  [3,4], Nspire "3D box" [22,28], Wolfram Boxed→True with BoxRatios [20]);
  default axis spans ±5 (Desmos [6], Nspire [29]); equalization control
  (Desmos Zoom Square [6], Nspire per-axis aspect ratio [29]). An axis box
  with labeled extents is part of the minimal viable 3D, not polish.
- **Color cues must not be the only depth/shape cues (WCAG 1.4.1).** The
  documented non-color encodings: wire mesh lines (Wolfram Mesh [20], Nspire
  surface+wire/wire-only [28]), height and slope color encodings as opt-in
  (Nspire [28], Desmos color maps [8], Wolfram ColorFunction [20]),
  transparency [6,27], and preserved surface colors under reverse contrast
  (Desmos [10]). For epher: pair each surface with a mesh/wire mode and
  transparency, and treat z-height coloring as an enhancement, not the
  primary cue.
- **Accessibility is thinner in 3D than 2D everywhere — expect to close gaps
  yourself.** Desmos excludes 3D from display enlargement [10]; neither
  Desmos's nor GeoGebra's 3D docs document trace or points of interest (only
  Nspire has the z-trace plane [27]); GeoGebra's altText3D convention [16] is
  the one documented screen-reader hook for a 3D scene. epher's legend
  (ADR-0014's accessible text alternative) must carry the same role in 3D,
  and its keyboard-orbit model (arrows, presets, focus shortcut) is the
  interaction accessibility story.
- **In scope for a minimal viable 3D:** `z = f(x,y)` surfaces (one or
  several) with per-surface styling [23,24,20,4]; fixed-grid mesh with a
  resolution knob [27,19]; wireframe mode and transparency [28,6]; orbit via
  drag + arrow keys with orientation presets [3,4,26,13]; perspective/
  orthographic choice [6,29]; axis box with bounds [6,29,20]; z-slice
  restriction with sliders (the one documented 3D animation idiom, in both
  Desmos [3,4] and Nspire [30,27]).
- **Out of scope (explicit deferrals), each with a source:** parametric
  surfaces in u,v (Desmos [4,5], Nspire t/u parameters [25]); solids,
  planes, spheres, vectors, and surfaces of revolution (GeoGebra [13],
  Desmos [4]); cylindrical/spherical coordinates (Desmos [7]); color maps,
  lighting models (Desmos [8,6], Nspire [28]); 3D points of interest and
  curve trace (absent from Desmos [3,4] and GeoGebra [13]; only Nspire's
  z-plane trace exists [27]); implicit 3D relations beyond the extended-2D
  idiom (Desmos's Extend to 3D [5] is the reachable subset).

---

## 3. Implementation guidance for epher (analysis only — not source claims)

Given epher's seams (core computes, frontends render thin; SVG in web/desktop,
ASCII in TUI; WCAG 2.2 AA), the competitive evidence above supports these
recommendations.

**Grammar — 3D surfaces as a sibling of `graph`, in core.** The competitors
converge on explicit surfaces as the minimal form: `z(x,y)` functions (Nspire
[23]), `f(x,y)=…` (GeoGebra [13]), `Plot3D[f,{x,…},{y,…}]` (Wolfram [20]),
`z = …` (Desmos [4]). Consistent with epher's `graph`/`table` grammar
(ADR-0014), propose:

```
graph3d <expr(x,y)> [from a to b in x] [from c to d in y] [points n]
```

- The expression language already supports two free variables (session
  constants and `x`); add `y` as a second bound variable for 3D expressions
  only.
- Defaults: square −5..5 bounds per axis (Desmos [6], Nspire [29]); a default
  mesh resolution in the 16–24 range per side (Nspire's default is 21 [28];
  Wolfram's PlotPoints default is similar in spirit [20]); `points` capped as
  `table` already caps its points.
- **Core emits a 2D mesh, not 3D.** Core samples the surface on the (x,y)
  grid, applies the view transform (orbit + projection), and emits **2D
  polylines of the visible mesh** (one polyline per mesh line, sorted
  back-to-front). Both renderers then reuse their existing line drawing
  unchanged — this is exactly how the 2D pipeline already projects parametric/
  polar curves to samples. Painter's algorithm in core: sort mesh quads/lines
  by depth before emitting. This keeps the ASCII renderer viable: a coarse
  mesh (points 8–12) plots as a recognizable ASCII wireframe, and the TUI
  earns 3D "for free" on the same seam.
- Grammar variant for the slice idiom (the documented 3D+animation
  intersection [3,4,30]): a `graph3d … slice z = <value>` (or a free constant
  in the expression, which epher's sliders already animate) plots the
  intersection curve — defer the general case; a constant-driven slice is
  just the surface with a z-domain restriction.
- Parametric surfaces (u,v) are a deferral [4,5,24]; spherical/cylindrical
  coordinates a deferral [7]; solids a deferral [13].

**Orbit interaction.** Store a view state in core: azimuth, elevation,
distance (Nspire's eye θ/φ/distance model [29] is the cleanest parameterization
found), plus projection mode (perspective default like Desmos [6], with
orthographic toggle — or orthographic default like Nspire [29]; either is
defensible, the requirement is that both exist). Web/desktop: drag to orbit,
wheel/Alt± to zoom, arrow keys to orbit when the plot is focused (Desmos's
focus-then-arrows [4,9] and Nspire's R-then-arrows [26] are the same pattern),
and `0`/`o`-style presets for default/axis views [25,3]. TUI: numeric view
commands (`view <az> <el> <dist>`) plus preset keywords — the TUI substitutes
typed view state for gestures, exactly as it substitutes stepping for slider
drag.

**Slider playback (animation).** No new language features: playback reuses
epher's existing session constants and the ADR-0014 slider UI. In core (or the
shell layer), a small transport model: per-slider play state, direction
(forward/backward/oscillate — the intersection of Desmos's loop modes [1],
GeoGebra's [11], and Wolfram's AnimationDirection [18]), repeat/once, and
speed as cycle duration (GeoGebra's "speed 1 ≈ 10 s per sweep" [11] and
Wolfram's DefaultDuration 5 [18] both define speed as duration; recommend a
default ~5 s). Playback starts **stopped** by default (Wolfram's
AnimationRunning->False pattern [18]; Desmos's click-to-play [1]) and a single
Pause/Stop control is always visible (WCAG 2.2.2; GeoGebra [11] and Nspire
[32] both ship global pause controls). Keyboard: arrow keys step (with
PageUp/Down coarse, Home/End bounds — Desmos [9]), Space plays/pauses
(GeoGebra [16]); the TUI gets a transport input (`play <name>`, `pause`,
`step`, `speed <s>`) on the same model. Each tick re-samples and re-analyzes
via the existing path (ADR-0014 sliders already do this per change) with the
viewport and legend **fixed** across frames (Wolfram's jiggle caution [18]).
**Reduced motion:** honor `prefers-reduced-motion` (web) and a TUI motion
setting by degrading playback to stepping — no vendor documents this, so epher
sets its own bar (WCAG 2.3.3). No ticker/actions (Desmos [2]) and no
multi-variable autorun (Wolfram [19]) — deferrals.

**Deferrals, explicitly:**

1. Parametric surfaces in two parameters (Desmos u,v [4,5]; Nspire t/u [25]).
2. Solids, spheres, planes, vectors, surfaces of revolution (GeoGebra [13],
   Desmos [4]).
3. Cylindrical/spherical coordinate systems (Desmos [7]).
4. Color maps / height–slope color encodings and any lighting model (Desmos
   [8,6], Nspire [28], Wolfram ColorFunction [20]).
5. 3D points of interest and curve-following trace; at most the z-slice
   cross-section (Nspire z Trace [27]) later.
6. Adaptive mesh refinement (Wolfram MaxRecursion [20]) — fixed grid first.
7. Translucent-surface compositing as a first-class feature — approximated by
   the wireframe mode (Nspire wire format [28]); true translucency (Desmos
   [6], Nspire transparency [28]) is renderer work.
8. Auto-rotation/inertia spin (Desmos [3], GeoGebra [13], Nspire A [26]) —
   nice-to-have; manual orbit and presets first.
9. TUI orbit is typed (view commands), not modal arrow keys — a modal 3D
   navigation mode is the same class of deferred TUI work as TUI trace
   (ADR-0014).

---

## Citations

All sources accessed 2026-08-17. Desmos article bodies were fetched through
the help center's Zendesk API (`help.desmos.com/api/v2/help_center/en-us/
articles/<id>.json`) — the human-readable pages are the same articles at
help.desmos.com (see Method). GeoGebra pages are manual pages at
geogebra.github.io/docs/manual/en/ (official mirror of the manual, source repo
github.com/geogebra/manual). Wolfram pages are at reference.wolfram.com. TI
sources are the TI-84 Plus guidebook PDF and the TI-Nspire™ Technology eGuide
webhelp (computer-software manual), both on education.ti.com. NumWorks pages
are at numworks.com.

**Desmos**

1. *Sliders and Movable Points in a Graph* — https://help.desmos.com/hc/en-us/articles/202529069-Sliders-and-Movable-Points-in-a-Graph
2. *Actions* — https://help.desmos.com/hc/en-us/articles/4407725009165-Actions
3. *Getting Started: Desmos 3D* — https://help.desmos.com/hc/en-us/articles/19796006153997-Getting-Started-Desmos-3D
4. *Desmos 3D User Guide* — https://help.desmos.com/hc/en-us/articles/25042283517069-Desmos-3D-User-Guide (article body links to the guide Google Doc: https://docs.google.com/document/d/1jDJC0Zw7cB82SNEc04m5HGQHaXYaJK62iZM88ojNYwI/preview)
5. *Extending from 2D to 3D* — https://help.desmos.com/hc/en-us/articles/19736835727885-Extending-from-2D-to-3D
6. *3D Graph Settings* — https://help.desmos.com/hc/en-us/articles/20301369699981-3D-Graph-Settings
7. *Cylindrical and Spherical Coordinates* — https://help.desmos.com/hc/en-us/articles/15824510769805-Cylindrical-and-Spherical-Coordinates
8. *Coordinate-Based 3D Color Maps* — https://help.desmos.com/hc/en-us/articles/40475048737421-Coordinate-Based-3D-Color-Maps
9. *Keyboard Shortcuts — 3D Calculator* — https://www.desmos.com/3dshortcuts (client-rendered; shortcut table extracted from the page's own published JS bundle)
10. *Introduction to Accessibility Features* — https://help.desmos.com/hc/en-us/articles/4404860698253-Introduction-to-Accessibility-Features

**GeoGebra**

11. *Animation* — https://geogebra.github.io/docs/manual/en/Animation/
12. *Views* — https://geogebra.github.io/docs/manual/en/Views/
13. *3D Graphics View* — https://geogebra.github.io/docs/manual/en/3D_Graphics_View/
14. *Slider Tool* — https://geogebra.github.io/docs/manual/en/tools/Slider/
15. *Rotate 3D Graphics View Tool* — https://geogebra.github.io/docs/manual/en/tools/Rotate_3D_Graphics_View/
16. *Accessibility* — https://geogebra.github.io/docs/manual/en/Accessibility/
17. *Keyboard Shortcuts* — https://geogebra.github.io/docs/manual/en/Keyboard_Shortcuts/

**Wolfram**

18. *Animate — Wolfram Language Documentation* — https://reference.wolfram.com/language/ref/Animate.html
19. *Manipulate — Wolfram Language Documentation* — https://reference.wolfram.com/language/ref/Manipulate.html
20. *Plot3D — Wolfram Language Documentation* — https://reference.wolfram.com/language/ref/Plot3D.html

**Texas Instruments**

21. *TI-84 Plus Guidebook* (PDF; graph styles, pausing a graph, graphing a family of curves, parametric graphing) — https://education.ti.com/html/eguides/graphing/84Plus/PDFs/TI-84-Plus-guidebook_EN.pdf
22. *TI-Nspire™ CX II Handhelds Guidebook* (PDF; Scratchpad: assigning a variable to a slider) — https://education.ti.com/-/media/files/download-center/guidebooks/ti-nspire/5,-d-,4/gb_ti-nspire_cxii_handhelds/ti-nspire_cxii-hh_guidebook_en.aspx
23. *TI-Nspire™ Technology eGuide — 3D Graphs* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs3d/m_graphs3d.HTML
24. *TI-Nspire™ Technology eGuide — Graphing 3D Functions* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs3d/g3d_graphing_3d_functions.HTML
25. *TI-Nspire™ Technology eGuide — Graphing 3D Parametric Equations* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs3d/g3d_graphing_3d_parametric.HTML
26. *TI-Nspire™ Technology eGuide — Zooming and Rotating the 3D View* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs3d/g3d_zoom_rotate_3d_view.HTML
27. *TI-Nspire™ Technology eGuide — Tracing in the 3D View* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs3d/g3d_tracing_in_3d_view.HTML
28. *TI-Nspire™ Technology eGuide — Changing the Appearance of a 3D Graph* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs3d/g3d_changing_appearance_3d_graph.HTML
29. *TI-Nspire™ Technology eGuide — Customizing the 3D Environment* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs3d/g3d_customizing_3d_environment.HTML
30. *TI-Nspire™ Technology eGuide — Example: Creating an Animated 3D Graph* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs3d/g3d_example_animated_3d_graph.HTML
31. *TI-Nspire™ Technology eGuide — Adjusting Variable Values with a Slider* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs/gra_adjusting_variable_values_with_slider.HTML
32. *TI-Nspire™ Technology eGuide — Animating Points on Objects* — https://education.ti.com/html/webhelp/EG_TINspire/EN/content/m_graphs/gra_animation.HTML

**NumWorks**

33. *Manual — Grapher* — https://www.numworks.com/manual/grapher/
34. *NumWorks User Manual* (root page, application list) — https://www.numworks.com/manual/
