# ADR-0077: The apps open the bundled examples from a menu browser

Date: 2026-10-26

Status: Accepted. Goal set by the user: the example scripts ship with
every app, but users have no idea they exist; make the collection
discoverable from the GUI and the TUI, and name its size without
hardcoding the number.

## Context

The 477 scripts install with the app — flatpak copies them to
`/app/usr/share/epher/scripts`, the MSIX and the Windows zip resolve
`scripts/` beside the executable, macOS buries them in the bundle — and
no install path tells the user they exist. The only in-app surface was
`Load script…`, which asks for a path: it presumes the user already
knows files exist and where they are. The location varies by install
method, so even a well-meant hint could rarely name a usable path.

The collection already has a catalog on the website
(`site/scripts-data.json`, built by `scripts/build-scripts.mjs` from
each header's `name.epher -- purpose` line), and the constants browser
(ADR-0045) established the pattern for a grouped, keyboard-navigable
picker in both frontends.

## Decision

1. **The fleet is compile-time data.** A new crate,
   `epher-scripts-catalog`, walks `epher scripts/` in a `build.rs` and
   generates the whole catalog — path, field, area, one-line purpose,
   and full text (`include_str!`) — as static tables. The parse rules
   match the website builder's, so the two catalogs cannot drift. The
   build fails on a missing or mis-headed script, keeping the header
   standard load-bearing. Both the TUI and the web app (and therefore
   the desktop shell, which wraps the web build) embed it. 2.1 MB of
   text buys independence from every install layout: the browser works
   even where no `scripts/` folder exists, and the disk copies remain
   for `load`-by-path users and the website.
2. **The count is computed, never written.** Every number that names
   the collection — the welcome hints in both frontends — comes from
   `ENTRIES.len()` at compile time. Adding a script updates the count
   everywhere by construction.
3. **File menu leads with "Example scripts…".** The TUI opens a
   full-screen grouped browser (field/area headings, script rows with
   title and file name; arrows, PageUp/PageDown, Home/End, Enter,
   Escape; wheel scrolls it). The web app opens the same dialog as the
   constants browser: searchable, grouped, each row a button that
   loads the script. In both, Enter replaces the entry field with the
   script's text — exactly the `Load script…` semantics — and the
   transcript clears. `Load script…` stays, for paths beyond the
   bundle.
4. **A fresh session names the collection.** While nothing has been
   computed and the entry is empty, the TUI's plot pane and the web
   app's entry panel carry one line: "{n} example scripts … Menu ›
   Example scripts". It stands down the moment there is work on
   screen, so it never competes with results.

## Consequences

- Discovery no longer depends on knowing a filesystem location; the
  path problem is dissolved, not documented.
- The binaries grow by the fleet's size (about 2.1 MB). Accepted:
  the desktop app is far larger, and the TUI buys install-layout
  independence for it.
- Editing a script or adding one requires a rebuild of the apps to
  reach their browsers — the same rebuild the packaging already does.
- Three area names repeat across fields (`coordinates`,
  `distributions`, `triangles`); the catalog qualifies them
  (`field / area`) so the groups stay unambiguous.
- The examples smoke (`scripts/examples-smoke.mjs`) drives the web
  browser end to end: menu, dialog, search, load.
