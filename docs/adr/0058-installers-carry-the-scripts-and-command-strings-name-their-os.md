# ADR-0058: Installers carry the script collection, and every command string names its operating systems

Date: 2026-09-04

Status: Accepted

## Context

The project ships 333 ready-to-run scripts in the `epher scripts/` folder
of the repository. The website's Scripts page browses them, the scripts'
README teaches them, and the guide's copy-and-paste commands reference
them — but every path in those examples pointed at the source checkout.
A user who installed epher from a release had none of them: the
copy-and-paste instructions from the previous round failed with parse
errors, because a path that names no existing file is evaluated as an
expression, and the error messages ("invalid number", "unexpected
character") pointed at the tokenizer rather than at the missing file.

The same round exposed a second documentation defect. The examples I
handed a user mixed operating systems under one heading: a bash-friendly
path for Debian and a PowerShell path for Windows, each looking like THE
way to run epher. A reader cannot tell from an unlabeled command string
which operating system it is for, so a working instruction reads as a
broken one on the other platform.

## Analysis

- **The scripts must travel with the installers, not the checkout.**
  Downloading a script from GitHub before the first run is exactly the
  ceremony epher exists to remove. The bundler already has a mechanism
  that fits: `bundle.resources`, whose map form copies a directory tree
  preserving structure (`tauri-utils` `ResourcePaths`: a directory key
  walks its contents under the mapped destination). One config line puts
  the tree inside every artifact — deb and rpm under `/usr/lib/epher`,
  the NSIS installer beside the binaries in `%LOCALAPPDATA%\epher`, the
  macOS app bundle at `Contents/Resources` — with no per-platform hooks.
  Verified empirically: `dpkg -c` on a locally built deb lists all 333
  `.epher` files under `usr/lib/epher/scripts/`.
- **The installed path is per operating system, so the documentation
  must be too.** One path per platform is not a wart to hide but the
  fact to state. A fenced block whose comment lines name the systems
  (`# Debian, Ubuntu, Fedora (deb, rpm)`, `# Windows (PowerShell)`,
  `# macOS`) lets a reader find their line and skip the rest. The guide's
  fence discipline (byte-identical across the eight locales) applies
  unchanged: the labels live inside the fence, the translations around
  it.
- **Where the copy/paste instructions live, the installed path must
  live too.** The guide's CLI chapter, the scripts' README, the Scripts
  page on the website, and the repository README all teach running
  scripts from a terminal; all four now show the installed paths. The
  source-checkout path keeps its place in the scripts' README —
  contributors run from the tree — with the installed paths beside it.

## Decision

1. The tauri bundle's resources map gains `"../../../epher scripts":
   "scripts"`. Every installer built from this repository carries the
   script collection, structure intact, at the platform's resource
   location:

   - Linux (deb, rpm): `/usr/lib/epher/scripts/`
   - Windows: `%LOCALAPPDATA%\epher\scripts\`
   - macOS: `/Applications/epher.app/Contents/Resources/scripts/`

2. Documentation shows terminal commands with each command's operating
   systems named in a comment line directly above it, and the three
   installed paths appear wherever scripts are taught: the guide's CLI
   chapter (section 4 intro), the scripts' README (Running a script),
   the website's Scripts page ("Run them from your terminal"), and the
   repository README's Quick Start.

3. The guide's piped-script example labels its shell: the `printf` pipe
   is marked for Linux and macOS shells, with the PowerShell
   spelling (`"x = 3`nx * 10" | epher -`) beside it.

## Consequences

- A fresh install of epher on any of the three desktop platforms can run
  the reference script from the terminal with the copied command, before
  anything is downloaded.
- The installers grow by the size of the collection (about half a
  megabyte uncompressed, less packaged) — noise against the webview
  runtime they already ship.
- The web app and the TUI are unchanged; the collection rides in their
  bundles' resources but nothing reads it from disk there.
- Future scripts added to `epher scripts/` ride the next release without
  further config; the Scripts page already builds its catalog from the
  same folder, so the two cannot drift apart.

## Alternatives considered

- **Downloading the scripts at install time.** Rejected: it makes the
  installer's behavior depend on the network, contradicts the privacy
  stance of chapter 6 ("nothing leaves your computer"), and breaks
  offline installs.
- **Materializing scripts into the user's home on first run of the
  app.** Rejected for this round: it writes into user space without a
  visible act, duplicates the collection per user, and still leaves the
  CLI-only user empty-handed. The installers' own file layout is the
  honest place.
- **A single user-home path on every OS** (`~/epher-scripts`). Rejected:
  system packages cannot write one user's home cleanly, and the per-OS
  resource locations are already correct, standard places that the
  OS-labeled fences state plainly.

## Amendment (2026-09-07): decision 1 lands one release late, and a
path that names no file says so

The release that carried this ADR (v0.5.25) shipped without the
resources-map line: the local build that verified it used an edited
`tauri.conf.json`, and the edit never reached the commit, so the
installers kept shipping only the guide files while the guide, the
scripts' README, the Scripts page, and the repository README all named
the installed script paths. A user copying any of those commands got a
tokenizer error — "unexpected character: ':'" from the Windows drive
letter, "invalid number: '.'" from the `.epher` suffix — because a path
that names no file is evaluated as an expression (ADR-0040).

The resources map now gains `"../../../epher scripts": "scripts"` in
`tauri.conf.json`, and the release workflow checks the artifact contents
on every platform instead of trusting the config line: `dpkg-deb -c` /
`rpm -qlp` on Linux, `7z l` on the NSIS installer, and a mounted-dmg
`test -f` on macOS, all looking for
`scripts/astronomy/moon/full-moons.epher`.

An argument that names no existing
file but *looks like* a path — it starts with `/`, `\`, `./`, `../`, or
a drive letter, none of which any expression can start with — now fails
with `error: no such script file: <path>` instead of a parse error, so
a missing install names the file rather than the tokenizer. Arguments
that could still be expressions keep ADR-0040's behavior: `1.5.5`
reports a parse error, and `a/b` stays division. The man page's
EXAMPLES section now shows the installed script path on each operating
system, so the manual and the guide agree.

## Amendment (2026-09-07, later the same day): the script headers and the Scripts page name the installed paths too

The report came back once more: every script's page on the website
still showed `epher "epher scripts/..."` — the checkout command each
file's header comment carries, and the run hint the Scripts browser
rendered from the repository-relative path. A reader on Windows or
macOS had no copyable equivalent at all. Both surfaces now speak the
installed locations: every one of the 333 headers carries the three
commands (`Linux (deb, rpm)`, `Windows`, `macOS`) over the installed
path, the REPL line points at them, and the Scripts page renders the
same three commands, each with its own copy button. The transcripts
are untouched — the lines are comments — and the checker passes all
333.

A later pass settled the page layout the same way: the per-script run
box sits *below* the script text (the reader sees the code first, then
how to run it), each command's copy button sits at the left of its
row — before the operating-system name it copies for, so label and
command read as one line — and the page's static "Run them from your
terminal" section — which documents the installed paths for the whole
collection — hides while a single script is open, since the script
page then carries the same commands itself.

## Amendment (2026-09-05): the collection-wide run section leaves the Scripts page

The Scripts page carried a static "Run them from your terminal" section
— an example script's installed path on each operating system, shown
whenever no single script was open. Every script page already carries
its own three copyable commands under the code (the section hid itself
on script pages for exactly that reason), so the collection-wide block
repeated what the per-script commands do better: an example nobody
asked for, three operating systems of path noise above the browser.
The section is gone; the per-script run commands stay as they are.
