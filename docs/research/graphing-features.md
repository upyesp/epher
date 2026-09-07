# Graphing feature catalogue for `epher` (competitive survey)

**Goal:** catalogue the graphing features offered by competitive graphing-calculator
applications, as primary-source-verified input for expanding epher's graphing.
epher today: 2D `y=f(x)` only, fixed domain, single curve, static SVG (web/desktop)
and static ASCII (TUI), no CLI graphing, no interaction, no analysis. It has a
programmable expression language, parametric and polar samplers in core (unwired),
and strict WCAG 2.2 AA accessibility requirements. This note inventories what the
competition documents, so epher can pick features by value-to-effort rather than by
guesswork.

**Method (all claims verified, not assumed):** consulted only first-party
documentation — the vendors' own help systems, manuals, and reference pages — and
read every cited page in full before quoting or paraphrasing it. No blog roundups,
no Wikipedia. *Desmos:* the official help center at help.desmos.com; the 84
published articles were enumerated through the help center's own Zendesk API and
the 29 relevant article bodies fetched in full. *GeoGebra:* the official manual.
The legacy manual wiki at wiki.geogebra.org no longer resolves in DNS, so the
identical manual published by GeoGebra's official GitHub organization at
geogebra.github.io/docs/manual/en/ (source repo: github.com/geogebra/manual) was
read; its content was spot-checked against Internet Archive snapshots of
wiki.geogebra.org (snapshot 2023-06-07) for the *Graphics View*, *Algebra View*,
*Keyboard Shortcuts*, and *Accessibility* pages — content identical. *NumWorks:*
the online manual at numworks.com/manual, *Grapher* and *Regression* pages, read
in full. *Texas Instruments:* PDF guidebooks downloaded from education.ti.com and
text-extracted (pdftotext): the TI-84 Plus guidebook, the TI-Nspire™ CX II
Handhelds guidebook, and the TI-84 Plus CE Getting Started Guide and Reference
Guide. (TI now splits the CE documentation into a Getting Started Guide plus a
Reference Guide; the full graphing-menu coverage — Window/ZOOM/TRACE/CALC/Tables —
lives in the TI-84 Plus guidebook, which documents the same graphing OS and menu
set.) *Wolfram:* reference.wolfram.com/language/ref/Plot.html read in full,
including the complete options table and the *Sampling*, *Labeling and Legending*,
and *Presentation* scope subsections. All pages were accessed and all claims
extracted on **2026-08-17**.

## Feature inventory

Legend: ✅ = the cited source documents the feature. ❌ = the cited source
documents that the feature is absent or limited in the stated way. — = not
documented in the consulted pages (absence not verified). Bracketed numbers are
sources in the Citations section. "TI" covers both the TI-84 Plus/TI-84 Plus CE
and TI-Nspire CX II guidebooks; cells name the product where they differ.

| Feature | What it does | Desmos | GeoGebra | NumWorks | TI (84 / Nspire) | Wolfram |
|---|---|---|---|---|---|---|
| Explicit `y=f(x)` plots | Graph functions of x | ✅ [1,24] | ✅ [31,32] | ✅ [68] | ✅ [70,71] | ✅ [74] |
| Multiple simultaneous curves | Several functions in one view | ✅ [1] | ✅ [31] | ✅ [68] | ✅ [70] (up to 10: `Y1`–`Y10`) | ✅ [74] |
| Named functions, reuse, composition | Define `f(x)`, use it in other expressions | ✅ [16] | ✅ [47,55] | ✅ [68] (`cos(f(x))`) | ✅ [70] (`Y2=−Y1`) | — |
| Piecewise functions | Condition-defined branches | ✅ [25] | ✅ [65] (`If` command) | ✅ [68] | ✅ [72] | ✅ [74] |
| Per-curve domain restriction | Limit the plotted x-interval of one curve | ✅ [14] | ✅ [54] | ✅ [68] | — | ✅ [74] |
| Inverse plots `x=f(y)` | Graph a function of y | — | — | ✅ [68] | — | — |
| Parametric curves | `(x(t), y(t))` with parameter interval | ✅ [5] | ✅ [62] | ✅ [68] | ✅ [70,71] | ✅ (separate `ParametricPlot`) [74] |
| Polar curves | `r=f(θ)` on a polar grid | ✅ [6] | — | ✅ [68] | ✅ [70,71] | ✅ (separate `PolarPlot`) [74] |
| Implicit equations / conics | Circles, `f(x,y)=0` | ✅ [12,14] | ✅ [66] | ✅ [68] | ✅ [71] (Nspire Equation→Circle); 84 ❌ two half-functions [70] | — |
| Inequalities (shaded regions) | Shade above/below/inside; dashed = strict | ✅ [14] | — | ✅ [68] | ✅ (CE Inequality Graphing app) [72] | — |
| Points and point lists (scatter) | Plot `(x,y)`, lists of points, data | ✅ [19,20] | ✅ [52] | ✅ [68] | ✅ [70] | — |
| Function tables | Input/output table of values, editable x | ✅ [4] | ✅ [42] | ✅ [68] | ✅ [70,71] | — |
| Derivative columns / exact values in tables | Extra table columns, exact-mode output | — | ✅ [42] | ✅ [68] | — | — |
| Regressions (built-in + custom models) | Fit data to a model, plot it | ✅ [7,8] | ✅ [48] | ✅ [69] | ✅ [70] (Stat CALC); [72] (QuickPlot & Fit Equation) | — |
| Regression diagnostics | `r`, `R²`, residuals, predicted values | ✅ [7] | — | ✅ [69] | ✅ [70] | — |
| Lists as data source | Reuse stored lists/tables in plots | ✅ [19] | ✅ [48] | ✅ [68] | ✅ [70] | — |
| Sliders for free variables | A parameter becomes a drag/arrow control | ✅ [3] | ✅ [43] | — | ✅ [71] (Nspire); 84: — | — |
| Slider bounds and step | Min/max/increment on the parameter | ✅ [3] | ✅ [43] | — | — | — |
| Slider animation | Play through values; speed, direction, loop modes | ✅ [3] | ✅ [36] | — | — | — |
| Draggable points | Drag a point; it updates its defining variables | ✅ [3] | ✅ [43,53] | — | — | — |
| Trace cursor | Move along a curve reading coordinates | ✅ [12] | — | ✅ [68] | ✅ [70,71] | ✅ [74] |
| Trace step control | Set the x-increment of tracing | — | — | — | ✅ [71] | — |
| Pan | Drag/scroll the viewing window | ✅ [1] | ✅ [51] | ✅ [68] | ✅ [70] (pan during trace) | — |
| Zoom in/out | Magnify around a point or cursor | ✅ [1,15] | ✅ [49,50] | ✅ [68] | ✅ [70] | — |
| Numeric viewport settings | Enter x/y min-max (and scale) directly | ✅ [15] | ✅ [35] | ✅ [68] | ✅ [70,71] | ✅ [74] (`PlotRange`) |
| Auto-fit view to curve/data | Window adapts to the plotted content | ✅ [4] | — | ✅ [68] (Auto) | ✅ [70] (`ZoomFit`, `ZoomStat`) | ✅ [74] |
| Equal-aspect ("square") axes | Preserve shapes; circles look circular | — | ✅ [35] (axis-ratio lock) | ✅ [68] (Make axes equal) | ✅ [70] (`ZSquare`) | ✅ [74] (`AspectRatio`) |
| Logarithmic axes | Multiplicative axis scaling | ✅ [22] | — | — | — | ✅ [74] (`ScalingFunctions→"Log"`) |
| Polar grid | Grid drawn in r/θ instead of Cartesian | ✅ [6,15] | ✅ [35] | — | — | — |
| Grid and axes display options | Toggle axes/grid, tick spacing, minor lines | ✅ [15] | ✅ [35,38] | ✅ [68] | ✅ [70]; [72] (grid lines) | ✅ [74] (`Axes`) |
| Per-curve color | Distinct colors per expression | ✅ [1] | ✅ [38] | ✅ [68] | ✅ [70,72] | ✅ [74] (`PlotStyle`) |
| Line style (thickness, dash, dot) | Style beyond color | ✅ [20] | ✅ [38] | — | ✅ [70] | ✅ [74] (`PlotStyle`) |
| Show/hide individual curves | Toggle visibility without deleting | ✅ [1] | ✅ [31] | ✅ [68] | ✅ [70] | — |
| Filling / shading under curves | Fill region under/between curves | ✅ [5] | ✅ [56,57] | ✅ [68] (integral/area shading) | ✅ [70] (integral shading) | ✅ [74] (`Filling`) |
| Labels (static and dynamic) | Text or `{variable}` labels on graph | ✅ [21] | ✅ [41] | — | — | ✅ [74] (`PlotLabels`, `Callout`) |
| Legends | Separate legend for multi-curve plots | — | — | — | — | ✅ [74] (`PlotLegends`) |
| Automatic points of interest | Intercepts, extrema, intersections shown without asking | ✅ [1] | — | ✅ [68] | — | ✅ [74] (interactive callouts) |
| Evaluate at given x | On-graph value at x (cursor jumps there) | ✅ [16] (`f(3)` in expression list) | — | ✅ [68] | ✅ [70] (CALC `value`) | — |
| Find x given y | Solve `f(x)=y` from the graph | — | — | ✅ [68] | — | — |
| Zeros / roots | Find x-intercepts | ✅ [1] | ✅ [45,59] | ✅ [68] | ✅ [70,71] | — |
| Minimum / maximum | Find local extrema | ✅ [1] | ✅ [44,58] | ✅ [68] | ✅ [70,71] | — |
| Intersections | Find where two curves cross | ✅ [1] | ✅ [46] | ✅ [68] | ✅ [70,71] | — |
| Inflection points | Find curvature sign changes | — | ✅ [60] | — | ✅ [71] | — |
| Derivatives (plot and at a point) | Derivative curve, `f′(x)`, numeric slope | ✅ [17] | ✅ [55] | ✅ [68] | ✅ [70,71] | — |
| Definite integral (shaded) | Numeric integral over bounds, area shaded | ✅ [18] | ✅ [56] | ✅ [68] | ✅ [70,71] | — |
| Area between curves | Integral of `f−g` over bounds | — | ✅ [57] | ✅ [68] | — | — |
| Tangent line with equation | Tangent at a point, equation shown | ✅ [17] | ✅ [47,63] | ✅ [68] | ✅ [70] (`Tangent(` DRAW) | — |
| Asymptotes | Detect and draw asymptotes | — | ✅ [61] | — | — | — |
| Slope fields (ODE) | Direction field of `dy/dx=f(x,y)` | — | ✅ [64] | — | — | — |
| Function inspector | Interval stats (min/max/roots/integral/mean/length) + editable point table + tangent + osculating circle in one dialog | — | ✅ [42] | — | — | — |
| Sequences / recursion | Recursive definitions plotted | ✅ [25] | — | — | ✅ [70] (Seq mode) | — |
| Object trace (trail) | Moving object leaves a trail | — | ✅ [37] | — | ✅ [70] (Path style) | — |
| Save/open user work | Persist and reopen graphs | ✅ [9] | ✅ [39,67] | — | ✅ [71] (Save to Document) | — |
| Share link / export image | Link to graph; PNG/SVG export | ✅ [9] | ✅ [39,67] | — | — | — |
| Keyboard shortcuts | Documented shortcut set | ✅ [10,11] | ✅ [39] | — | — | — |
| Keyboard / screen-reader accessibility | Tab navigation, screen-reader guidance | ✅ [11] | ✅ [40] | — | — | — |
| Audio trace (sonification) | Hear the curve: pitch/slope, static for negative y, pops at intersections | ✅ [12,26] | — | — | — | — |
| Braille output (Nemeth/UEB) | Refreshable-braille mode, embosser export | ✅ [11,13] | — | — | — | — |
| Display enlargement / reverse contrast | Bigger, bolder curves; inverted colors | ✅ [11] | ✅ [40] | — | — | — |
| Statistical plots (histogram, boxplot, dotplot) | Data visualizations in the same tool | ✅ [23] | — | — | — | — |
| Adaptive sampling / discontinuity handling | Refine sampling; split at discontinuities | ✅ [29] | — | — | ✅ [70] (`Xres` resolution) | ✅ [74] (`PlotPoints`, `MaxRecursion`, `Exclusions`) |
| 3D graphing | Surfaces in three dimensions | ✅ (separate 3D calculator) [27] | ✅ [33] | — | — | ✅ (separate `Plot3D`) [74] |
| Geometry construction tools | Toolbar-based point/line/circle construction | ✅ (separate Geometry tool) [28] | ✅ [31,34] | — | — | — |
| CAS (symbolic algebra) | Symbolic simplification/solving in-app | — | ✅ [33] | — | — | — |

## Citations

All sources accessed 2026-08-17. Desmos pages are help-center articles at
help.desmos.com; GeoGebra pages are manual pages at
geogebra.github.io/docs/manual/en/ (official mirror of wiki.geogebra.org — see
Method); NumWorks pages are at numworks.com; TI documents are PDFs hosted on
education.ti.com; the Wolfram page is at reference.wolfram.com.

**Desmos (help.desmos.com)**

1. *Getting Started: Desmos Graphing Calculator* — https://help.desmos.com/hc/en-us/articles/4406040715149-Getting-Started-Desmos-Graphing-Calculator
2. *Desmos Graphing Calculator User Guide* — https://help.desmos.com/hc/en-us/articles/202529279-Desmos-Graphing-Calculator-User-Guide
3. *Sliders and Movable Points in a Graph* — https://help.desmos.com/hc/en-us/articles/202529069-Sliders-and-Movable-Points-in-a-Graph
4. *Tables* — https://help.desmos.com/hc/en-us/articles/4405489674381-Tables
5. *Parametric Equations* — https://help.desmos.com/hc/en-us/articles/4406906208397-Parametric-Equations
6. *Polar Graphing* — https://help.desmos.com/hc/en-us/articles/4406895312781-Polar-Graphing
7. *Regressions* — https://help.desmos.com/hc/en-us/articles/4406972958733-Regressions
8. *Nonlinear Regressions* — https://help.desmos.com/hc/en-us/articles/360042428612-Nonlinear-Regressions
9. *Saving and Sharing Your Work* — https://help.desmos.com/hc/en-us/articles/4405901719309-Saving-and-Sharing-Your-Work
10. *Keyboard Shortcuts* — https://help.desmos.com/hc/en-us/articles/4405966811021-Keyboard-Shortcuts
11. *Introduction to Accessibility Features* — https://help.desmos.com/hc/en-us/articles/4404860698253-Introduction-to-Accessibility-Features
12. *Audio Trace* — https://help.desmos.com/hc/en-us/articles/37064105800333-Audio-Trace
13. *Embossing Graphs with Desmos* — https://help.desmos.com/hc/en-us/articles/4407851291149-Embossing-Graphs-with-Desmos
14. *Inequalities and Restrictions* — https://help.desmos.com/hc/en-us/articles/4407885334285-Inequalities-and-Restrictions
15. *Graph Settings* — https://help.desmos.com/hc/en-us/articles/4405296853517-Graph-Settings
16. *Functions* — https://help.desmos.com/hc/en-us/articles/4405177116941-Functions
17. *Derivatives* — https://help.desmos.com/hc/en-us/articles/4406809433613-Derivatives
18. *Integrals* — https://help.desmos.com/hc/en-us/articles/4406810279693-Integrals
19. *Lists* — https://help.desmos.com/hc/en-us/articles/4407889068557-Lists
20. *Graph and Connect Coordinate Points* — https://help.desmos.com/hc/en-us/articles/4405411436173-Graph-and-Connect-Coordinate-Points
21. *Labels* — https://help.desmos.com/hc/en-us/articles/4405487300877-Labels
22. *Set an Axis to a Logarithmic Scale* — https://help.desmos.com/hc/en-us/articles/15276544054413-Set-an-Axis-to-a-Logarithmic-Scale
23. *Statistics* — https://help.desmos.com/hc/en-us/articles/4405633253389-Statistics
24. *Supported Functions* — https://help.desmos.com/hc/en-us/articles/212235786-Supported-Functions
25. *Recursion* — https://help.desmos.com/hc/en-us/articles/25917735966989-Recursion
26. *Tone* — https://help.desmos.com/hc/en-us/articles/21373904717197-Tone
27. *Getting Started: Desmos 3D* — https://help.desmos.com/hc/en-us/articles/19796006153997-Getting-Started-Desmos-3D
28. *Getting Started: Desmos Geometry* — https://help.desmos.com/hc/en-us/articles/15316366009997-Getting-Started-Desmos-Geometry
29. *Unresolved Detail In Plotted Equations* — https://help.desmos.com/hc/en-us/articles/202529079-Unresolved-Detail-In-Plotted-Equations

**GeoGebra (geogebra.github.io/docs/manual/en/)**

30. *GeoGebra Manual* (root page) — https://geogebra.github.io/docs/manual/en/
31. *Graphics View* — https://geogebra.github.io/docs/manual/en/Graphics_View/
32. *Algebra View* — https://geogebra.github.io/docs/manual/en/Algebra_View/
33. *Views* — https://geogebra.github.io/docs/manual/en/Views/
34. *Toolbar* — https://geogebra.github.io/docs/manual/en/Toolbar/
35. *Customizing the Graphics View* — https://geogebra.github.io/docs/manual/en/Customizing_the_Graphics_View/
36. *Animation* — https://geogebra.github.io/docs/manual/en/Animation/
37. *Tracing* — https://geogebra.github.io/docs/manual/en/Tracing/
38. *Style Bar* — https://geogebra.github.io/docs/manual/en/Style_Bar/
39. *Keyboard Shortcuts* — https://geogebra.github.io/docs/manual/en/Keyboard_Shortcuts/
40. *Accessibility* — https://geogebra.github.io/docs/manual/en/Accessibility/
41. *Labels and Captions* — https://geogebra.github.io/docs/manual/en/Labels_and_Captions/
42. *Function Inspector Tool* — https://geogebra.github.io/docs/manual/en/tools/Function_Inspector/
43. *Slider Tool* — https://geogebra.github.io/docs/manual/en/tools/Slider/
44. *Extremum Tool* — https://geogebra.github.io/docs/manual/en/tools/Extremum/
45. *Roots Tool* — https://geogebra.github.io/docs/manual/en/tools/Roots/
46. *Intersect Tool* — https://geogebra.github.io/docs/manual/en/tools/Intersect/
47. *Tangents Tool* — https://geogebra.github.io/docs/manual/en/tools/Tangents/
48. *Best Fit Line Tool* — https://geogebra.github.io/docs/manual/en/tools/Best_Fit_Line/
49. *Zoom In Tool* — https://geogebra.github.io/docs/manual/en/tools/Zoom_In/
50. *Zoom Out Tool* — https://geogebra.github.io/docs/manual/en/tools/Zoom_Out/
51. *Move Graphics View Tool* — https://geogebra.github.io/docs/manual/en/tools/Move_Graphics_View/
52. *Point Tool* — https://geogebra.github.io/docs/manual/en/tools/Point/
53. *Move Tool* — https://geogebra.github.io/docs/manual/en/tools/Move/
54. *Function Command* — https://geogebra.github.io/docs/manual/en/commands/Function/
55. *Derivative Command* — https://geogebra.github.io/docs/manual/en/commands/Derivative/
56. *Integral Command* — https://geogebra.github.io/docs/manual/en/commands/Integral/
57. *IntegralBetween Command* — https://geogebra.github.io/docs/manual/en/commands/IntegralBetween/
58. *Extremum Command* — https://geogebra.github.io/docs/manual/en/commands/Extremum/
59. *Root Command* — https://geogebra.github.io/docs/manual/en/commands/Root/
60. *InflectionPoint Command* — https://geogebra.github.io/docs/manual/en/commands/InflectionPoint/
61. *Asymptote Command* — https://geogebra.github.io/docs/manual/en/commands/Asymptote/
62. *Curve Command* — https://geogebra.github.io/docs/manual/en/commands/Curve/
63. *Tangent Command* — https://geogebra.github.io/docs/manual/en/commands/Tangent/
64. *SlopeField Command* — https://geogebra.github.io/docs/manual/en/commands/SlopeField/
65. *If Command* — https://geogebra.github.io/docs/manual/en/commands/If/
66. *ImplicitCurve Command* — https://geogebra.github.io/docs/manual/en/commands/ImplicitCurve/
67. *Export Worksheet Dialog* — https://geogebra.github.io/docs/manual/en/Export_Worksheet_Dialog/

**NumWorks (numworks.com)**

68. *Manual — Grapher* — https://www.numworks.com/manual/grapher/
69. *Manual — Regression* — https://www.numworks.com/manual/regression/

**Texas Instruments (education.ti.com)**

70. *TI-84 Plus Guidebook* (PDF; sections: Chapter 3 Function Graphing, Chapter 5
    Polar Graphing, Chapter 7 Tables) —
    https://education.ti.com/html/eguides/graphing/84Plus/PDFs/TI-84-Plus-guidebook_EN.pdf
71. *TI-Nspire™ CX II Handhelds Guidebook* (PDF; sections: Using the Scratchpad,
    Tracing a Plot, Finding Points of Interest) —
    https://education.ti.com/-/media/files/download-center/guidebooks/ti-nspire/5,-d-,4/gb_ti-nspire_cxii_handhelds/ti-nspire_cxii-hh_guidebook_en.aspx
72. *TI-84 Plus CE Getting Started Guide* (PDF; sections: Working with Graphs,
    Working with Tables) —
    https://education.ti.com/download/en/ed-tech/3BBF042421644CE2AF713484B03A8B11/FF49CCD0060F4DCFBDF8874AEA7F1854/84PLCE_GSG_EN.pdf
73. *TI-84 Plus CE Reference Guide* (PDF; catalog commands) —
    https://education.ti.com/download/en/ed-tech/3BBF042421644CE2AF713484B03A8B11/DA0D22E4BC924472A8E6D147FE76CC74/GRefGuide_84PlusCE_EN.pdf

**Wolfram (reference.wolfram.com)**

74. *Plot — Wolfram Language Documentation* (complete options table; Scope →
    Sampling, Labeling and Legending) — https://reference.wolfram.com/language/ref/Plot.html

## Synthesis for epher (analysis only — not source claims)

Reachability against epher's architecture (2D, expression-language-driven, SVG
renderer + ASCII renderer, parametric/polar samplers already in core, WCAG 2.2 AA
hard requirement):

**Reachable.** Multiple curves and per-curve domain restriction (every product);
viewport controls — numeric bounds, zoom, pan, auto-fit (all five); wiring the
existing parametric/polar samplers (Desmos, NumWorks, TI); tables of values (all
five; the TUI's ASCII table is a natural fit); named functions, composition, and
piecewise conditionals in the expression language (Desmos, GeoGebra, NumWorks);
points and point lists (Desmos, GeoGebra, NumWorks, TI); regressions with `r`/`R²`
diagnostics (Desmos, NumWorks, TI) — least-squares in core, Desmos's
tilde-model syntax (`y₁ ~ a·x₁²+c`) is a clean language design; the analysis
toolbox — value-at-x, zero, min/max, intersect, derivative, definite integral,
area between curves, tangent line, inflection (TI's CALC menu and NumWorks's Find
menu are the canonical minimal sets; one numeric engine in core serves web + TUI
and a future CLI); automatic points of interest (Desmos, NumWorks) — same root-
finding engine plus labeling; trace cursor with arrow-key stepping (NumWorks, TI —
keyboard-native, maps directly to the TUI); slider-driven parameters (Desmos,
GeoGebra — re-sampling is already cheap; the TUI substitutes step/playback input
for dragging); log axes, polar grid, equal-aspect axes, grid/axes options,
per-curve color *and* style, show/hide (pure renderer work); fill/inequality
shading for explicit functions (Desmos, TI, Wolfram); save/open and PNG/SVG
export (Desmos, GeoGebra — epher's SVG renderer makes export trivial);
keyboard-shortcut catalogue (Desmos, GeoGebra).

**Not reachable short-term.** 3D graphing (Desmos 3D, GeoGebra 3D View, Wolfram
`Plot3D`) — a renderer rewrite for both backends; geometry construction tools
(GeoGebra, Desmos Geometry) — a tool-centric, point-and-click interaction model
incompatible with epher's expression-list design; CAS symbolic algebra (GeoGebra
CAS View) — a separate subsystem; braille-embosser export (Desmos) — hardware
integration, though UEB/Nemeth math formatting is worth revisiting later; audio
trace (Desmos) — Web Audio work in the web app and a beep/MIDI scheme in the TUI,
valuable but its own project.

**Ordered by value-to-effort** (highest value-to-effort first):

1. Multiple curves + per-curve domain restriction — trivial, unlocks nearly
   everything else.
2. Viewport controls (numeric bounds, zoom, pan, auto-fit) — trivial in SVG,
   keyboard-friendly in the TUI, prerequisite for trace and analysis UX.
3. Wire parametric + polar samplers — core work already done; matches the syntax
   of all three competitors that support it.
4. Tables of values — both renderers get it nearly free; doubles as the analysis
   output surface.
5. Shared numeric analysis engine → zero/min/max/intersect/value, then
   derivative/integral/tangent/area-between/inflection — one core implementation
   feeds web, TUI, and a future CLI; this is TI's CALC menu plus NumWorks's Find
   menu.
6. Points of interest on top of (5) — NumWorks shows the design (auto-marked
   intercepts/extrema/intersections with a legend).
7. Expression-language upgrades: named functions, composition, piecewise
   conditionals.
8. Sliders (web); TUI gets parameter stepping/animation instead of dragging.
9. Trace cursor — arrow-key model; near-free in the TUI.
10. Regressions (least-squares + model picker + `r`/`R²`).
11. Renderer polish: line styles alongside colors, log axes, polar grid,
    inequality/fill shading.
12. Save/open/share/export — schema extension plus PNG export from the existing
    SVG.
13. Audio trace and braille math formatting — flagship accessibility features,
    long-horizon.

**Accessibility implications (WCAG 2.2 AA is a hard requirement).** Every
interactive feature above must be keyboard-operable (WCAG 2.1.1): trace and
points-of-interest navigation via arrow keys and Tab (NumWorks/TI model; Desmos
binds arrows and Tab to points of interest in audio-trace mode), sliders via
arrow keys plus typed bounds (GeoGebra's `+`/`−` and step increments; Desmos's
audio-trace `S` key links a slider to arrows), zoom via shortcuts (Desmos
Alt+Plus/Minus; GeoGebra Ctrl+Plus/Minus). Nothing may be mouse-only, including
dragging points. Differentiation must not rely on color alone (WCAG 1.4.1):
Desmos encodes strict vs non-strict inequalities as dashed vs solid boundaries,
and GeoGebra's accessibility page explicitly advises dashed lines in addition to
color and warns against pure red/green — epher should pair every curve color with
a selectable line style and expose both. Contrast (1.4.3/1.4.11): GeoGebra
recommends dark-on-white with thick lines; Desmos ships display enlargement and
reverse contrast — epher needs a display-size setting and a high-contrast theme
in both renderers. Screen-reader semantics (4.1.2): Desmos's editor reads
"x squared" rather than "x superscript 2", and its audio trace can describe
points, curves, and axes; every point-of-interest marker and analysis label in
the web SVG needs a text equivalent, and the TUI's textual nature should be
treated as a feature, not a fallback. Motion (2.3.3, 2.2.2): slider playback and
any auto-pan/auto-fit animation must be pausable and respect
`prefers-reduced-motion`. Focus management (2.4.7/2.4.11): the interactive web
graph needs a visible focus indicator and a documented tab order across curves
and points of interest.
