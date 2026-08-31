# ADR-0040: PHP-style comments, a roomier graph, script files for the CLI and REPL, a fuller share, and a legend that never leaves

Date: 2026-08-31

## Status

Accepted (amends ADR-0001's "no comments" language note, ADR-0013's CLI
conventions, ADR-0016's keypad layout, ADR-0035's text-button underline
rule as applied, ADR-0038's solar legend and share text, and ADR-0039's
tab height)

## Context

A user report round surfaced six gaps:

1. On the solar pane, unchecking every legend checkbox removed the
   graph - and the legend with it, because the pane rendered only when
   the filtered scene produced parts. With nothing visible there was no
   way to bring anything back.
2. The site heading "epher... ephemeris" read as a pun at the expense
   of clarity; the brand line should just say "epher".
3. The script language had no comments. Scripts longer than a few
   lines needed notes.
4. The graph panes lost too much space above the plot (toolbar rows, a
   vertical legend stack, a bare "3D" heading, sliders), and on the
   calculator pane the history list was squeezed; long text labels
   (Clear, Copy SVG) and the boxed Clear history command wasted more.
5. A shared history line carried the app link but not the expression
   itself; the recipient saw a bare URL.
6. The CLI could not run a script file as an argument, and the REPL
   could not load one (or a saved script) on demand.

## Decision

**Comments, PHP style.** The tokenizer skips `//` and `#` line
comments (to the end of the line, which still separates statements)
and `/* ... */` block comments, which may span lines or sit inline
between tokens; a block comment's newlines never become separators. An
unterminated `/*` is a parse error, as in PHP. Because the interactive
frontends evaluate one line at a time (the same reason `\n` means `;`),
a block comment closes on its line there; multi-line block comments
parse wherever a whole script parses at once (the one-shot expression,
the web entry, script files read as a whole are line-by-line, so they
follow the one-line rule).

**The solar legend never disappears.** The pane renders whenever the
scene exists: the frame comes from the full scene (ADR-0038), the parts
from the filtered one, and an empty part set renders an empty plot
inside the stable frame - the checkboxes survive to re-admit bodies.

**The share reads as three lines**: the message ("Sharing this in the
epher app:"), the expression, then the link. The share sheet keeps the
link as its own field; the clipboard fallback writes the three lines
together.

**Script files reach the CLI and the REPL.** `epher plots/sine.es`
runs a file line by line like a piped script. An argument counts as a
file when it names an existing file and contains `.`, `/`, or `\` -
characters an expression name cannot carry - so `epher x` still
evaluates the name `x` next to a file called x. The REPL's `load`
command runs a file path or a saved script's name the same way (the
`load` line itself joins the history), and `save script name` remains
the way to store one. The line loop is one shared function: piped mode,
script files, and `load` cannot drift apart.

**The graph gets the space.** The plot's box claims at least 38% of
the viewport (220px at any size) instead of a token 96px - the pane
scrolls, and the POI list scrolls beneath a real plot. The 2D legend is
a wrapping strip, not a 44px-per-curve stack. The Clear and Copy SVG
commands are icon buttons (trash, copy; `aria-label` and `title` keep
the names). The 3D orbit/zoom sliders moved below the plot, and the
bare "3D" heading is gone (the plot's accessible name still says 3D).
The Clear history command loses its box - text-styled, underline only.
Keypad tabs step down to 34px tall (the 24px AA floor with room to
spare; the 44px width floor stays), the hint bar tightens, the answer
panel gives back a step, and the keypad grid's gap narrows - all to hand
space to the history list. Every change is mirrored in the
accessibility record's target-size row.

**The site heading is just "epher"**, at the h1's full size; the
descriptor line beneath it still carries the localized tagline.

## Consequences

- The language gains a fifth lexical element; the parity tests fail a
  locale whose guide lacks the comments section, and the core tests pin
  both comment forms, inline use, statement separation across
  comments, and the unterminated error.
- A fully-hidden solar scene shows an empty framed plot instead of
  nothing; hiding the last body is no longer a trap.
- Shared links read as a sentence, an expression, and a link; the
  platform decides how its sheet joins the text and URL fields.
- The calculator pane yields roughly 60px to history on a phone and
  the graph plot grows by 3-4x wherever a long POI list previously
  squeezed it; every shrunken control stays above the 24px AA floor
  and the 44px keypad keys are untouched.
