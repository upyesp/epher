# ADR-0084: The macOS GUI hands off to the wrapper-free build

Date: 2026-11-11

Status: Accepted (amends ADR-0011's platform story: macOS now mirrors the
Windows GUI handoff; Linux keeps the in-process foreground window).

## Context

ADR-0011 ships one unified binary per OS that hosts every frontend. The
console entry (`crates/tauri-app/src-tauri/src/main.rs`) runs
`run_with_args` on a dedicated 512 MiB fat-stack thread (since the
archive-era commit f814446, 2026-10-08) because script evaluation
recurses per expression node and collection searches legitimately reach
hundreds of thousands of frames; the default 8 MiB main-thread stack
overflows first.

On macOS that wrapper thread cannot host the GUI: tao (the windowing
layer under tauri) panics unless the event loop runs on the process's
main thread (`tao 0.35.3`, `event_loop.rs`: "On macOS, `EventLoop` must
be created on the main thread!"). Windows never meets the problem
because ADR-0011's W2 split gives it a GUI-subsystem sibling that the
console build spawns detached; on Windows any thread may own a message
loop, so even the wrapper build could fall back to re-spawning itself.

Which binary actually contains the wrapper depends on the build tool,
and nothing in the tree documented that:

- `cargo tauri build` bundles the `default-run` binary
  (`epher-gui`, a wrapper-free `main()` that reaches
  `run_with_args` on the true main thread) renamed to `epher` by the
  tauri CLI's `rename_app` under `mainBinaryName`. Every shipped macOS
  artifact (the `.app`/DMG, and the Homebrew tarball that packages
  `target/release/epher` after that rename) is therefore wrapper-free
  and launches fine. Verified on v0.9.7: the shipped binary runs
  evaluation on thread `main`.
- A plain `cargo build` produces both binaries without the rename, so
  `target/release/epher` is the wrapper build. Its bare run panics at
  first window creation. This is how local dev builds, CI test
  artifacts, and any source reproduction of a bug report behave today,
  with a panic message that points at tao internals instead of the
  real requirement.

The thread identity is a reliable runtime discriminator: the wrapper
thread is spawned unnamed, the process main thread is named `main`.

## Decision

On macOS, `launch_gui` mirrors the Windows handoff, gated on the
wrapper rather than on the binary name:

- If `launch_gui` runs on a thread not named `main` (that is, inside
  the fat-stack wrapper) and without `EPHER_GUI_CHILD` set, it spawns
  the wrapper-free sibling `epher-gui` from the same directory (then
  the parent directory), passing the pending script path in argv and
  marking the child with `EPHER_GUI_CHILD=1`, and exits 0. The child
  reaches `run_with_args` on the main thread and opens the window
  in-process.
- If no sibling exists, the process prints an error that names the
  requirement (a wrapper-free build next to the binary, or a
  `cargo tauri build`) and exits 1. Unlike Windows there is no
  self-respawn fallback: re-executing the wrapper build would only
  re-create the problem being handed off, and a panic inside tao is
  the wrong answer either way.
- Wrapper-free binaries, including the tauri build's renamed `epher`,
  see a `main`-named thread and keep today's in-process window, so
  shipped macOS behavior is byte-for-byte unchanged (the `.app` smoke
  in CI exercises exactly that path).

Linux keeps the in-process foreground window: nothing there requires
the main thread.

## Consequences

- `cargo build --release` followed by a bare `./target/release/epher`
  opens the GUI on macOS again (via the sibling), and the cryptic tao
  panic is gone from every path that used to produce it.
- Terminal modes (CLI one-shot, REPL, piped scripts, the TUI) keep
  running on the fat-stack thread; they never touch the event loop and
  keep their deep-recursion headroom.
- The macOS sibling is not bundled or installed: tauri continues to
  ship the single renamed binary, and the handoff is a no-op for it.
  The sibling only exists next to plain-cargo builds, which is exactly
  where it is needed.
- The wrapper-free GUI path runs terminal modes on the default stack,
  as shipped builds always have; deep-recursion headroom for GUI
  sessions is unchanged (and unchanged is the point of this ADR).
- The build-tool invariant is now documented in the code (`main.rs`,
  `epher-gui.rs`, `launch_gui`) instead of living only in
  `Cargo.toml`'s `default-run` comment.
