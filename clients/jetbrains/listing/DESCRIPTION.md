# JetBrains Marketplace listing for io.epher.jetbrains

This folder is the listing kit for plugins.jetbrains.com, the same
pattern as clients/visualstudio/listing/.

- `demo.epher` — the script the captures show (real epher syntax,
  verified against the 0.5.57 CLI).
- `CAPTURE-GUIDE.md` — how to take the four screenshots and the
  animated GIF on a machine with a desktop (the marketplace listing
  needs authentic shots of a commercial JetBrains IDE, which no CI or
  headless box can run: the IDE requires an account login).
- `setup.ps1` — prepares a Windows machine for the captures: installs
  IntelliJ IDEA Ultimate 2024.2.4 (free 30-day trial, no purchase),
  installs the plugin from disk, and creates the demo project.

## Where each piece of the listing lives

| Piece | Source | Rides |
|---|---|---|
| Name | plugin.xml `<name>` | every build |
| Description | plugin.xml `<description>` (the body below the `---`) | every build |
| Change notes | plugin.xml `<change-notes>` | every build |
| Icon | `META-INF/pluginIcon.svg` (+ `_dark`) | every build |
| Screenshots + GIF | the **Media** section of the plugin's admin page | uploaded once, manually |
| Tags | chosen during the first upload | editable in the admin panel |
| Compatible products | computed from the zip; fix in General Information (drop IDEA Community and Android Studio) | admin panel |
| License | EULA chosen at first upload (GPL-3.0-with-classpath-exception style per repo LICENSE) | admin panel |

Everything below the `---` is the description text that lives in
clients/jetbrains/src/main/resources/META-INF/plugin.xml. Keep the two
in sync: edit plugin.xml, then copy the body here.

---

A calculator language with the answers inline: write ordinary math,
with units that convert, and every statement's answer appears next to
the line that produced it, right in your JetBrains IDE.

Type a formula and the answer is already there. No REPL in a side
panel, no print statements: the editor <em>is</em> the calculator.

<h2>What you get</h2>
<ul>
  <li><b>Answers inline</b>: each statement's result renders next to
  its line: <code>x = 40 + 2</code> shows <code>= 42</code>.</li>
  <li><b>Units that convert</b>: <code>6371 km</code>,
  <code>55 mile/hr</code>, <code>30 deg</code> are quantities, not
  comments. <code>speed in km/hr</code> converts; the answer carries
  the right unit.</li>
  <li><b>Live diagnostics</b>: syntax errors point at the exact token,
  and evaluation errors carry the same message the epher calculator
  shows.</li>
  <li><b>Hover signatures</b>: hover any name for its canonical
  signature; your own functions show their definitions, catalog
  functions show their docs.</li>
  <li><b>Completion</b>: the whole catalog (math, astronomy,
  statistics), your own definitions, keywords, and snippets for the
  common statement shapes.</li>
  <li><b>Run the script</b>: Run Epher Script in the Tools menu (or
  the gutter play icon) evaluates the whole file; the transcript with
  every 2D or 3D graph opens in the epher results tool window.</li>
</ul>
<h2>First use</h2>
<p>The plugin downloads the shared <code>epher-lsp</code> binary for
your platform on first use (a few megabytes) and caches it; after that
everything is local. All understanding lives in the language server,
so every IDE behaves identically.</p>
<h2>Requirements</h2>
<p>A commercial JetBrains IDE, 2024.2 or newer: IntelliJ IDEA Ultimate,
PyCharm Professional, WebStorm, PhpStorm, RubyMine, CLion, DataGrip,
GoLand, Rider, Aqua, or RustRover. The platform's built-in LSP client
ships only in the commercial IDEs, so IntelliJ IDEA Community and
Android Studio cannot load this plugin.</p>
<p><a href="https://epher.org">epher.org</a> has the full
documentation, the standalone calculator, and a library of ready-made
scripts.</p>
