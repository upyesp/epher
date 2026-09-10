# epher architecture

One math engine, every interface. This document describes the runtime
architecture of epher — a programmable calculator whose engine is a
single Rust crate, compiled to native code for the terminal frontends
and to WebAssembly for the browser-based frontends — the desktop GUI's
interface is that same WebAssembly build, not a separate native UI. All user-facing
text lives in externalized Fluent locale files covering 8 languages.

> Drafted for the website but not yet linked there; edit freely.

## Overview

```mermaid
flowchart TB
    subgraph engine["Single math engine — crate epher-core (Rust)"]
        direction LR
        LEX["tokenizer"] --> PARSE["parser / statements"]
        PARSE --> EVAL["evaluator<br/>Env: constants, functions,<br/>number bases, ans"]
        PARSE --> GRAPH["grapher<br/>2D sampling + points of interest,<br/>3D surface sampling"]
        GRAPH --> SVG["SVG renderer<br/>(deterministic export)"]
    end

    subgraph svc["Shared services (Rust crates, one source of truth)"]
        SHELL["epher-shell<br/>command kernel: history, open/save,<br/>language, theme, plots"]
        STORE["epher-store<br/>DocStore persistence"]
        I18N["epher-i18n<br/>Fluent Localizer"]
        GUIDE["epher-guide<br/>guide renderers<br/>(markdown fetched on demand)"]
    end

    subgraph ui["User interfaces (the UI layer is externalized)"]
        CLI["CLI — epher-cli<br/>one-shot / piped / script"]
        REPL["REPL — interactive<br/>(same crate as CLI)"]
        TUI["TUI — epher-tui (ratatui)<br/>keypad, menus, history, plots"]
        WEBAPP["Yew frontend — epher-web<br/>the one graphical UI,<br/>Rust compiled to WebAssembly"]
        GUI["Desktop GUI — epher-gui<br/>native Tauri shell (window, webview,<br/>native bridge); hosts the Yew frontend"]
        PWA["PWA — the same Yew frontend<br/>served at epher.org<br/>(service worker + manifest)"]
    end

    subgraph lang["Languages — 8 externalized locales (.ftl)"]
        L8["ar · de · en · es · fr · hi · pt · zh-CN<br/>scripting language itself is never localized"]
    end

    subgraph plat["Platforms and packages (one engine binary per OS)"]
        WIN["Windows x86_64<br/>NSIS installer: epher.exe (console)<br/>+ epher-gui.exe (GUI subsystem)<br/>WebView2"]
        MAC["macOS (Apple Silicon)<br/>.dmg, WKWebView,<br/>Terminal=true/false dispatch"]
        LIN["Linux x86_64 (glibc 2.35+)<br/>deb · rpm · AppImage, WebKitGTK,<br/>man page, Terminal=false entry"]
    end

    engine --> SHELL
    SHELL --> STORE
    SHELL --> I18N
    I18N --> L8
    GUIDE --> I18N

    CLI --> engine
    REPL --> engine
    TUI --> engine
    CLI --> SHELL
    REPL --> SHELL
    TUI --> SHELL

    WEBAPP -->|wasm module| engine
    WEBAPP -->|wasm module| SHELL
    GUI -->|hosts| WEBAPP
    PWA -->|serves| WEBAPP
    GUI -->|native commands: save dialogs, store| SHELL

    CLI --- WIN & MAC & LIN
    TUI --- WIN & MAC & LIN
    GUI --- WIN & MAC & LIN
    PWA -.->|runs in any modern browser| WIN & MAC & LIN
```

## Compilation targets

```mermaid
flowchart LR
    CORE["engine crates — core, shell, store, i18n<br/>(pure Rust, no platform code)"]
    NAT["native build — cargo"]
    BIN["epher-gui binary — native code:<br/>CLI · REPL · TUI · Tauri shell<br/>(window, webview, native bridge —<br/>no UI of its own; it hosts the wasm bundle)"]
    WASM["WebAssembly build — trunk compiles the<br/>Yew app (epher-web) and the engine crates"]
    WEB["web bundle — JS glue + wasm:<br/>the Yew interface"]
    DESK["Tauri webview asset<br/>(web/dist, embedded at build time)"]
    PWA["PWA asset on epher.org"]
    CORE --> NAT --> BIN
    CORE --> WASM --> WEB
    WEB --> DESK
    DESK -.->|embedded into| BIN
    WEB --> PWA
```

## Explanatory notes

**The engine.** `epher-core` holds the tokenizer, parser, evaluator, and
both graphers (2D curve sampling with points-of-interest analysis, 3D
surface sampling), plus the deterministic SVG renderer. It has no UI
code and, since ADR-0037, exactly one platform read (`now()`'s clock) —
every frontend calls the same functions, so
`2 + 2` and `graph x ^ 2` mean the same thing in every interface.

**Shared services.** `epher-shell` is the command kernel used by every
frontend that has commands (history, open/save, `language`, `theme`,
plot assembly). `epher-store` persists documents (`DocStore<FsStore>`
→ `~/.epher/` on desktop and TUI, a browser `localStorage` adapter in
the PWA). `epher-i18n` is a Fluent `Localizer` over externalized `.ftl`
files. `epher-guide` is the user guide's renderers (markdown → HTML for
the web/desktop overlay, markdown → plain text for the TUI pager); the
markdown itself is not compiled into any binary — the web app fetches
`guide/<locale>.md` from its static files, and the TUI reads the
installed files when the guide opens (ADR-0053).

**Terminal frontends (native).** `epher-cli` is the one-shot/piped
expression evaluator and the REPL. `epher-tui` is the full-screen
ratatui app: keypad, menus, clickable history, 2D/3D plots. Both link
the engine as an ordinary Rust dependency.

**Browser frontends (WebAssembly).** `epher-web` is the Yew
single-page app — the one and only graphical interface. Trunk compiles
it, together with the engine, shell, store, and i18n crates, to
WebAssembly and bundles the result with the UI glue and a service
worker: that bundle *is* the PWA at epher.org **and** the desktop GUI.
The **desktop GUI** (`epher-gui`, the Tauri app) has no native UI of
its own: it contributes only a shell — window, system webview, and a
thin bridge of native commands (save dialogs, filesystem store) — and
embeds the same wasm bundle as its content (`frontendDist: web/dist`,
built by `trunk build --release`). The desktop calculator and the PWA
are two ways to serve one Yew codebase.

**One binary, three terminals.** On macOS and Linux a single
`epher-gui` binary decides by how it was launched (and by the
`Terminal=` desktop entry) whether to run the CLI/REPL, the TUI, or the
Tauri window. On Windows the NSIS installer ships two subsystem builds
of the same program: `epher.exe` for the console and `epher-gui.exe`
for double-click launching.

**Platforms.** Windows x86_64 via a themed NSIS installer (WebView2);
macOS Apple Silicon via an unsigned `.dmg` (WKWebView); Linux x86_64 as
deb, rpm, and AppImage built on glibc 2.35 (WebKitGTK), with a man page
and a `Terminal=false` desktop entry. The PWA runs anywhere a modern
browser does.

**Languages.** Every string the interfaces show comes from one of the
8 externalized Fluent locales (ar, de, en, es, fr, hi, pt, zh-CN) —
the UI is translated, the scripting language is deliberately never
localized, and diagnostics stay byte-identical across locales.
