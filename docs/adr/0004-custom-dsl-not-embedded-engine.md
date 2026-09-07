# Build a custom DSL instead of embedding a language engine

- Status: accepted
- Date: 2026-08-13

epher's scripting language is a custom domain-specific language with its own
lexer, parser, and evaluator — not an embedded engine (Rhai, Lua, or JS).

We chose to own the language so that LaTeX input, the layered numerics
(ADR-0005), graphing (ADR-0006), and the math-oriented grammar all share one AST.
Embedding would trade the problem we most want to control — the math/value model
— for a dependency whose number types and syntax fight those same goals. v1
targets "L2": expressions, variables, named functions with recursion, control
flow (`if`/`while`/`for`), and lists; closures, modules, and pattern matching are
deferred.

## Amendment (2026-08-18): newlines and `;` are one separator

The language grew two seams: `;` inside a script line, and the newline at
the input layer (each line submitted separately). Users had to remember
which surface wanted which. Because the language has no strings, comments,
or constructs that span lines, a newline can only ever appear *between*
statements — so the tokenizer now emits the semicolon token for `\n` and
`\r`, making newlines and `;` exactly the same separator everywhere.
Redundant separators (blank lines, `;;`) are skipped. Consequences:

- Every surface accepts a free mix: multi-line pastes, `;`-joined lines,
  and combinations (`x = 1;` on one line of a pasted block).
- The one-shot CLI runs scripts: `epher "x = 10; x + 5"` prints each
  statement's value on its own line (`10`, then `15`) — the piped mode's
  output without the `=` prefix.
- A statement never spans lines; `parse("1 +\n2")` is an error.
- Frontends still split input into statements for dispatch (graph/shell
  prefixes and per-statement history), now on both separators.
