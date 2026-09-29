# Zed listing captures — how they were taken (2026-09-29)

All five deliverables in `clients/zed/images/` are exactly 1244x900, window-only
captures of Zed stable rendering on the headless rig (display :99, Xvnc
1920x1200x24, Mesa 25.0.7 software Vulkan, `ZED_ALLOW_EMULATED_GPU=1`). Dark
theme = Zed's default One Dark; buffer font size 15. No image was resized
except the GIF frames, which were already 1244x900.

## What was installed

- **Zed stable 1.21.0** (`/home/pete/.local/bin/zed`, build 33c95853).
- **epher extension 0.5.57** as a **dev extension**: command palette →
  `zed: install dev extension` → picked `/home/pete/code/epher/clients/zed`
  in the GTK folder chooser. Zed compiled it itself (wasm32-wasip2, ~9 s,
  `extension.wasm` written into the extension dir, covered by its
  .gitignore) and linked `~/.local/share/zed/extensions/installed/epher`
  back to the repo folder. Log line: `finished compiling extension
  "/home/pete/code/epher/clients/zed" in 9.16s`.
- **epher-lsp 0.5.57**, downloaded by the extension itself on first
  activation from the GitHub release `v0.5.57` into
  `~/.local/share/zed/extensions/work/epher/epher-lsp-0.5.57/epher-lsp`
  (the 0.5.49 per-version-dir fix confirmed working). Language server
  start is in Zed.log at `07:35:50`.
- Settings actually required (in `~/.config/zed/settings.json`) — without
  these the buffer colors nothing and shows no answers:

```json
{
  "theme": "One Dark",
  "inlay_hints": { "enabled": true },
  "buffer_font_size": 15,
  "languages": {
    "epher": {
      "semantic_tokens": "full",
      "inlay_hints": { "enabled": true }
    }
  },
  "show_completion_documentation": true
}
```

The canonical `demo.epher` from `docs/research/capture-rig.md` lives in
`~/zed-demo/`. The demo project worktree must be **trusted** (Zed opens
single files in Restricted Mode; the language server does not start until
you click through the trust dialog).

## The shots

- `editor.png` — hero. Whole 1244x900 window, demo.epher open, all eight
  statement lines carrying live inline answers (`= 6371 km` … `= 1`),
  semantic-token coloring on, cursor parked at the end of
  `graph sin(x)`.
- `hover.png` — mouse hovered over `height` (line 12); popup reads
  "height: a variable in this document" plus the current value `= 4 m`.
- `completion.png` — new line at end of buffer, typed `sq`, `ctrl+space`;
  menu shows `sqrt`, `september_equinox`, `chisq_gof` from the catalog.
- `results.png` — see below.
- `demo.gif` — 9 frames (empty buffer → lines 1–7 typed one per frame →
  settled last frame), typed live with `xdotool type --delay 40`, one
  frame captured after each line completes, answers appearing as inlay
  hints; assembled with `convert -delay 50 -loop 0`.

## results.png, honestly

The extension exposes **no run surface**: Zed's extension API has no
commands, panels, or webviews (ADR-0069; see clients/zed/README,
"Running scripts, honestly"). So results.png shows the run surface Zed
users actually have per the README: the **integrated terminal** running
the calculator over the same file —

```
/home/pete/code/epher/target/release/epher-cli /home/pete/zed-demo/demo.epher
```

— printing the same transcript the editor shows inline (`= 6371 km` …
`= 1`, `graph: sin(x)`), with the editor and its inline answers still
visible above. It is a real terminal run, not a mock pane.

## Quirks a re-shoot must know

1. **xdg-desktop-portal must be running** or `install dev extension`
   silently shows no folder chooser: `XDG_RUNTIME_DIR=/run/user/1000
   DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus /usr/libexec/xdg-desktop-portal`
   and `/usr/libexec/xdg-desktop-portal-gtk` (both were started by hand
   here). In the chooser, navigate Home → code → epher → clients, select
   `zed`, press Select. The location bar (ctrl+L) flakes over xdotool;
   clicking through is deterministic.
2. **Inlay hints are hidden on the cursor's own line.** The answer for a
   line appears once the cursor moves off it (hint refresh debounce is
   ~1–2 s after the edit). For demo.gif each frame is therefore captured
   after pressing Return (cursor already on the next line), never before.
   Same trick applies to editor.png: park the cursor on `graph sin(x)`,
   which produces no answer, so every answered line is visible.
3. **Completion docs arrive as LSP `detail` strings** (see
   crates/lsp/src/analysis.rs — `detail: signature + description`), so
   Zed renders them truncated in the menu rows and shows no separate docs
   pane. The popup in completion.png is exactly what a user gets; do not
   wait for a docs pane.
4. **Comment auto-continuation**: Zed inserts `// ` on Return after a
   comment line (epher has no grammar, but the language config declares
   line comments). For the typed demo.gif, `"extend_comment_on_newline":
   false` was set during capture and reverted afterwards; a human typing
   the demo would hit this too.
5. `/usr/bin/epher` on this box is a stale **0.5.38** with no `run`
   subcommand. The terminal run in results.png uses the repo release
   build `target/release/epher-cli` (0.5.57, matching the pinned server
   train). A future box should install 0.5.57 to PATH and shorten the
   command.
6. Zed's install-time compile rewrites the `epher-zed` version entry in
   `clients/zed/Cargo.lock` (it was stale at 0.5.49 against a Cargo.toml
   at 0.5.57). The lock was reverted after capture to keep the tree as
   found; expect the same one-line rewrite on a re-shoot.
7. The trusted-worktree prompt and the first-run onboarding (theme/keymap
   page) both need clicking through on a fresh data dir; the update nag
   ("Failed to Update", no rsync on this box) appears in the title bar
   until dismissed.
