# Desktop PWA bridges to the Native Store via the File System Access API

- Status: accepted
- Date: 2026-08-13

The browser-installed PWA is sandboxed off the host filesystem, so by default it
uses its own isolated Web Store (ADR-0002). To let a desktop PWA share the Native
Store, we add an opt-in bridge: on Chromium desktop browsers (Chrome, Edge,
Brave) the PWA uses the File System Access API to ask the user once to grant the
Native Store folder, persists that grant, and thereafter reads and writes the same
files as the CLI, TUI, and desktop app.

We rejected silent host access (impossible — browsers don't expose fixed host
paths to web origins) and a local sync daemon (needs the native app running;
fragile; adds a security surface). Firefox and Safari don't ship the File System
Access API, so on those the PWA stays on its isolated Web Store. Mobile and
other-browser PWAs always stay isolated. Sharing the same files across processes
requires atomic writes and reload-on-change — accepted as a downstream design
constraint because the shared data (preferences, scripts, functions) is small and
rarely written.
