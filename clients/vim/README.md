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

Getting the binary: download `epher-lsp-<target>` for your platform
from the releases page (`https://github.com/upyesp/epher/releases`),
uncompress it, and put it on your PATH or name the path in the
`cmd` line above.

## Running scripts

With `vim-lsp` installed, `:EpherRun` runs the whole script through
the same `epher/run` request the VS Code results pane uses (ADR-0069)
and opens a results pane beside the script: every answer and error,
a row per statement. `<CR>` on a row jumps to its statement, `<CR>`
on a graph row reopens its SVG (written under
`~/.cache/epher/runs` and opened with the system viewer), and `r`
re-runs. Without `vim-lsp` the command says exactly what is missing;
the syntax layer above works either way.

## The conservative rule

The vim syntax file is a static approximation: the exact unit
coloring comes from the language server's semantic tokens where the
editor supports them (Neovim does not consume LSP semantic tokens
into its syntax engine today, so the regex approximation is what
you get). Everything here errs toward matching the parser's real
rule: an identifier immediately after a number, no separator, never
`i`, never a call.
