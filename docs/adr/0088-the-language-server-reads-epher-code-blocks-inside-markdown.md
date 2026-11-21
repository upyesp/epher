# ADR-0088: the language server reads epher code blocks inside markdown

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-21

## Context

The extensions (ADR-0066) speak epher only where the whole file is
epher: the server analyzes whatever text a client opens, and every
client attaches the server to the `.epher` file kind alone. The prose
surfaces where people actually keep epher examples (guide pages, `epher
scripts/README.md` in this very repository, every README that documents
a script) cannot use any of it: a ```epher fence inside a markdown file
is plain text to every editor we ship, so its examples get no
diagnostics, no inline answers, no hover, and no run.

Three verified facts shaped the decision:

- The server is language-blind today: it never reads the didOpen
  `languageId` and never filters by URI, so a projection has exactly
  one natural home. The clients only ever widen the filters they
  already own.
- The run is already a server request (`epher/run`, ADR-0069), and the
  analysis pass already re-evaluates the whole document on one fresh
  `Session`, so concatenating a file's fences into one virtual script
  needs no engine change at all.
- `didChange` carries no `languageId`; only `didOpen` does. A
  document's markdown-ness must therefore be remembered server-side
  across edits, not re-derived per notification.

## Decision

The server gains a markdown projection; every client widens its
activation to markdown files that carry epher fences.

1. **One file, one script.** All epher fences in one markdown document
   concatenate, in document order, into one virtual script with one
   `Session`, one evaluation trace, one debounce pass, and one run.
   Later blocks see earlier blocks' names, which is how a tutorial
   flows, and it is the whole-file model the server already runs. A
   markdown document without fences is the empty script: no
   diagnostics, no hints, no run statements.
2. **Fence grammar, a CommonMark subset.** An opening fence is up to
   three leading spaces, then three or more backticks or tildes, then
   an info string whose first word is `epher` (case-insensitive); a
   backtick info string containing a backtick is not a fence. The
   closing fence is the same character, at least as long, with nothing
   else on the line; an unclosed fence runs to end of file. Block
   content is taken verbatim, with one trailing newline stripped: no
   dedenting, so writers indent the fence, not the code. Indented code
   blocks and inline code spans are ignored. Only `epher` is claimed;
   every other language's fences are untouched.
3. **Markdown-ness comes from the didOpen language** (`markdown`,
   case-insensitive), falling back to the `.md`/`.markdown` URI suffix
   when the client sends an empty language id, and rides on the held
   edit through the debounce so every later `didChange` keeps it. A
   markdown file forced to the epher language stays a plain script,
   and a `.epher` file is never projected.
4. **The server translates every position.** The client's text stays
   the wire truth; the projection maps client positions into the
   virtual script and maps every span the analysis produces back out:
   diagnostics, inlay hints, hover, completion, semantic tokens, and
   the run payload's per-statement lines and `answerLine`. Positions
   outside a fence map to nothing: hover and completion outside code
   are silent, never errors. Semantic token deltas are re-encoded in
   markdown coordinates, and token lengths are counted in UTF-16 code
   units as the LSP specification requires (they were byte counts).
5. **`epher/run` runs the virtual script.** The run answers with
   per-statement lines anchored to markdown lines, so every results
   surface (the VS Code pane, the JetBrains tool window, the Visual
   Studio browser tab, the text-first editors' windows) links errors
   and graphs back into the fenced block with no client change.

The clients change only at their edges: activation filters gain
markdown-with-fences gates (the JetBrains provider, the Visual Studio
content type, the Neovim, Vim, Emacs, Sublime, and Eclipse configs),
the VS Code family gains a TextMate injection grammar for the fence
plus the markdown document selector, and each client's README
documents its highlighting route. Zed stays a highlighting-only client
here: its extension API cannot attach a language server to the
built-in markdown language, the same honesty ADR-0069 records for it.
F5 and Ctrl+Enter keep working on markdown in the VS Code family: the
debug adapter already runs documents through `epher/run`, so only its
`program` check widens.

## Alternatives considered

- **Client-side extraction** (each extension carves out the blocks and
  manages its own virtual documents): nine reimplementations of the
  fence grammar and the position mapping, drifting nine ways. Rejected;
  it is the exact shape ADR-0066's "one server, thin clients" rule
  exists to prevent.
- **Per-block sessions** (each fence its own document state): breaks
  tutorial flow where block 2 uses block 1's names, and multiplies
  passes and runs. Rejected.
- **`epher run script.md`** (the CLI evaluates fences too): plausibly
  wanted later, but every editor surface flows through the server
  already, and this ADR is about the editors. Not decided here.

## Consequences

- Diagnostics, inline answers, hover, completion, unit-aware semantic
  coloring, and whole-file runs work inside epher fences in every
  editor family that ships an LSP client today, with positions
  anchored in the markdown file.
- The `.epher` path stays byte-identical on the wire: no projection is
  built for it, and the existing integration tests double as the
  golden guard. The one deliberate wire change is semantic token
  lengths now counted in UTF-16 code units (the specification), which
  only shows on tokens containing non-ASCII text; that also fixes the
  same latent mis-highlight in plain `.epher` files.
- Feature edges stay honest per client: no Zed server features inside
  markdown; per-block runs (one fence alone), document symbols for
  fences, and code actions (extract a block to a `.epher` file) are
  named follow-ups, none of them blocked by this design.
- Staging order follows the house order: server and tests first, then
  the VS Code family pilot, then the graphical native clients
  (JetBrains, Visual Studio), then the text-first editors and Eclipse,
  each with its README one-liner.
