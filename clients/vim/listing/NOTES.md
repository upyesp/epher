# Vim captures (clients/vim/images/)

Taken 2026-09-29 on the build box, display `:95` (Xvfb 1920x1200x24),
with the system Vim and the client's pinned server, `epher-lsp 0.5.57`
(`target/release/epher-lsp`, built from this tree).

Terminal-authentic: Vim inside an xterm window, xterm's own frame and
font handling, Vim's built-in dark colors on a dark `-bg`. Launch line:

```sh
LANG=C.UTF-8 xterm -fa 'DejaVu Sans Mono' -fs 13 \
  -geometry 113x41+2+2 -bg '#101010' \
  -title 'vim ~/vim-demo/demo.epher' \
  -e "env TERM=xterm-256color LANG=C.UTF-8 \
      PATH=/tmp/vim-rig-bin:/home/pete/code/epher/target/release:$PATH \
      vim -u ~/vim-demo/vimrc ~/vim-demo/demo.epher"
```

The window was resized with `xdotool windowsize` to exactly 1244x900
before capturing (`import -window <id>`, no window manager frame), so
every deliverable is 1244x900 with no rescaling. Input was
`xdotool type --delay 40` and `xdotool key`, with `Ctrl+L` and a
settle pause before each snapshot (xterm repaint lag).

## Editor and plugins

- `/usr/bin/vim`: VIM 9.0 (2022 Jun 28, compiled 2025 Feb 16), the
  Debian huge build, patches 1-1378, 1499, 1532, 1848, 1858, 1873,
  1969, 2142. The features that matter here: **`+textprop`** and
  `+popupwin` (vim-lsp renders inlay hints as virtual text, which
  needs Vim9 with patch 9.0.0167+), `+vim9script`, `+signs`,
  `+terminal`; `-clientserver` and `-clipboard`, so the session is
  driven entirely through `xdotool`.
- `/tmp/vimlsp/vim-lsp` at `bbffa60` (2026-09-02) with its
  `async.vim` dependency at `2082d13` (2022-04-05), shallow clones
  from GitHub.
- epher-lsp 0.5.57, the repo binary, named by absolute path in the
  `lsp#register_server` `cmd`.

## vimrc used (`~/vim-demo/vimrc`)

```vim
" Capture vimrc for the epher Vim captures (clients/vim/images/).
" Minimal real-client setup: the install block from clients/vim/README.md
" plus the vim-lsp plugin pair, nothing synthetic.
set nocompatible
set background=dark

" clients/vim on the runtimepath, then the two plugins.
set runtimepath+=/home/pete/code/epher/clients/vim
set runtimepath+=/tmp/vimlsp/vim-lsp
set runtimepath+=/tmp/vimlsp/async.vim

syntax on
filetype plugin indent on

" Dark popups: Vim's built-in Pmenu default is magenta (the classic
" vim completion-menu pink). Every real setup overrides it; one dark
" pair covers the hover float and the completion menu.
highlight Pmenu ctermbg=235 ctermfg=250 cterm=NONE
highlight PmenuSel ctermbg=238 ctermfg=255 cterm=NONE
highlight PmenuSbar ctermbg=236
highlight PmenuThumb ctermbg=244

" The full vim-lsp feature set the listing promises.
" menuone so the menu shows even for one match; noinsert/noselect so
" nothing lands in the buffer until a match is picked (the vim-lsp
" documentation float follows the selection).
" Send didChange immediately instead of vim-lsp's default 1s queue, so
" the inlay-hint request that follows a change sees the changed text
" (otherwise every answer trails one cursor-nudge behind the line).
let g:lsp_use_event_queue = 0

set completeopt=menuone,noinsert,noselect
let g:lsp_diagnostics_enabled = 1
let g:lsp_diagnostics_signs_enabled = 1
let g:lsp_diagnostics_float_cursor = 0
let g:lsp_diagnostics_echo_cursor = 0
let g:lsp_diagnostics_virtual_text_enabled = 0
let g:lsp_inlay_hints_enabled = 1
let g:lsp_inlay_hints_delay = 150
let g:lsp_completion_documentation_enabled = 1
let g:lsp_completion_documentation_delay = 80

" The README's lsp#register_server block (cmd = the repo binary).
if executable('/home/pete/code/epher/target/release/epher-lsp')
  au User lsp_setup call lsp#register_server({
        \ 'name': 'epher',
        \ 'cmd': {server_info -> ['/home/pete/code/epher/target/release/epher-lsp']},
        \ 'allowlist': ['epher'],
        \ })
endif

" Vim's own LSP bindings used for the captures.
autocmd FileType epher setlocal omnifunc=lsp#complete
nnoremap K :LspHover<CR>
```

No client code was modified; `demo.epher` is the canonical file from
`docs/research/capture-rig.md`, unmodified.

## What each shot shows

- **editor.png** - the hero. `demo.epher` open, server attached,
  **the inlay hints render**: every answerable line carries its inline
  answer as Vim virtual text: `= 6371 km`, `= 12742000 m`,
  `= 40030173.592 m`, `= 24873.5966904 mile`, `= 3 m`, `= 5 m`,
  `= 4 m`, `= 1`. `graph sin(x)` has no inline answer (it produces the
  SVG in the results pane). Vim's built-in colors under a 256-color
  xterm; a single-window Vim with the default `laststatus=1` shows no
  statusline, which is why the bottom is just `~` lines.
- **hover.png** - cursor on `height` (`12,1`), `K` (mapped to
  `:LspHover`) opened vim-lsp's floating hover
  (`g:lsp_preview_float` default 1): `height: a variable in this
  document` and the current value `= 4 m`, in the dark `Pmenu` colors.
- **completion.png** - insert mode on a fresh line after the script,
  `sq` typed, `<C-x><C-o>` (`omnifunc=lsp#complete`) opened the
  popup: the single item `sqrt` with kind `function`. With
  `noinsert`/`noselect` the buffer still held `sq`; `<C-n>` selected
  the item (and vim-lsp inserted it) and **the completion
  documentation float shows the server's `detail`**:
  `sqrt(q): square root; negative real s fall back to complex;
  quantities need even dimensions` (the client wraps the float, hence
  the broken word). The `E>` sign is the diagnostics layer reacting to
  the incomplete identifier (`unknown name: sqrt` - a bare function
  reference is not a value); diagnostics signs are on because the
  listing promises them.
- **results.png** - `:EpherRun`: the script's own results pane
  (`rightbelow vnew`) beside the script, with the transcript rows
  `L2 ... L18 graph: sin(x)` and the `Graphs (1, each opened with the
  system viewer):` section naming the written SVG under
  `~/.cache/epher/runs/`. The same `epher/run` response the VS Code
  pane uses (ADR-0069). Note the `L1/L6/L9/L14/L15 v:null` rows: the
  server sends `display: null` for comments and `def` lines and this
  client renders every statement that carries a `display` key (the
  nvim client filters them); the shot keeps the client's real output.
  `<CR>` on a row jumps to its statement, `<CR>` on the graph row
  reopens the SVG, `r` re-runs.
- **demo.gif** - `demo.gif` typed live from an empty buffer in the
  same file (never saved), one frame after each line's answer settled:
  12 frames, `convert -delay 50 -loop 0`, infinite loop. Frame 0 is
  the empty buffer; then the first comment; `radius` + `= 6371 km`;
  `diameter` + `= 12742000 m`; `circumference` +
  `= 40030173.592 m`; the blank line + second comment;
  `circumference in mile` + `= 24873.5966904 mile`; the triangle
  comment + `legs` + `= 3 m`; `hypotenuse` + `height` with `= 5 m` and
  `= 4 m`; the discriminant comment + `def` (no answer); `disc(1, -5,
  6)` + `= 1`; the blank line + `graph sin(x)`.

## Quirks (real client/rig behavior, not fakes)

- **Inlay hints only exist in normal mode.** vim-lsp clears hints on
  `InsertEnter` and its renderer returns unless `mode()` is `n`, and
  it asks for hints on `CursorMoved`/`CursorHold` after
  `g:lsp_inlay_hints_delay`. So while you type, the answers are
  invisible; they appear when you leave insert mode and move the
  cursor. The GIF frames are captured after `Esc` + a `k` nudge for
  exactly this reason.
- **Two debounces sit between a keystroke and its answer.**
  epher-lsp holds `didChange` for a 200ms quiet gap before it
  re-analyzes, and vim-lsp's own change queue defaults to a >=1s
  delay; the vimrc sets `g:lsp_use_event_queue = 0`. Even then the
  hint request can race the held edit, so the capture recipe waits
  ~0.9s after `Esc` before the cursor nudge that triggers the request.
  Without both, every answer trails one frame behind its line.
- **Vim's default `completeopt=menu,preview` inserts the first match
  the moment the menu appears.** The vimrc uses
  `menuone,noinsert,noselect`; with `noselect` the vim-lsp
  documentation float appears only once an item is selected
  (`CompleteChanged`), so the completion shot is taken after `<C-n>`.
- **Vim's built-in `Pmenu` default is magenta** (the classic vim
  completion-menu pink); the vimrc overrides it with dark colors, or
  the hover and completion floats would be unreadable on a dark rig.
- **xterm repaint lag under xdotool bursts.** Early takes captured
  pre-update frames (missing hint text, stale dividers); every final
  snapshot is preceded by `Ctrl+L` and a settle pause, and the
  double-snapshots were identical.
- **No `xdg-open` on this box.** `:EpherRun` hands every written graph
  SVG to the system viewer; a no-op `xdg-open` shim at
  `/tmp/vim-rig-bin/xdg-open` (first in `PATH`) keeps the run path
  quiet. The SVG is still written and its path is what the pane shows.
- **The `E>` sign in completion.png is a diagnostic sign**, not an
  error message from the editor; the completion request itself
  succeeded.
