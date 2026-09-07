# ADR-0059: The TUI takes a paste as one paste, and Enter runs the whole entry

Date: 2026-09-08

Status: Accepted

## Context

A user copied a script from the website and pasted it into the TUI; the
same text pasted into the desktop app's entry ran fine. The reason is
how each frontend receives the paste.

The desktop and web entries are text areas: the paste lands in the
entry as text, the user presses Enter once, and `submit_line` — which
splits on newlines and semicolons the way the tokenizer does (comments
and string literals stay whole) — runs the whole program and records
one history entry.

The TUI had no paste concept. It never enables bracketed paste, so the
terminal delivers a paste as a burst of ordinary keystrokes, and each
pasted newline arrives as Ctrl+J — the terminal convention for line
feed — which the key arm treats as Enter. Every line of the paste
therefore submitted on its own, and a script's opening line,
`/* ============ ...`, died as an unterminated block comment. The
behavior was even deliberate (it mirrored the REPL and piped scripts),
but "line by line like the REPL" is right only for input that is lines;
a pasted file is one program.

## Decision

The TUI enables bracketed paste for the session (alongside mouse
capture, released with it) and handles `Event::Paste`: the clipboard
lands in the entry as one unit at the cursor, newlines and all, via a
new `App::paste_text` (which normalizes `\r\n` to `\n` and, unlike
typing, injects no `ans` for a leading operator — a paste is verbatim).
The next Enter runs the whole entry through the same `submit_line`
path the desktop and web entries use, so a script pasted from the
website behaves identically in every frontend: one paste, one Enter,
one transcript, one history entry.

The keystroke fallback stays: on a terminal without bracketed paste the
paste still arrives as key events, and Ctrl+J still submits, line by
line — degraded but not broken for one-line pastes. The guide view
stays modal (pasted text is swallowed there, like every key but
scrolling and closing), and a file-path prompt takes the pasted text
without its control characters, since a path is single-line.

## Consequences

- Scripts pasted from the website run in the TUI exactly as they run
  from a file: comments, blank lines, and statement groups survive,
  because the evaluation is the file-shaped one.
- The entry itself was already multi-line (Shift+Enter inserts a
  newline; the input scrolls its caret line into view), so a long paste
  renders and edits with no new widget work.
- Terminals without bracketed-paste support keep the old line-by-line
  behavior; every terminal this project's users have reported
  (Windows Terminal, iTerm2, GNOME Terminal, kitty, alacritty, tmux)
  supports it.
- The Edit menu's Paste item still shows its hint rather than reading
  the clipboard: a menu has no read-side clipboard API in a terminal;
  pasting remains the terminal's gesture (Ctrl+V / Cmd+V), which now
  does the right thing.
- Tested at the `App` seam: `paste_text` placement and cursor
  movement, CRLF normalization, no `ans` injection, and the whole
  paste-then-one-Enter flow against a script with comments and blank
  lines.
