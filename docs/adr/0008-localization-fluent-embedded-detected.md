# UI localization: Fluent catalogs embedded at build time, device-detected, user-overridable

- Status: accepted
- Date: 2026-08-13

UI translations are build-time resources, not user data. Message catalogs live in
the repo as Fluent (`.ftl`) files and are compiled into each frontend artifact
(native binary for the CLI/TUI, WASM bundle for the web/PWA/desktop app) at build
time, so every frontend is offline-complete as a single artifact. The total
catalog is only tens of kilobytes, so bundling beats fetching.

A shared i18n crate (`fluent` + `fluent-langneg` + `unic-langid`) holds the
catalogs and locale negotiation and compiles to both the native and
`wasm32-unknown-unknown` targets; only the thin locale-detection layer differs
per frontend (`sys-locale` on native, `navigator.languages` on web). The UI
detects the device's current language and uses it, with a per-user `language`
Setting in the Store as an override. English is the default and the
always-complete fallback. The scripting language itself is never localized
(ADR-0007).

v1 ships six locales — English (`en`), Mandarin Chinese (`zh-CN`), Hindi (`hi`),
Spanish (`es`), French (`fr`), and Arabic (`ar`). The set covers the five most
widely spoken languages plus Arabic, which brings right-to-left support into v1.
Layout uses CSS logical properties and a root `dir`/`lang` driven by the active
locale.

> **Amendment (v0.3.x, 2026-08-17):** German (`de`) and Portuguese (`pt`)
> joined the locale set — the guide, landing page, and app catalogs ship in
> eight languages. The mechanism is unchanged: the code list in
> `SUPPORTED_LOCALES` is the single source of truth the CLI, shell, and all
> frontends validate against.
