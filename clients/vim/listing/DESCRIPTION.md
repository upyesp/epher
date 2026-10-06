# Listing description for the Vim Scripts page (vim.org)

Everything below the `---` separator is plain-text listing copy: it
goes into the description field of the vim.org script form, with the
install block doubling as the separate "install details" field. The
site preserves whitespace and turns bare URLs into links, but renders
no HTML and has no image field, so there are no embeds here; the
captures it points at live in the repository.

---

epher is a calculator language: you write ordinary math, with units
that convert, and every statement's answer appears inline, right next
to the line that produced it.

This package teaches Vim the language. It adds filetype detection for
.epher files, syntax highlighting for comments, strings, every number
form (decimal, scientific, 0b, 0o, 0x, imaginary i), the twenty
keywords, and the conservative unit-suffix rule that mirrors the
parser's adjacency rule. The ftplugin settings cover # comments
(commentstring) and _ in iskeyword, so x1 and sin_x stay whole. The
same files work in Neovim.

What Vim gets on its own: the highlighting and filetype settings
above, in any .epher file, with nothing to configure. The inline
answers, diagnostics, hover, completion, and script runs come from the
shared epher-lsp language server and need a client for it. Vim reaches
that server through the vim-lsp plugin (with its async.vim
dependency); with both installed and epher-lsp on your PATH, register
it once in your vimrc:

    if executable('epher-lsp')
      au User lsp_setup call lsp#register_server({
            \ 'name': 'epher',
            \ 'cmd': {server_info -> ['epher-lsp']},
            \ 'allowlist': ['epher'],
            \ })
    endif

That wiring adds the language server features to Vim: diagnostics,
hover, completion, and the :EpherRun results pane. The inline answers
are inlay hints; vim-lsp renders them when g:lsp_inlay_hints_enabled
is set and Vim runs as Vim9 with virtual text. Neovim has its native
glue in the repository's clients/nvim directory; its README has the
setup lines.

Required prerequisite: the epher-lsp language server. Without it,
.epher files get highlighting only — the inline answers, diagnostics,
hover, completion, and script runs all come from the server. Download
the build for your operating system from the releases page and put it
on your PATH:

https://github.com/upyesp/epher/releases

Per-platform server-install instructions are on the epher website, in
the Any LSP client section of the IDE Extensions page:

https://epher.org/ide.html#any-ide

Install:
- copy the ftdetect, syntax, and ftplugin directories into ~/.vim/
  (~/vimfiles on Windows, $VIM/vimfiles for a system-wide install);
- or download epher-vim.zip from
  https://github.com/upyesp/epher/releases and extract it into
  ~/.vim/pack/epher/start/, where Vim picks up the epher-vim
  directory automatically;
- or with a plugin manager, point it at the directory, or add
  "set runtimepath+=/path/to/epher-vim" to your vimrc.

Usage: open any .epher file. Vim detects the filetype and applies the
syntax automatically, and :EpherRun runs the script once the language
server is attached.

What it looks like: the repository has terminal captures of the inline
answers, hover, completion, and the run results at
https://github.com/upyesp/epher/tree/main/clients/vim/images

epher.org has the standalone calculator and a library of ready-made
scripts: https://epher.org
