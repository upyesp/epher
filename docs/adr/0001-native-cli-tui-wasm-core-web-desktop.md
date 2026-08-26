# Compile CLI and TUI natively; compile core, web, and desktop to WASM

- Status: accepted
- Date: 2026-08-13

We split compilation targets by where WASM's cost is justified. `epher-core`,
the Yew web frontend, and the Tauri desktop frontend compile to
`wasm32-unknown-unknown`; the CLI and TUI compile natively and link
`epher-core`. App logic is not duplicated — every frontend links the same core
crate.

We rejected compiling the CLI and TUI to WASM: it would force a bespoke WASI
terminal host and a `wasmtime` runtime dependency on end users, with no
user-visible benefit over a native binary. Native CLI/TUI still share all logic
via the core crate, so "logic exists once" is preserved.
