# ADR-0013: The command line follows clig.dev

Date: 2026-08-16
Status: accepted

## Context

The unified `epher` binary (ADR-0011) grew its command surface ad hoc: a
positional expression, `-` for stdin, three subcommands, clap defaults.
Users coming from other modern tools expect familiar conventions —
`--help` behavior, a `help` command, `man` pages, sensible flags, errors
on stderr, meaningful exit codes. [clig.dev](https://clig.dev/) is the
community-maintained distillation of those conventions; adopting it makes
epher guessable for anyone who has used jq, git, or ripgrep.

## Decision

The CLI follows clig.dev, with clap (a library clig.dev itself recommends
for Rust) doing the parsing. The argument surface lives in `epher-cli`
(`dispatch` module) so the unified binary and any other binary parse
identically — one definition, no drift.

### Help and the manual

- **`-h` is concise and leads with examples** (jq-style): description,
  usage, four examples, the commands, and a pointer to `--help`.
- **`--help` is the full reference**: same lead, detailed argument help,
  then the tail — where data lives, documentation links, and the support
  path (issue tracker) clig.dev asks for.
- **`epher help` pages the manual**: if the system has an installed page
  (`man -w epher` succeeds), run `man epher` — the git/npm convention;
  otherwise print the full help (macOS app installs, Windows, anything
  without the page). `epher help <command>` prints that subcommand's
  help; unknown topics are usage errors (exit 2).
- **A real man page exists**: `packaging/man/epher.1` is generated from
  the clap surface (`cargo run -p epher-cli --example gen-man`) plus
  hand-written sections summarizing the user guide — the language, the
  built-in functions, shell commands, exit status, files, environment,
  examples. The deb and rpm bundles install it to
  `/usr/share/man/man1/epher.1` (`bundle.linux.deb/rpm.files`); regenerating
  is a documented one-liner whenever the surface changes.

### Streams and exit codes

- Results go to stdout; **diagnostics go to stderr** — including REPL and
  piped-script evaluation errors, which previously mixed into stdout.
  `printf "x=3\nx*10\n" | epher - | next-tool` now yields pure data.
- Exit codes: `0` success; `1` a failed calculation (a script keeps
  evaluating after an error line but still exits 1 — errors are never
  silent to scripts); `2` usage errors, via clap.
- `epher -` with a terminal on stdin refuses to hang: it explains that
  stdin mode expects a pipe and suggests `epher repl`, exit 2.
- Errors print in red when stderr is a terminal, honoring `NO_COLOR` and
  `TERM=dumb` (anstream, already in the tree via clap).

### Argument surface

- The positional expression stays (a calculator's primary action —
  CLIG's "common, primary action" exception), `allow_hyphen_values`
  keeps `epher "-2 + 5"` working, and `-` reads stdin (the standard
  convention). Expression and subcommands conflict explicitly rather
  than guessing.
- Bare `epher` still launches the GUI (interactive-by-default, the
  double-click path from ADR-0011 — clig.dev's explicit exception).
- No short flags beyond `-h`/`-V`: the surface stays small until a flag
  earns a letter.

## Consequences

- The shell layer (`Handled`) marks messages as diagnostics; the CLI
  decorates them with the `error:` voice while the TUI/GUI show the same
  messages inline unchanged.
- epher-cli's dev binary shares the dispatch (bare invocation now means
  GUI, as in the unified binary; `repl` is explicit).
- The man page is a committed artifact regenerated on demand; guide-first
  wording changes flow through `gen-man`, not just the page.
- Windows ships no man page (no man); `epher help` falls back to help
  text there — the same manual, unpaged.
