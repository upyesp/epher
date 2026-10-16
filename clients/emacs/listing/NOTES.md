# Emacs captures (clients/emacs/images/)

Taken 2026-09-29 on the build box, display `:94` (Xvfb 1920x1200x24),
with **GNU Emacs 28.2** (`emacs-nox`, terminal) inside xterm, the
client's pinned server `target/release/epher-lsp` built from this tree
(0.5.57), and the canonical `demo.epher` from
`docs/research/capture-rig.md`, unmodified. No client code changed.

Emacs 28 predates built-in eglot (that lands in 29), so this session
uses the path the README names for older Emacsen: **eglot 1.24 from
GNU ELPA** (it still declares `emacs (26 3)`) installed with
`package.el` into `~/emacs-init/elpa`, plus **markdown-mode 2.8 from
NonGNU ELPA** — without it eglot has no `gfm-view-mode` and renders
hover as raw markdown (`**height**: …`). `epher.el`'s declared
`Package-Requires` floor (`emacs 29.1`) is enforced by package.el at
install time, not at load time; nothing in the file needs 29 here.

Terminal-authentic: `emacs -nw` in a bare xterm window (no window
manager, no decorations), xterm's own font handling, Emacs' default
dark faces (auto-detected from the terminal background via OSC 11).

Launch line:

```sh
DISPLAY=:94 xterm -geometry 113x40 -fa 'DejaVu Sans Mono' -fs 13 \
  -tn xterm-256color -bg black -fg white +sb -bw 0 \
  -xrm 'XTerm*internalBorder: 0' \
  -e sh -c 'exec emacs -nw -l "$HOME/emacs-init/init.el" "$HOME/emacs-demo/demo.epher"'
```

Geometry: DejaVu Sans Mono 13pt measures 11x22 px per cell here, so a
113x40 grid with a zero internal border is exactly 1243x880. Captures
are `import -window <id>` of the xterm; each is padded to exactly
1244x900 with a black `-extent` (1 px right, 20 px bottom, invisible
against the black terminal). **No scaling**: text is pixel-exact.
The menu bar row is Emacs' own default `-nw` chrome, kept as captured.

## init.el used (`~/emacs-init/init.el`)

```elisp
;;; init.el --- epher capture rig for Emacs 28.2 (emacs-nox) -*- lexical-binding: t; -*-

;; Emacs 28 predates built-in eglot, so eglot 1.24 and its deps (GNU ELPA)
;; plus markdown-mode 2.8 (NonGNU ELPA) live in this private package dir,
;; installed with package.el, exactly as a user of this Emacs would.
(setq package-user-dir (expand-file-name "elpa" "~/emacs-init"))
(require 'package)
(package-initialize)

;; The epher client from the repo.  epher-mode auto-starts eglot when the
;; server program is executable (`epher-lsp-autostart' is t by default).
(add-to-list 'load-path "/home/pete/code/epher/clients/emacs")
(require 'epher)

;; Point the client at the server built from this repo.
(setq epher-server-program '("/home/pete/code/epher/target/release/epher-lsp"))

;; Capture hygiene.  Fresh ELPA packages native-compile on first load and
;; the compile warnings pop a *Warnings* window over the frame; keep the
;; frame clean (the .eln cache makes later loads quiet anyway).
(setq native-comp-async-report-warnings-errors 'silent)
(setq warning-suppress-types '((comp) (bytecomp)))
(setq ring-bell-function #'ignore)
(blink-cursor-mode -1)
;; Hover: let the echo area grow to the whole doc instead of one line.
(setq eldoc-echo-area-use-multiline-p t)
```

`epher-mode` auto-starts eglot on open (the server binary is on the
customized `epher-server-program` full path). The mode line reads
`(epher Flymake[0 0] ElDoc)` and `[eglot:emacs-demo]` once attached.
`M-x server-start` was used to drive the rig over `emacsclient`
(position point, clear the echo area, size windows, re-request hints);
xdotool typing goes through XTEST after `windowfocus` (Emacs ignores
`XSendEvent` keys on this box).

## What each shot shows

- **editor.png** - the hero. `demo.epher` open, eglot attached, point
  at the top, no popups. Every answerable line carries its inline
  answer (eglot inlay hints, the server's `textDocument/inlayHint`
  replies): `= 6371 km`, `= 12742000 m`, `= 40030173.592 m`,
  `= 24873.5966904 mile`, `= 3 m`, `= 5 m`, `= 4 m`, `= 1`;
  `graph sin(x)` has no inline answer (it produces the SVG in the run
  transcript instead). Font-lock is `epher-mode`'s conservative layer
  (comments, keywords, numbers, units, call names) - eglot does not
  consume the server's semantic tokens, exactly as the README says.
- **hover.png** - point on `height` (12,1, the value hover for a user
  constant). The echo area shows eglot's hover line
  `height: a variable in this document`; the `*eldoc*` window below
  (sized to content) shows the complete hover doc the server sent:
  the same line plus `= 4 m`. Eglot deliberately puts only the first
  line in the echo area (`:echo` = first newline); the full doc lives
  in eldoc's doc buffer, which is what this shot adds.
- **completion.png** - a new line at the end of the file with `sq`
  typed; `M-x completion-help-at-point` opened the `*Completions*`
  window: `chisq_gof`, `september_equinox` and `sqrt`, each followed
  by eglot's annotation, which is the server's `detail` line
  (`sqrt(q): square root; negative reals fall back to complex;
  quantities need even dimensions`). The pink `!!` and red `sq` are
  Flymake showing the server's own diagnostic for the incomplete
  statement - live diagnostics, not an artifact. Eglot's completion
  matcher is flex, so `sq` also matches `chisq_gof` and
  `september_equinox` (subsequence), not just the prefix.
- **results.png** - `C-c C-c` (`epher-run`, ADR-0069: the client
  sends the same `epher/run` request the graphical clients use). The
  `*epher run*` transcript fills the frame: every statement with its
  display (`= 40030173.592 m`, `= 1`, `graph: sin(x)`), then
  `Graphs (1, each opened with the system viewer):` with the written
  SVG path under `~/.cache/epher/runs/`. The echo area carries the
  client's own message, `no desktop session found; graphs saved under
  …` - there is no xdg-open on this box, and a terminal Emacs cannot
  show the SVG; the transcript + path is the honest run surface.
- **demo.gif** - the earth / miles / discriminant section typed live
  from an empty buffer with `xdotool type --delay 40`, one frame
  after each line's answer landed (the hint overlay was verified
  present before every capture): 11 frames, `convert -delay 50 -loop 0`.
  Frame 01 is the empty, attached buffer; frames 02-10 add one line
  each; frame 11 is the settled final state. The file on disk was
  truncated for the take and restored to the canonical contents after.

## Quirks (rig, not client)

- **Inlay-hint refresh race on Emacs 28.** Eglot requests hints from
  jit-lock for just the edited region; epher-lsp evaluates a
  `didChange` asynchronously (~200 ms behind it) and answers the
  immediate `inlayHint` request with `[]`, and eglot never re-asks
  because the region is already fontified. On this box the GIF rig
  therefore re-requests hints over the whole document once per line
  (`eglot--update-hints-1`) after waiting for the evaluation, then
  captures; the hints shown are only ever the server's own replies.
  (On Emacs 29+'s built-in eglot the same code path refreshes on its
  own; this is a 28 + ELPA-eglot timing shape, not a client bug.)
- **Hover needs markdown-mode.** Eglot's rendering checks for
  `gfm-view-mode`; with markdown-mode installed the `**` markers and
  code fences are fontified/hidden, without it hover shows raw
  markdown. Eglot's own docs recommend markdown-mode for this.
- **Completion of a unique prefix is silent.** `sq` matches only
  `sqrt`-the-prefix, and `completion-at-point` inserts a sole match
  without a popup ("Sole completion"). The shot uses
  `completion-help-at-point`, the standard command that displays the
  `*Completions*` buffer without inserting anything.
- **`fit-window-to-buffer` deletes tiny windows** (it removed the
  3-line `*eldoc*` window); the doc pane was sized with
  `set-window-text-height` instead. Both are user-level window ops.
- **First-load native compilation pops `*Warnings*`** over the frame
  (byte-compile advice for the newer ELPA packages on 28); the cache
  was prewarmed and the two warning variables in init.el keep the
  frame clean.
- **No `xdg-open` on this box** (same as the nvim captures):
  `epher-run` takes its documented no-desktop branch and says so in
  the echo area; the graph SVG is still written and its path shown.
- The capture box runs parallel client tasks on other displays
  (:96-:99); `:94` was used only for this take.
