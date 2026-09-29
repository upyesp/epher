# Eclipse Marketplace listing body for epher

The listing copy for the Solutions Listing on marketplace.eclipse.org.
The Install button the body describes requires the p2 update site that
`SUBMISSION.md` next to this file documents as the one open build task;
the copy can be pasted into the listing form before the site exists,
but the listing is not complete, or installable from MPC, until the
site is live.

Copy source: `clients/vscode/README.md`, adapted for Eclipse (the p2
and PATH install story from `clients/eclipse/README.md`; no VS Code
Run script CodeLens, launch.json, or wasm/browser sections). The
screenshots live in `clients/eclipse/images/` and go into the listing's
screenshot gallery, not into the body; no social media links. The
gallery uploads, in order: `editor.png` (hero with the inline
answers), `hover.png`, `completion.png`, `results.png`, `demo.gif`.

## Short description (the card line, 190 characters)

epher is a calculator language: write ordinary math with units that
convert, and every statement's answer appears inline next to its line,
with diagnostics, hover, and completion in Eclipse.

## Body (HTML accepted)

```html
<p><strong>epher</strong> is a calculator language: you write ordinary
math, with units that convert (<code>6371 km</code>,
<code>55 mile/hr</code>), and every statement's answer appears inline,
next to the line that produced it. No REPL in a side panel, no print
statements: the editor is the calculator.</p>

<h2>What you get</h2>
<ul>
  <li><strong>Answers inline</strong>: each statement's result renders
  next to its line; <code>x = 40 + 2</code> shows <code>= 42</code>.</li>
  <li><strong>Units that convert</strong>: <code>6371 km</code>,
  <code>55 mile/hr</code>, and <code>30 deg</code> are quantities, not
  comments; <code>speed in km/hr</code> converts and the answer carries
  the right unit.</li>
  <li><strong>Live diagnostics</strong>: syntax errors point at the
  exact token, and evaluation errors carry the same message the epher
  calculator shows.</li>
  <li><strong>Hover signatures</strong>: hover any name for its
  canonical signature; your own functions show their definitions,
  catalog functions show their docs.</li>
  <li><strong>Completion</strong>: the whole catalog (math, astronomy,
  statistics), your own definitions, keywords, and snippets for the
  common statement shapes.</li>
  <li><strong>Highlighting</strong>: the shared epher grammar, through
  TM4E, with the same colors as the other epher editor clients.</li>
</ul>

<h2>Install</h2>
<p>Install from this listing (or Help, Eclipse Marketplace, search for
epher), then open any <code>.epher</code> file; Eclipse picks up the
content type and starts the language server. The plugin never downloads
anything: get the <code>epher-lsp</code> binary for your platform from
the <a href="https://github.com/upyesp/epher/releases/latest">releases
page</a> and put it on PATH before the first file is opened.</p>

<h2>Requirements</h2>
<p>Eclipse 2024-09 or newer with LSP4E and TM4E, which ship in the
standard Eclipse IDE packages, and Java 21, the floor current LSP4E
sets. On macOS, an Eclipse started from the Dock does not inherit the
shell PATH; start it from a terminal or symlink the binary into
<code>/usr/local/bin</code>.</p>

<p><a href="https://epher.org">epher.org</a> has the full
documentation, the standalone calculator, and a library of ready-made
scripts. The source and issue tracker are at
<a href="https://github.com/upyesp/epher">github.com/upyesp/epher</a>.</p>
```
