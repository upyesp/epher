# epher for Zed

**epher** is a calculator language: you write ordinary math, with units
that convert, and every statement's answer appears inline, right next
to the line that produced it.

Download the [epher calculator](https://epher.org), and a large
selection of [ready-made scripts](https://epher.org/scripts.html) from
epher.org.

![A script computing Earth's circumference, the discriminant of a quadratic, and a speed converted from miles to kilometers per hour, each line's answer shown inline](https://github.com/upyesp/epher/raw/HEAD/clients/zed/images/editor.png)

![The demo script typed live, each line's answer appearing as it completes](https://github.com/upyesp/epher/raw/HEAD/clients/zed/images/demo.gif)

Type a formula and the answer is already there. No runnable repl in a
side panel, no print statements: the editor *is* the calculator.

## What you get

- **Answers inline**: each statement's result renders next to its
  line: `x = 40 + 2` shows `= 42`.
- **Units that convert**: `6371 km`, `55 mile/hr`, `30 deg` are
  quantities, not comments. `speed in km/hr` converts; the answer
  carries the right unit.
- **Live diagnostics**: syntax errors point at the exact token, and
  evaluation errors carry the same message the epher calculator shows.
- **Hover signatures**: hover any name for its canonical signature;
  your own functions show their definitions, catalog functions show
  their docs.

![Hovering a defined name shows its signature and current value](https://github.com/upyesp/epher/raw/HEAD/clients/zed/images/hover.png)

- **Completion**: the whole catalog (math, astronomy, statistics),
  your own definitions, keywords, and snippets for the common
  statement shapes.

![Completion offers a catalog name with its documentation](https://github.com/upyesp/epher/raw/HEAD/clients/zed/images/completion.png)

- **Unit-aware coloring**: the shared tree-sitter grammar colors the
  language, and the server's semantic tokens refine it live: the `m`
  in `2 m` is a unit, not a variable. Turn semantic tokens on for the
  refinement (the settings below).

## Quick start

From Zed's extension gallery, once epher is published:

1. Open the command palette and run `zed: extensions`.
2. Search for **epher** and install it.
3. Open any `.epher` file, then set the two options below.

As a dev extension, today:

1. Clone this repository (or download and unzip `epher-zed.zip` from
   the [releases page](https://github.com/upyesp/epher/releases/latest)).
2. In Zed, open the command palette and run `zed: install dev
   extension`.
3. Select this directory (`clients/zed`).

Either way, turn semantic tokens and inlay hints on, or nothing will
be colored and no answers will appear:

```json
{
  "languages": {
    "epher": {
      "semantic_tokens": "full",
      "inlay_hints": { "enabled": true }
    }
  }
}
```

`full` lets the server's tokens refine the tree-sitter coloring, with
the unit suffix styled by meaning. The extension ships
`semantic_token_rules.json` for the custom `unit` token type; the
standard token types use Zed's built-in rules.

## Running a script

Zed's extension API has no commands, no panels, and no webviews, so
there is no run button in the editor. Short scripts need none: the
inline answers stay live beside every statement as you type. Whole
script runs happen in Zed's integrated terminal with the calculator:

```sh
epher run my-script.epher
```

The run prints the same transcript and writes every `graph save` plot
exactly as everywhere else.

![The script running in Zed's integrated terminal, the transcript matching the inline answers](https://github.com/upyesp/epher/raw/HEAD/clients/zed/images/results.png)

## Requirements

- Zed 0.192 or newer. The extension builds against `zed_extension_api
  0.6.0`, the API version that release train introduced.
- Linux x86_64 and ARM64, macOS Apple silicon, or Windows x86_64.
- The dev-extension install compiles the extension with Zed itself,
  which needs Rust and the `wasm32-wasip2` target (Zed adds the target
  through rustup when it is missing) and resolves a wasi-sdk toolchain
  for the declared grammar. Gallery installs are prebuilt and need no
  toolchain.
- First use needs the network once, to fetch the server binary for
  your platform. After that everything is local.

## Troubleshooting

Check the two settings first: a fresh extension recognizes `.epher`
files, but with `semantic_tokens` off and inlay hints disabled, the
buffer colors nothing and shows no answers.

If the extension does not appear under Installed after a dev install,
the answer is in Zed's log:

- Windows: `%LOCALAPPDATA%\Zed\logs\Zed.log`
- macOS: `~/Library/Logs/Zed/Zed.log`
- Linux: `~/.local/share/zed/logs/Zed.log`

Search it for `epher`. Load failures are log-only in Zed; they never
surface in the UI.

## Data and telemetry

None. Everything evaluates on your machine.

## License

[MIT](https://github.com/upyesp/epher/blob/main/LICENSE)

---

<details>
<summary><strong>Building and contributing (extension developers)</strong></summary>

The extension declares the shared grammar for the registry
(`[grammars.epher]`, pinned by revision to
[tree-sitter-epher](https://github.com/upyesp/tree-sitter-epher)):
Zed's packaging CI compiles it with its own wasi-sdk toolchain, so a
gallery install ships a finished package. A dev-extension install pays
that cost locally instead: Zed resolves wasi-sdk before anything else
(the 0.5.47 field reports of `wasi-sdk ... ENOENT` were that stage),
so a dev install needs the toolchain reachable or it fails.

Verify a build by hand:

```
cargo build --target wasm32-wasip2 --release
```

`wasm32-wasip2` is the target Zed compiles dev extensions with, and
Zed adds the target through rustup when it is missing. The version in
`extension.toml` and `Cargo.toml` is kept in lockstep and must match
the `v<version>` release tag; CI fails the build when the locks drift.

</details>
