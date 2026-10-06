# epher for Vim and Neovim

Ready-made configuration files that teach the editors the epher
language and wire the shared `epher-lsp` language server (ADR-0066
ships these as configs and documentation, not plugins). The syntax
and filetype rules live in this directory once and work in both
editors; Neovim adds its native LSP glue under `clients/nvim/`.

## What each file does

- `ftdetect/epher.vim`: makes every `.epher` file an `epher` buffer.
- `syntax/epher.vim`: the twenty keywords, all number forms
  (decimal, scientific, `0b`/`0o`/`0x`, imaginary `i`), strings with
  the five escapes, the three comment forms, and a conservative
  unit-suffix rule that mirrors the parser's adjacency rule.
- `ftplugin/epher.vim`: `commentstring` for `#` comments, `_` in
  `iskeyword`, undo-safe.

## Install: Vim

Copy the directory (or symlink it) onto your runtime path:

```vim
" in your .vimrc, before plugin loading
set runtimepath+=/path/to/epher/clients/vim
```

or copy the three subdirectories into `~/.vim/`.

## Install: Neovim

Add the directory to your runtimepath, then use the LSP glue from
`clients/nvim/` (see that README):

```lua
vim.opt.rtp:append("/path/to/epher/clients/vim")
```

## Wiring the language server (Vim)

Vim has no built-in LSP client, so the documented path is the
`vim-lsp` plugin (with its `async.vim` dependency). With both
installed and `epher-lsp` on your PATH:

```vim
if executable('epher-lsp')
  au User lsp_setup call lsp#register_server({
        \ 'name': 'epher',
        \ 'cmd': {server_info -> ['epher-lsp']},
        \ 'allowlist': ['epher'],
        \ })
endif
```

Required prerequisite: the `epher-lsp` language server. Without it,
.epher files get highlighting only — the inline answers, diagnostics,
hover, and `:EpherRun` all come from the server. Download the build
for your operating system from the
[releases page](https://github.com/upyesp/epher/releases), uncompress
it, and put it on your PATH or name the path in the `cmd` line above.

Per-platform server-install instructions live on the epher website, in
the [Any LSP client section](https://epher.org/ide.html#any-ide) of
the IDE Extensions page.

## Running scripts

With `vim-lsp` installed, `:EpherRun` runs the whole script through
the same `epher/run` request the VS Code results pane uses (ADR-0069)
and opens a results pane beside the script: every answer and error,
a row per statement. `<CR>` on a row jumps to its statement, `<CR>`
on a graph row reopens its SVG (written under
`~/.cache/epher/runs` and opened with the system viewer), and `r`
re-runs. Without `vim-lsp` the command says exactly what is missing;
the syntax layer above works either way.

## What it looks like

![A script computing Earth's circumference, the discriminant of a quadratic, and a speed converted from miles to kilometers per hour, each line's answer shown inline](https://github.com/upyesp/epher/raw/HEAD/clients/vim/images/editor.png)

![The demo script typed live, each line's answer appearing as it completes](https://github.com/upyesp/epher/raw/HEAD/clients/vim/images/demo.gif)

![Hovering a defined name shows its signature and current value](https://github.com/upyesp/epher/raw/HEAD/clients/vim/images/hover.png)

![Completion offers a catalog name with its documentation](https://github.com/upyesp/epher/raw/HEAD/clients/vim/images/completion.png)

![The results pane after :EpherRun, with the transcript and the graph row](https://github.com/upyesp/epher/raw/HEAD/clients/vim/images/results.png)

## The conservative rule

The vim syntax file is a static approximation: the exact unit
coloring comes from the language server's semantic tokens where the
editor supports them (Neovim does not consume LSP semantic tokens
into its syntax engine today, so the regex approximation is what
you get). Everything here errs toward matching the parser's real
rule: an identifier immediately after a number, no separator, never
`i`, never a call.
