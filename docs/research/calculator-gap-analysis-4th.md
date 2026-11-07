# Calculator gap analysis, fourth pass: epher 0.9.6 and the platform story

Date: 2026-11-07 · Status: findings only, nothing implemented

## Why this fourth pass

The third pass (docs/research/calculator-gap-analysis-3rd.md, 2026-09-06)
re-measured the 31-row expectation matrix against epher v0.5.37 and found
24 of 31 covered. Two months later epher stands at v0.9.6, with a
statement-body language (control flow, destructuring, a string library),
one LSP server feeding ten editor families, 532 shipped scripts in eleven
categories, a fully documented language reference, and install paths for
Linux (five channels), macOS (three paths), and Windows. This fourth pass
re-measures the same 31 rows against v0.9.6 and adds a second matrix for
the platform and delivery dimensions the language matrix never covered:

- the documented language reference,
- first-party IDE/editor extensions,
- a first-party LSP server,
- the UI surfaces: CLI, REPL, terminal UI, desktop GUI, web app, mobile,
- native availability on Linux, Windows, and macOS,
- the shipped script collection.

The comparison must survive third-party scrutiny, so every claim names its
source and fetch date, carried-forward columns are labeled as carried, and
every epher cell is probed empirically against a clean store.

## Method

- **epher column**: empirical. Every matrix row was probed against a
  release build of the current tree (`epher-cli --version` prints
  `epher 0.9.6`; commit 10012d4 on staging, two commits past the v0.9.6
  train commit, both touching editor-client files only: a Sublime settings
  fix and an nvim ftplugin sync). Probes ran one-shot and piped with
  `EPHER_STORE_DIR` pointed at a clean store (/tmp/gap4/store). Probe
  outputs quoted below are verbatim. The REPL was smoke-tested
  (`epher repl` answers `2+3` with `5`), a bundled astronomy script was run
  end to end, and `graph save` wrote a 6.4 KB SVG. UI-only behaviors (POI
  markers, sliders, keypad banks) were confirmed against the guide and the
  ADR record as in earlier passes.
- **App columns**: re-verified against primary sources on 2026-11-07.
  Each app's current version was pinned first, then the matrix rows and
  the new platform rows were checked against that version's own docs,
  release feed, or store listing. Where a network path was bot-blocked,
  the claim is labeled as carried (with the date of its last direct
  verification) or cited from the vendor's own repository or store
  listing instead.

### Latest versions pinned this pass

| # | App | Latest version on 2026-11-07 | Primary evidence |
|---|-----|------------------------------|------------------|
| 1 | NumWorks | Epsilon 23.2.3 | numworks.com/manual/ (200) + latest tag in numworks/epsilon on GitHub; Google Play listing live |
| 2 | Desmos | continuously updated web app (no versioned releases); API docs v1.9 current | desmos.com/scientific (200); desmos.com/api/v1.9/docs (200); Google Play listing live; help.desmos.com bot-blocked (403 / Cloudflare challenge) |
| 3 | GeoGebra | Classic 6.0.934.0 | download.geogebra.org/installers/6.0/version.txt; manual now at geogebra.github.io/docs/manual/en/ (200) |
| 4 | TI-84 Plus CE / TI-Nspire CX II | current guidebook editions (education.ti.com guidebook pages, 200) | TI-84 Plus CE Python guidebook page + TI-Nspire CX II guidebook page fetched; OS build numbers not extractable from the pages |
| 5 | WolframAlpha / Wolfram Language | Wolfram Language 14.3 (August 2025) | wolfram.com/mathematica/quick-revision-history.html (200) |
| 6 | SpeedCrunch | 1.1.0 (released 2026-10-20) | site download page (GitHub release assets); v1.1.0 release metadata (prerelease: false); site manual is the 1.1 build |
| 7 | HiPER Scientific | manual v12.0 still current | hiperlabs.eu (200); hipercalc-manual-12.0.pdf fetched (499,550 bytes); no 12.1/13.0 exists; Google Play listing (cz.hipercalc) live; the site's structured data now claims Android and iOS |
| 8 | HP Prime | live sources unreachable | hpcalc.org returns 403 to automated clients (site and /prime/, http and https); Wayback snapshot of hpcalc.org/prime/ (2024-08-28) fetched for ecosystem facts; matrix cells carried from 2026-09-02 |
| 9 | MATLAB / GNU Octave | MATLAB: R2025a or later is the current train per the official VS Code extension listing; GNU Octave 11.3.0 | docs.octave.org/latest/ (200, "GNU Octave (version 11.3.0)"); octave.org/download (200); MathWorks repos mathworks/MATLAB-language-server and mathworks/MATLAB-extension-for-vscode (both pushed 2026-11-05); marketplace listing for MathWorks.language-matlab. mathworks.com doc pages are Akamai-blocked (Access Denied) to automated clients |

### App-side changes found since 2026-09-06

- **SpeedCrunch moved to a 1.x line.** v1.0.0 and v1.1.0 (2026-10-20) are
  the current releases; the site manual is rebuilt (Sphinx, "1.1
  documentation"). The 1.1 downloads ship macOS (arm64) and Windows
  (x64) only; the site's Linux download still points at SpeedCrunch-0.12
  tarballs on Bitbucket. The reference section list (basic, integer,
  statistical, units, ieee754) shows the feature set unchanged: no
  regression, no hypothesis tests, no distribution functions.
- **HiPER now ships (or claims) an iOS version.** The vendor site's
  structured data declares "operatingSystem": "Android, iOS" and lists
  graphing, symbolic algebra, step-by-step solutions, and image
  recognition. The manual is unchanged (v12.0), so the capability matrix
  carries, and the new iOS claim rests on the vendor's own listing
  (Apple's App Store pages are bot-blocked).
- **HP Prime Python support confirmed in the ecosystem record.** The
  Wayback snapshot of hpcalc.org/prime/ (2024-08-28) describes official
  beta firmware with Python support for Prime G1/G2 hardware and
  Connectivity Kit software for Windows and macOS (a beta at snapshot
  time), plus a FLOSS third-party cross-platform connectivity kit.
- **NumWorks** moved Epsilon 21.0.0 to 23.2.3; the fetched manual shows
  the same feature surface (Python scripting, regression, inference,
  distributions, exam mode).
- **GNU Octave** moved 11.1.0 to 11.3.0; the statistics package remains
  a package, not core.
- **Wolfram Language** is at 14.3 (August 2025); the quick-revision page
  lists nothing later. WolframScript remains the command-line entry.
- **GeoGebra** is at Classic 6.0.934.0; the manual's command pages
  (CAS_Specific_Commands, Spreadsheet_Commands, Geometry_Commands,
  Scripting_Commands, 3D_Commands) were fetched directly and confirm the
  CAS, spreadsheet, geometry, scripting, and 3D rows.
- **TI**: the TI-Nspire CX II product page moved (old slug 404s); the
  guidebook pages fetched fine. No capability-level change surfaced.
- **Desmos**: no versioned releases; the API v1.9 documentation is
  current and the scientific page loads. Its column carries from the
  2026-09-02 re-verification (help.desmos.com is bot-blocked).

No fetched source showed any of the nine apps gaining or losing a
capability-matrix feature since the third pass.

## The expectation matrix

✓ full first-class support · ~ partial or adjacent · · absent.
Count = apps of the nine offering the row (✓ or ~).

| Capability | NW | DM | GG | TI | WL | SC | HP | Pr | Oc | of 9 | epher v0.5.37 | epher v0.9.6 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Complex numbers | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 9 | ✓ | ✓ |
| Numeric equation solving | ✓ | ~ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | ✓ | ✓ |
| Lists as values | ✓ | ✓ | ✓ | ✓ | ✓ | · | ~ | ✓ | ✓ | 8 | ✓ | ✓ |
| Regression / curve fitting | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | ✓ | ✓ |
| Probability distributions | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ~ | 8 | ✓ | ✓ |
| Random numbers | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | ✓ | ✓ |
| Matrices | ✓ | ~ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | ✓ | ✓ |
| Calculus (deriv/integral) | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ~ | 8 | ✓ | ✓ |
| Data plots (hist/box/scatter) | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | ✓ | 7 | ✓ | ✓ |
| Units + conversion | ✓ | · | · | ✓ | ✓ | ✓ | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Exact/fraction results | ✓ | ✓ | ✓ | ✓ | ✓ | ~ | ✓ | ✓ | · | 8 | ✓ default | ✓ default |
| Display formats (sci/eng/digits) | ✓ | ~ | · | ✓ | · | ✓ | ✓ | ✓ | ✓ | 7 | ✓ | ✓ |
| Table of values | ✓ | ✓ | ~ | ✓ | ~ | · | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Auto points of interest | ✓ | ~ | ✓ | ✓ | ~ | · | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Parameter sliders + animation | ~ | ✓ | ✓ | ✓ | ~ | · | ~ | ✓ | · | 6 | ✓ | ✓ |
| Inequality graphing | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | · | 7 | ✓ | ✓ |
| Percentage operator | · | ✓ | · | · | ~ | ~ | ✓ | · | · | 4 | ✓ | ✓ |
| Hypothesis tests + CIs | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | ~ | 7 | ✓ | ✓ |
| Constants library (physics…) | ✓ | · | · | ~ | ✓ | ✓ | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Financial (TVM/NPV/IRR) | ~ | · | ✓ | ✓ | ~ | · | · | ✓ | · | 4 | ✓ | ✓ |
| Bitwise ops | ~ | · | · | ~ | ✓ | ✓ | ✓ | ✓ | ✓ | 7 | ✓ | ✓ |
| Prime factorization / nt extras | ✓ | · | ✓ | · | ✓ | · | ~ | ✓ | · | 5 | ✓ | ✓ |
| Implicit relations (x²+y²=1) | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | · | 6 | ✓ | ✓ |
| CAS (symbolic algebra) | · | · | ✓ | ✓ | ✓ | · | ✓ | ✓ | · | 4 | · | · |
| Exam mode | ✓ | · | ✓ | ✓ | · | · | · | ✓ | · | 4 | · | · |
| Step-by-step solutions | · | · | ~ | · | ✓ | · | ✓ | · | · | 3 | · | · |
| Spreadsheet view | · | ~ | ✓ | ✓ | · | · | · | ✓ | · | 3 | · | · |
| Geometry | · | · | ✓ | ✓ | · | · | · | ✓ | · | 3 | · | · |
| Programming surface | ✓ | · | ~ | ✓ | ✓ | · | · | ✓ | ✓ | 5 | ~ no input/IO | ~ no input/IO |
| Natural-language input | · | · | · | · | ✓ | · | · | · | · | 1 | · | · |
| Curated/live data | · | · | · | · | ✓ | · | · | · | · | 1 | · | · |

NW = NumWorks, DM = Desmos, GG = GeoGebra, TI = TI-84 Plus CE / TI-Nspire
CX II, WL = WolframAlpha / Wolfram Language, SC = SpeedCrunch, HP =
HiPER Scientific, Pr = HP Prime, Oc = MATLAB / GNU Octave.

**epher v0.9.6 holds 24 of 31 (23 full, 1 partial), unchanged in count
from the third pass.** The seven absent rows are still the deliberate
product boundaries. The one partial row keeps its note but the note
changed underneath (see "What changed since the third pass").

### Row-by-row probe record for epher v0.9.6 (clean store)

Verbatim outputs; one-shot mode unless noted.

- Complex: `3 + 4i` answers `3+4i`; `abs(3 + 4i)` answers `5`.
- Solving: `solve x^2 - 2 == 0` answers `x = -1.4142135623730951, x = 1.414213562373095`;
  `solve x^2 + 1 == 0` answers `x = -i, x = i`.
- Lists: `xs = {1, 2, 3}; xs * 2` answers `{2, 4, 6}`.
- Regression: `linreg({1,2,3},{2,4.1,6.2})` answers `y = 2.1*x + -0.1 (r = 1)`;
  `quadreg({0,1,2,3},{1,2,5,10})` answers `y = 1*x^2 + -0*x + 1 (r = 1)`.
- Distributions: `normcdf(1)` answers `0.841344746069`; `invnorm(0.975)`
  answers `1.95996398454`; `binompdf(2, 5, 0.5)` answers `0.312500000004`;
  `chi2cdf(3.84, 1)` answers `0.949956478751`.
- Random: `randseed(42); random(); randseed(42); random()` reproduces
  `0.741564878772` twice; `randn(0, 1)` draws `-0.342244134738`.
- Matrices: `inv([[1,2],[3,4]])` answers `[[-2, 1], [1.5, -0.5]]`;
  `rref([[2,1,5],[1,-1,1]])` answers `[[1, 0, 2], [0, 1, 1]]`; `det` answers `-2`.
- Calculus: `derivative(x^3, 2)` answers `12`; `integral(x^2, 0, 1)` answers `1/3`.
- Data plots: `graph hist({1,2,2,3})`, `graph boxplot(d)`, and
  `graph scatter(xs, ys)` all parse and route to the plot pane (rendered
  in the GUI frontends; the CLI confirms the plot statement).
- Units: `5 km in mile` answers `3.10685596119 mile`; `72 km/hr in m/s`
  answers `20 m/s`.
- Exact: `frac(1, 3)` answers `1/3`; `exact(0.75)` answers `3/4`;
  `1/3 + 1/6` answers `0.5` (repeating decimals fraction by default).
- Display: `scientific(12345.678)` answers `1.2345678e4`;
  `engineering(12345)` answers `12.345e3`; `grouped(1234567)` answers
  `1 234 567`.
- Table: piped mode prints the aligned table; `table x^2 from 1 to 3
  points 3 derivative x^3` prints the x, y, y' columns; `table x ^ 2
  values d` prints data columns. The CLI one-shot mode fails to parse
  `table` (see Defects).
- POI / sliders: `graph x^2 - 4` and parameter graphs parse and render
  with points of interest and slider controls in the GUI frontends
  (guide sections 2 and 3; unchanged from the third pass's verification).
- Inequality: `graph y < x^2` parses (`graph: y < x^2`).
- Percentage: `200 + 10%` answers `200.1`; `200 * (1 + 10%)` answers `220`.
- Tests and intervals: `ztest({5.4, 4.8, 5.1}, 5, 2)` answers
  `z = 0.0866, p = 0.931`; `tinterval({5.1, 4.9, 5.2}, 0.95)` answers
  `(4.6872, 5.4461)`; `chisq_gof({10, 9, 11}, {10, 10, 10})` answers
  `chi2 = 0.2, p = 0.9048`; `anova` and `ttestpaired` probe clean.
- Constants: `c` answers `299792458`; `n_a` answers the exact Avogadro
  integer; `h`, `G` probe clean.
- Financial: `tvm_pmt(360, 0.08/12, -100000, 0)` answers `733.764573879`;
  `npv(0.1, {-100, 60, 60})` answers `500/121`; `irr({-100, 60, 60})`
  answers `0.130662386292`.
- Bitwise: `5 & 3` answers `1`; `0xFF` answers `255`; `5 << 2` answers `20`;
  `bits(8)` sets the word size.
- Number theory: `isprime(97)` answers `true`; `factors(360)` answers
  `2^3 * 3^2 * 5`; `gcd(12, 18)` answers `6`.
- Implicit: `graph x^2 + y^2 == 1` parses (`graph: x^2 + y^2 == 1`).
- CAS probes stay absent: `expand((x+1)^2)` with unbound `x` is a typed
  error ("unknown name: x"); there is no symbolic engine.
- Exam mode: no subcommand, flag, or store setting exists.
- Programming surface (the one partial row): strings now carry escapes
  (`print("line1\nline2\ttab")` prints a newline and a tab) and a string
  library (`upper`, `lower`, `trim`, `substr`, `find`, `replace`,
  `split`, `join` all probe clean: `split("a,b", ",")` answers `{a, b}`,
  `join({1,2}, "+")` answers `1+2`); control flow is complete
  (`if 3 > 2 then 1 else 0` answers `1`; `while` and `for` probe clean,
  `for x in 1 to 4 do x^2` answers `{1, 4, 9, 16}`); `def` bodies with
  `return` and recursion work (`def f(x) do x * 2 end; f(5)` answers
  `10`); destructuring works (`{a, b} = {10, 20}; a + b` answers `30`).
  What keeps the row partial: there is still no `input()` ("unknown
  name: read") and no file I/O ("unknown name: write"). Every one of the
  four programming-surface peers (TI-Basic, NumWorks Python, Octave, PPL)
  offers keyboard input.
- Natural-language input and curated/live data: absent.

## The platform and delivery matrix

This table is new in the fourth pass; it measures how each calculator
reaches its user, not what the language can do. The row definitions are
written to be checkable against a vendor's own pages, and each epher cell
cites its ADR or probe.

✓ ships today, first-party · ~ partial or qualified (see note) · · absent.

| Delivery dimension | NW | DM | GG | TI | WL | SC | HP | Pr | Oc | of 9 | epher 0.9.6 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Documented language reference | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 9 | ✓ |
| First-party IDE/editor extension | · | · | · | · | ~ | · | · | · | ✓ | 1 | ✓ |
| First-party LSP server | · | · | · | · | ✓ | · | · | · | ✓ | 2 | ✓ |
| CLI (one-shot/batch) | · | · | · | · | ✓ | · | · | · | ✓ | 3 | ✓ |
| REPL | · | · | · | · | ✓ | · | · | · | ✓ | 3 | ✓ |
| Terminal UI (full-screen) | · | · | · | · | · | · | · | · | · | 0 | ✓ |
| Desktop GUI app | · | · | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | 7 | ✓ |
| Web app | ✓ | ✓ | ✓ | · | ✓ | · | ~ | · | ✓ | 5 | ✓ |
| Mobile app or mobile UI | ✓ | ✓ | ✓ | · | ✓ | · | ✓ | ~ | ✓ | 6 | ~ |
| Linux (native) | · | · | ✓ | · | ✓ | ✓ | · | ~ | ✓ | 5 | ✓ |
| Windows (native) | · | · | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | 7 | ✓ |
| macOS (native) | · | · | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | 7 | ✓ |
| Large offline script collection | ~ | · | ~ | ~ | ✓ | · | · | ~ | ~ | 2 | ✓ |

Cell notes (the short form of what backs each mark):

- **Documented language reference** (9/9, epher ✓): every one of the nine
  publishes a complete reference; this row exists to hold epher to the
  same bar, and it clears it: site/reference.md is generated from the
  core dispatch and catalog (a core test keeps it from drifting), and
  the byte-identical guide ships in eight locales (ar, de, en, es, fr,
  hi, pt, zh-CN) with 30 language sections.
- **First-party IDE/editor extension** (1/9): MATLAB ships an official
  VS Code extension (repo mathworks/MATLAB-extension-for-vscode,
  marketplace item MathWorks.language-matlab, supporting R2025a and
  later; pushed 2026-11-05). Wolfram earns ~: WolframResearch ships the
  LSPServer package and once shipped Wolfram Workbench, but the editor
  extensions in common use are community-built (lsp-wl and its clients).
  epher ✓: nine editor families ship today, each with a per-editor
  download (ADR-0067, ADR-0068): VS Code Marketplace (upyesp.epher),
  Visual Studio Marketplace (upyesp.EpherVisualStudio), JetBrains
  Marketplace (plugin 34631), Eclipse Marketplace, Package Control
  (LSP-epher), Open VSX (upyesp/epher), vim.org (script 6195), and
  GitHub release packages for Zed, Neovim, and Emacs; all nine
  marketplace/repository URLs returned 200 on 2026-11-07.
- **First-party LSP server** (2/9): MathWorks publishes
  mathworks/MATLAB-language-server ("implements the Microsoft Language
  Server Protocol for the MATLAB language", requires R2021b or later);
  WolframResearch publishes LSPServer. epher ✓: one server
  (crates/lsp) speaks for every editor family (ADR-0066) and runs
  scripts with results shown in the editor (ADR-0069).
- **CLI** (3/9): WolframScript ("execute Wolfram Language code
  anywhere", command-line page fetched), MATLAB batch mode (matlab
  -batch; mathworks.com doc pages are bot-blocked this pass, carried
  from general availability), and Octave's octave-cli. epher ✓: one-shot
  and piped modes, `graph save file.svg` export probed.
- **REPL** (3/9): WolframScript's interactive mode, MATLAB's command
  window / -nodesktop, Octave's native REPL. epher ✓: `epher repl`
  (probed; auto-ans on operator-first lines, autocomplete with
  per-function descriptions).
- **Terminal UI** (0/9): epher is alone in shipping a full-screen
  terminal interface (epher tui, 80×24, keypad banks, paste-as-one-paste
  ADR-0059). None of the nine offers a TUI; Octave's CLI is
  line-oriented, not a full-screen application.
- **Desktop GUI app** (7/9): GeoGebra Classic desktop, TI's Nspire
  Student Software and TI-SmartView CE, Mathematica, SpeedCrunch, HP
  Prime Virtual Calculator, MATLAB desktop, and the Octave Qt GUI. The
  three without: NumWorks (web emulator only), Desmos (web and mobile
  only), HiPER (Android/iOS only). epher ✓: the desktop app (`epher gui`)
  opens .epher files on double-click (ADR-0070); the store manifest
  ships two visible applications (ADR-0071).
- **Web app** (5/9 + HiPER ~): NumWorks's web emulator, Desmos, GeoGebra
  web apps, WolframAlpha/Wolfram Cloud, MATLAB Online. HiPER ~: a web
  app existed; its domain was parked at the third pass and was not
  re-checked this pass. Octave ~: Octave Online is community-hosted,
  not first-party. epher ✓: the offline PWA at epher.org/pwa/ (fetched
  200 on 2026-11-07), same engine and store as the other frontends.
- **Mobile** (6/9 + HP Prime ~): NumWorks, Desmos, GeoGebra, HiPER
  (cz.hipercalc on Google Play), Wolfram (Cloud App and Player App), and
  MATLAB (MATLAB Mobile) ship native apps; HP Prime ~: mobile apps
  existed historically, current store status unverified this pass.
  TI ·: no mobile app surfaces on any TI page fetched this pass (the
  legacy iPad apps are gone from the current product pages); epher ~:
  the PWA carries a mobile view
  (auto-slide-to-graph on phones, ADR-0029) but ships no store app; a
  native app would close this to ✓.
- **Linux** (5/9): GeoGebra Classic desktop, Wolfram, SpeedCrunch (the
  0.12 line; 1.x ships no Linux build), MATLAB, Octave. HP Prime ~: no
  official virtual calculator for Linux (a FLOSS third-party
  connectivity kit exists per the 2024-08-28 snapshot). epher ✓: one
  package across apt, dnf, Flathub, Snap, and AUR (ADR-0061).
- **Windows** (7/9): GeoGebra, TI (Student Software, SmartView CE),
  Wolfram, SpeedCrunch 1.1 (x64 exe), HP Prime Virtual/Connectivity Kit,
  MATLAB, Octave. NumWorks, Desmos, and HiPER ship no Windows app.
  epher ✓: installers plus the Microsoft Store manifest (ADR-0071, with
  ADR-0074's headless waiver note for the CLI).
- **macOS** (7/9): same set as Windows with SpeedCrunch 1.1 (arm64 dmg).
  epher ✓: Homebrew (ADR-0072) within three install paths, none needing
  an approval (ADR-0076).
- **Large offline script collection** (2/9): Wolfram (thousands of
  runnable examples ship inside the documentation) and MATLAB (shipped
  example galleries). Partial marks: NumWorks (example scripts in the
  manual and the online workshop), GeoGebra (manual examples; Materials
  are online and community), TI (guidebook programs; community archive
  online), HP Prime (guide examples; the hpcalc.org archive is large but
  online and community), Octave (manual examples and small bundled
  files). epher ✓: 532 scripts in eleven categories (astronomy 131,
  engineering 55, algebra 47, geometry 46, graphs 44, finance 42,
  number-theory 40, statistics 39, probability 34, trigonometry 29,
  calculus 25) ride every installer (ADR-0058), open from an in-app
  examples browser (ADR-0077), and one of them was run end to end this
  pass (julian-day-from-date.epher answers 2451545 for J2000.0 and
  2461041.5 for 2026-01-01).

## What changed since the third pass

epher moved v0.5.37 to v0.9.6 across ADRs 0062 through 0081. The ones
that move matrix cells or their notes:

- **Statement bodies, control flow, destructuring, and the string
  library** (ADR-0064): `if`/`then`/`else` with statement bodies,
  `while` loops, `return` with early exits, multi-statement `def` bodies,
  `{a, b} = expr` destructuring, string escapes (`\n`, `\t`), and the
  string library (upper, lower, trim, substr, find, replace, split,
  join). The programming row stays partial only on input and file I/O;
  the third pass's "no strings/for/IO" note is three-quarters closed.
- **The imaginary unit is reserved** (ADR-0065), closing the third
  pass's defect #2: `i = 5; 3+4i` now answers "error: cannot assign to
  i: that name is the imaginary unit" instead of 23.
- **IDE extensions speak one LSP server** (ADR-0066), **editors beyond
  VS Code** (ADR-0067), **every editor gets a download and the last two
  families ship** (ADR-0068), and **the language server runs scripts and
  every editor shows the results** (ADR-0069): the whole platform row of
  this report, nine editor families on one server.
- **Staged delivery and CI-cut releases** (ADR-0062, ADR-0073): release
  channels follow the verified tree.
- **Desktop delivery** (ADR-0070 double-click, ADR-0071 store manifest
  with two visible applications, ADR-0072 Homebrew, ADR-0076 three macOS
  paths, ADR-0075 Apple Silicon gate) and **Linux channels** (ADR-0061,
  extended in ADR-0058's amendment): the OS rows above.
- **The scripts collection grew 333 to 532** (nine categories to
  eleven; astronomy 31 to 131), still riding every installer, still
  browsable on the website, now openable from an in-app examples browser
  (ADR-0077).
- **Result-pane tiers** (ADR-0078, ADR-0079) and **silent constant
  definitions** (ADR-0080): presentation, not matrix rows.
- The **graph3d param CLI defect from the third pass is closed**: the
  one-shot CLI now parses `graph3d param cos(t), sin(t), t from 0 to 6`.

## The remaining gaps

### The one partial row

**Programming surface** (5/9 apps offer one). Strings, escapes, the
string library, if/while/for, def with return, recursion, and
destructuring are all in. What remains: no `input()` (keyboard input),
no file I/O beyond loading scripts, and one value per function return
(destructuring covers the pattern). All four programming peers offer
keyboard input; Octave and PPL offer file I/O and richer strings. A
small round would close input and file I/O.

### Deliberate product boundaries (unchanged, absent by decision)

- **CAS / symbolic algebra** (4/9): GeoGebra, TI, Wolfram, HP Prime.
  Still the right boundary per ADR-0004; expand/factor/simplify remain
  typed errors.
- **Exam mode** (4/9): NumWorks, GeoGebra, TI, HP Prime. Classroom
  hardware territory.
- **Step-by-step solutions** (3/9): Wolfram, GeoGebra (partial), HiPER.
- **Spreadsheet view** (3/9): GeoGebra, TI, HP Prime.
- **Dynamic geometry** (3/9): GeoGebra, TI, HP Prime.
- **Natural-language input** (1/9): Wolfram's moat.
- **Curated/live data** (1/9): Wolfram only; needs network.

## Defects found by probing this pass

1. **The CLI one-shot mode cannot parse `table`.** `epher 'table x^2
   from 1 to 3'` fails with "parse error: expected ';' or a newline
   between statements", while `printf 'table x^2 from 1 to 3\n' | epher -`
   prints the table correctly, as do the REPL, TUI, and web frontends.
   The line router strips and routes the `table ` prefix
   (crates/shell/src/lib.rs:224), but the one-shot argument path parses
   the argument as ordinary statements first. The derivative and values
   clauses work once a table parses. Neither previous pass caught this
   because both probed tables piped-only. One routing branch closes it,
   mirroring the graph3d param fix.
2. **Resolved from the third pass**: the `i` shadowing footgun is fixed
   (ADR-0065; assignment to `i` is a hard error, probed), and the CLI
   `graph3d param` gap is fixed (probed above).
3. **Documented shapes, not defects**: `normcdf` takes 1 or 3 numbers
   (a two-argument form is a type error, by design); the interval names
   are `zinterval(data, sigma, level)` and `tinterval(data, level)`.

## epher's unique strengths (re-checked, all hold)

- **One engine, six surfaces, one live store**: CLI (one-shot, piped,
  REPL), full-screen TUI, desktop GUI, offline PWA, mobile view of the
  PWA, plus the LSP inside ten editors; sessions persist across them.
  No reference calculator pairs a CLI and REPL with a GUI that shares
  the session store (MATLAB comes closest with desktop + Online + Mobile
  but its surfaces are products, not one binary).
- **The scripts collection as a first-class artifact**: 532 runnable,
  categorized scripts in every installer with an in-app examples browser;
  none of the nine ships its examples this way (Wolfram and MATLAB ship
  examples inside heavyweight documentation systems, not as a browsable
  script library beside the binary).
- **The offline solar system** and the 131-script astronomy collection,
  still the only orrery among the nine (the HP Prime community archive
  hosts astronomy programs, but they are community contributions, not
  shipped, and the calculator needs the programs sideloaded).
- **Script transcripts, seeded reproducible randomness across every
  frontend, and the byte-identical 8-locale guide**, still unmatched.
- **A full-screen terminal UI**: no reference calculator has one.
- **Ten editor families on one first-party LSP server**: only MathWorks
  and Wolfram Research ship first-party language servers, and neither
  ships one that runs scripts and shows results in the editor the way
  epher's does.

## Notes, corrections, and carried columns

- **Carried forward, not re-fetched this pass**: Desmos's capability
  column (help.desmos.com is bot-blocked: 403 plus a Cloudflare
  challenge; the scientific page and API v1.9 docs fetched instead and
  show no change), HP Prime's capability column (hpcalc.org 403 to
  automated clients; the 2024-08-28 Wayback snapshot and HP's own
  firmware notes are the newest ecosystem evidence), and MATLAB's doc
  pages (mathworks.com returns Access Denied; MathWorks's own GitHub
  repositories and the marketplace listing carry the platform claims).
  All carried cells are unchanged from their last direct verification
  (2026-09-02).
- **GeoGebra's wiki** (wiki.geogebra.org) was DNS-dead from this network
  for the third consecutive pass; the manual has moved to
  geogebra.github.io/docs (GitHub Pages) and that is the source cited
  here. The CAS/spreadsheet/geometry/scripting/3D command pages were
  fetched directly, so those rows no longer rest on Wayback.
- **SpeedCrunch's GitHub "latest release" endpoint** still answers 0.11
  (2014) while v1.1.0 (prerelease: false, 2026-10-20) sits in the same
  repo's release list and is what the official site links. The pinned
  version above follows the site.
- **HiPER's iOS claim** rests on the vendor's structured data; Apple's
  App Store pages are bot-blocked, so no store-side confirmation was
  possible. hipercalc.com's parked status was not re-checked.
- **NumWorks's exam mode** carried from the 2026-09-06 verification;
  the fetched manual's HTML does not surface the phrase and the book
  PDF is not text-extractable with the tools on this machine, but
  nothing in the 23.x line suggests a change to a feature that anchors
  the product's classroom story.
- The probe files and fetched sources live in /tmp/gap4/ (per-app
  directories nw, dm, gg, ti, wl, sc, hp, pr, oc, plus probe transcripts
  inline in this report) and are working files, not committed; this
  report is the durable record.
