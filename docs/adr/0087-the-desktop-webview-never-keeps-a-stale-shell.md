# ADR-0087: the desktop webview never keeps a stale shell

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-20

## Context

A Windows user updated the Microsoft Store build from 0.9.12 to 0.9.13.
The title bar (set natively, `env!("CARGO_PKG_VERSION")`, ADR-0032's
window) said 0.9.13, but quaternions - engine features of 0.9.12 and
later - were missing. Uninstalling and reinstalling the same version
restored them.

The desktop shell is Tauri: the app's HTML/JS/wasm ships inside the
binary (`frontendDist`), so an updated exe carries an updated shell.
The one mutable layer between them is the WebView2 profile, and the
web app registers a PWA service worker (`sw.js`) on every origin it
runs on, the desktop webview's `tauri.localhost` included. On Windows,
WebView2 does not raise its `WebResourceRequested` interception for
requests made from inside a service worker; the worker's network
fetches fall through to the real network stack, where
`http://tauri.localhost` has no server, so every fetch fails and the
worker answers from its `CacheStorage` cache. The cached shell never
expires: every launch renders the shell as it was at install time,
while the title bar happily reports the new binary. Uninstalling wipes
the profile, which is why the reinstall "fixed" it.

This is a silent, total functionality freeze masked by a fresh version
number - the worst kind of update bug.

## Decision

The desktop webview never runs the PWA service worker, and any worker
left by an older build is removed once per version change.

- **Registration guard.** `crates/web/index.html` skips
  `serviceWorker.register` when `window.__TAURI_INTERNALS__` is
  present (the same desktop-detection the boot-fallback script
  already uses). The desktop's assets live in the binary; a service
  worker there can only ever serve stale content. The PWA in a
  browser keeps the worker and its offline support unchanged.
- **One-time heal.** In the Tauri setup, the app writes a
  `shell-version` marker into `app_local_data_dir`. When the marker
  exists and differs from the running version - the first launch of a
  new build - the app calls `clear_all_browsing_data()` on the
  webview and reloads it, so profiles poisoned by an older worker
  heal on the first launch of the new build, without uninstalling.
- **No data loss beyond webview-local conveniences.** The user's
  history, functions, scripts, constants, session, language, and
  theme all live in the native store (ADR-0010), not in the webview.
  What resets is what the webview itself holds: the graph pane's
  localStorage toggles (ADR-0020) and boot-retry markers.

## Alternatives considered

- **Fix the service worker to clean up under Tauri.** Can't reach the
  poisoned population: their installed worker is the old code, and its
  own fetches never reach the updated assets, so no new worker or shell
  code can run there. Only the native side can act.
- **Delete the WebView2 Service Worker directories from disk.** The
  files are locked once the environment is created; the official
  `clear_all_browsing_data` is the supported cross-platform route.
- **Version probe with a JS-side self-reload.** A stale shell by
  definition runs stale JS; any heal written in the shell cannot save
  the stale shell itself.

## Consequences

- An MSIX, NSIS, or in-place update can no longer show a new version
  over old functionality; at most the first launch of a new build
  reloads once while the profile is cleared.
- The PWA's service worker story is untouched; browsers get their
  updates network-first as before.
- The webview-local toggles reset once per version change; the
  alternative (surgery on WebView2 internals) was rejected as
  unsupported.
- Windows note for verification: update a store or side-loaded install
  in place, confirm quaternion evaluation without reinstalling.
