# Calculator benchmark stage 2: feasibility and go/no-go

Date: 2026-11-07 · Status: findings only; this is the stage gate for
docs/research/calculator-benchmark-plan.md (stage 2 of 5)

## What stage 2 set out to do

The plan's stage table fixes stage 2 as: the per-tool x per-task
feasibility matrix; license verification (GeoGebra terms, since the bench
set has no paid licenses); the NumWorks emulator clock-fidelity check and
go/no-go; and an automation spike per tool, including the W10
interactive-latency drivers. Exit criterion: matrix and go/no-go per tool
committed. All four parts are below.

## Environment established

Reference tools, all installed on the benchmark machine on 2026-11-07:

| Tool | Version installed | Provenance |
|---|---|---|
| GNU Octave | 11.3.0_1 | Homebrew bottled formula (matches the plan's 11.3.0 pin) |
| GeoGebra Classic 6 | 6.0.934.0 | official portable zip, download.geogebra.org/package/mac-port (the Homebrew cask 6.0.929.3 lags and has been Gatekeeper-disabled since 2026-09-01, so it was not used) |
| SpeedCrunch | 1.1 (arm64) | official release asset, github.com/heldercorreia/speedcrunch v1.1.0 (the same build the vendor site pins as 1.1.0) |

Already present: epher 0.9.6 release build (staging tree; the exact commit
is pinned again when stage 3 runs it), Microsoft Edge and Safari, Node
26.12.0, Python 3.9.

Harness stack, matching the repo's existing automation: Chrome for
Testing 152.0.7977.54 arm64 from the shared puppeteer cache (the same
stack scripts/a11y uses), driven over the raw DevTools protocol by a
zero-dependency Node driver (Node 26 ships global fetch and WebSocket).
GUI tools are driven by AppleScript System Events, the same mechanism the
a11y suites use for native apps. All spikes were run with the tooling in
/tmp/bench2 (scratch space, not committed); the stage 3 harness turns the
proven paths into committed drivers.

## Go/no-go per tool

| Tool | Tier | Drive (proven) | Readout (proven) | Verdict |
|---|---|---|---|---|
| epher native (CLI/REPL/TUI) | N | CLI one-shot; pty driving has a repo precedent (scripts/a11y/tui_pty_drive.py) | stdout | GO |
| epher web | W | raw CDP; the PWA exposes a visible TEXTAREA input | #epher-result div, aria-live=polite, DOM text | GO |
| GNU Octave | N | octave-cli --eval | stdout; tic/toc for in-engine | GO |
| SpeedCrunch | N | System Events keystrokes into the main window | the history AXTextArea exposes the expression and result ("6 x 7 = 42"); Edit > Copy Last Result is a clipboard backup path | GO |
| GeoGebra web applet | W | the Apps API in a local embed page (deployggb.js from the vendor CDN): evalCommand, getValue | numeric API values | GO |
| GeoGebra desktop | N | System Events keystrokes are accepted | no deterministic readout: AXManualAccessibility is rejected by the window, and the Electron remote-debugging port does not open | NO-GO; the W-tier applet runs the same engine, and the report discloses this |
| Desmos | W | CDP; window.Calc.setExpression stores latex into the live expression list (verified via getExpressions) | window.Calc.HelperExpression exists on the live site (constructor confirmed); its numericValue is the numeric readout | GO |
| NumWorks web simulator | W | CDP key events; the vendor's calculator.js supports embedding with a Module hook (onDisplayRefresh) for timestamps | canvas pixels (320x240 LCD); no framebuffer export in the wasm module, so a small LCD template matcher is a stage 3 build item | GO, with the readout work item |
| NumWorks emulator as a hardware proxy | E | not applicable | not applicable | NO-GO (see the fidelity finding) |

## Automation spike evidence

- epher one-shot `6*7`, 10 runs: min 4.0, median 4.5, max 8.9 ms
  end-to-end (spawn + eval + print).
- octave-cli --eval `disp(6*7)`, 10 runs: min 177.2, median 178.6, max
  2651.6 ms (the max is the cold first run). Octave's end-to-end numbers
  are startup-dominated, exactly why the plan routes W1-W8 through the
  in-engine tic/toc boundary; the end-to-end number is still useful for
  W10 launch-to-ready.
- GeoGebra applet page (cold load): API ready in 447 ms; then
  evalCommand("a=6*7") plus getValue("a") returned 42 in 9 ms. This is
  the W-tier driver shape for every GeoGebra task.
- Desmos: window.Calc exists on the production calculator page;
  setExpression({latex: "5\\times7"}) stored into the expression list
  (verified through getExpressions); typeof Calc.HelperExpression is
  "function", giving the numeric readout without DOM scraping.
- SpeedCrunch: after typing "6*7" and Enter, the window's history
  AXTextArea reads "6 x 7 = 42" (read verbatim via System Events). One
  earlier attempt failed (the input line is not exposed as AXTextField;
  the history area is the reliable surface), which is recorded here so
  stage 3 drivers target the history area from the start.
- NumWorks: the production simulator renders to a 320x240 canvas plus a
  mirror canvas, with no Module global and no iframe on the page; the
  vendor's calculator.js documents embedding with a caller-supplied
  module (element, mirrorCanvas, onDisplayRefresh), which is the stage 3
  driver shape: self-hosted embed page, key events in, display-refresh
  timestamps, canvas-pixel readout.
- GeoGebra desktop: keystrokes land (Electron accepted System Events
  input), but neither AXManualAccessibility on the window nor a
  remote-debugging port could be made to open, so no deterministic
  readout exists. Desktop timing is therefore not attempted.

## License findings

- GeoGebra: the license page (geogebra.org/license, fetched 2026-11-07)
  licenses the Software under the GeoGebra license, free of charge for
  noncommercial use, and the Materials and Language Files under CC
  BY-NC-SA 4.0. A benchmark study run on a personal machine and
  published as research findings is noncommercial use; the report will
  attribute GeoGebra as the license asks. No benchmark numbers are
  collected from GeoGebra before stage 3 re-checks this page.
- The rest of the bench set (epher, Octave, SpeedCrunch, Desmos web,
  NumWorks web simulator) runs on free terms; Desmos' site and NumWorks'
  simulator are vendor web services used as a user would use them.

## NumWorks emulator fidelity finding (E-tier no-go)

Epsilon's README names the online simulator as a way to try the OS in
the browser and makes no clock-fidelity claim (fetched 2026-11-07). The
web simulator is the Epsilon codebase compiled to wasm, driven by
browser callbacks (window_callback.cpp forwards display refreshes to JS;
calculator.js runs the module on a canvas). Its execution speed is
therefore host-bound wasm speed, not the calculator SoC's clock. With no
physical NumWorks in the bench set (scope decision, 2026-11-07) there is
nothing to calibrate the emulator against, so the E-tier emulator entry
is NO-GO: it cannot honestly represent the device. The same simulator
remains in W tier, where every tool runs in the same pinned browser on
the same machine; its numbers describe the vendor's web simulator, and
the report discloses exactly that.

## Feasibility matrix

Marks follow the gap analysis: complete, partial, absent, not
applicable. Capability cells are grounded in the fourth-pass matrix
(docs/research/calculator-gap-analysis-4th.md, probed 2026-11-07) plus
each tool's documentation; driver cells come from the spikes above.
SpeedCrunch and Desmos have no user scripting, so W9 is not applicable
to them; GeoGebra scripting is partial and the stage 3 pilot decides.

| Task | epher | Octave | SpeedCrunch | GeoGebra (W) | Desmos | NumWorks (W) |
|---|---|---|---|---|---|---|
| W1 scalar arithmetic | yes | yes | yes | yes | yes | yes |
| W2 transcendental | yes | yes | yes | yes | yes | yes |
| W3 matrices | yes | yes | no | yes | partial (lists only) | yes |
| W4 statistics | yes | yes | partial (mean/std only) | yes | yes | yes |
| W5 numerical methods | yes | yes | no | yes | yes | yes |
| W6 unit conversions | yes | no (manual factors) | yes | no | no | yes |
| W7 complex arithmetic | yes | yes | yes | yes | yes | yes |
| W8 date and astronomy | yes (built in) | yes | partial (manual epoch) | partial (manual epoch) | partial (manual epoch) | partial (manual Julian day) |
| W9 script processing | yes (epher script) | yes (.m) | n/a | partial (scripting commands; pilot decides) | n/a | yes (MicroPython) |
| W10 interactive latency | yes (CLI/REPL/TUI, web) | yes (CLI) | yes (GUI launch) | yes (web) | yes (web) | yes (web) |

Every partial or absent cell enters the results report as partial,
absent, or not applicable, never as a time.

## Deviations from the plan

None of the plan's fairness rules changed. Two stage-2 outcomes are
worth stating plainly: GeoGebra desktop goes from "N-tier candidate" to
NO-GO (the plan's exit criterion anticipated per-tool go/no-go), and the
NumWorks readout needs a small LCD template matcher, which is new stage
3 work the plan did not enumerate.

## Stage 3 work list

1. scripts/bench/ skeleton: the runner, the result JSON schema, and the
   data generator for W1-W8 inputs with fixed seeds.
2. The epher in-process timing binary (links the core crate; the
   engine-only boundary the plan defines).
3. Drivers, one per tool and tier, from the proven paths: epher native
   and web, Octave (--eval plus tic/toc parsing), SpeedCrunch (System
   Events + history AXTextArea), GeoGebra (applet embed page + Apps
   API), Desmos (Calc.setExpression + HelperExpression numericValue),
   NumWorks (embed page + onDisplayRefresh timestamps + LCD matcher).
4. W10 drivers: spawn-to-ready for N-tier, page-load-to-ready via CDP
   for W-tier.
5. The pilot (epher, Octave, SpeedCrunch on W1, W2, W9a, both tracks,
   same-day re-run), which is stage 3's exit gate.
