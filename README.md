# epher

**A programmable, scriptable calculator** — one calculation engine, five ways to use it: command line, interactive REPL, full-screen TUI, desktop app, and an offline web app. Type expressions, save functions and scripts, graph results in 2D or 3D, animate constants, and keep history between sessions and user interfaces.

[![GitHub stars](https://img.shields.io/github/stars/upyesp/epher?style=social)](https://github.com/upyesp/epher/stargazers)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Last commit](https://img.shields.io/github/last-commit/upyesp/epher)](https://github.com/upyesp/epher/commits/main)
[![Latest release](https://img.shields.io/github/v/release/upyesp/epher?style=flat)](https://github.com/upyesp/epher/releases/latest)
[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

<p align="center">
  <a href="https://epher.org"><img src="https://img.shields.io/badge/Web_App-epher.org-2dd4bf?style=for-the-badge&logo=googlechrome&logoColor=white" alt="Web App"></a>&nbsp;
  <a href="https://epher.org/guide/en/"><img src="https://img.shields.io/badge/User_Guide-epher.org%2Fguide%2Fen-2dd4bf?style=for-the-badge&logo=readthedocs&logoColor=white" alt="User Guide"></a>&nbsp;
  <a href="https://epher.org/examples.html"><img src="https://img.shields.io/badge/Examples-epher.org%2Fexamples-2dd4bf?style=for-the-badge&logo=copy&logoColor=white" alt="Examples"></a>
</p>

<p align="center">
  <a href="https://github.com/upyesp/epher/releases/latest/download/epher-windows-x86_64.exe"><img src="https://img.shields.io/badge/Download-Windows_(.exe)-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Download Windows"></a>&nbsp;
  <a href="https://github.com/upyesp/epher/releases/latest/download/epher-macos-aarch64.dmg"><img src="https://img.shields.io/badge/Download-macOS_Apple_Silicon-000000?style=for-the-badge&logo=apple&logoColor=white" alt="Download macOS"></a>&nbsp;
  <a href="https://github.com/upyesp/epher/releases/latest/download/epher-linux-x86_64.deb"><img src="https://img.shields.io/badge/Download-Linux_(.deb)-FCC624?style=for-the-badge&logo=linux&logoColor=black" alt="Download Linux"></a>
</p>

---

## What It Does

- **One download, five frontends** — every installer carries the unified `epher` binary: one-shot command, REPL, piped scripts, TUI, and desktop app, plus the web app in your browser
- **A real language** — variables, functions with recursion, loops, and saveable scripts; newlines and `;` separate statements
- **Exact when it matters** — binary floats by default, with exact `frac`, `dec`, and `big` layers one call away
- **Graphs in 2D and 3D** — curves and surfaces, trace, points of interest, animated constants with a play button, SVG export
- **Private by design** — no accounts, no telemetry, no cloud; everything computes and stores on your device
- **Eight languages** — English, Chinese, Hindi, Spanish, French, German, Portuguese, Arabic, with right-to-left support
- **Accessible** — WCAG 2.2 AA throughout (keyboard-only use, visible focus, recorded contrast)

## Quick Start

Grab an installer from the badges above, or run straight from source:

```bash
git clone https://github.com/upyesp/epher.git
cd epher
cargo run --release -- "2 + 3 * 4"   # one-shot calculation
cargo run --release -- repl          # interactive session
cargo run --release -- tui           # full-screen terminal UI
cargo run --release -- gui           # desktop app
```

Open [epher.org](https://epher.org) for the web app, the [user guide](https://epher.org/guide/en/), and copyable [examples](https://epher.org/examples.html). The command's own help pages everything: `epher --help`, `epher help`.

Every installer ships the whole [script collection](https://epher.org/scripts.html) (333 ready-to-run scripts), installed beside the program. The same script, on each operating system:

```sh
# Debian, Ubuntu, Fedora (deb, rpm)
epher /usr/lib/epher/scripts/astronomy/moon/full-moons.epher

# Windows (PowerShell)
epher "$env:LOCALAPPDATA\epher\scripts\astronomy\moon\full-moons.epher"

# macOS
epher /Applications/epher.app/Contents/Resources/scripts/astronomy/moon/full-moons.epher
```

---

## More Information

- **[User guide](https://epher.org/guide/en/)** — the language, the frontends, and your data, in eight languages
- **[Examples](https://epher.org/examples.html)** — copyable code for every frontend
- **[Privacy](https://epher.org/privacy.html)** — what stays on your device (nothing leaves it)
- **[Releases](https://github.com/upyesp/epher/releases)** — downloads and changelogs
- **[Issues](https://github.com/upyesp/epher/issues)** — bug reports and feature requests
- **[Architecture decision records](docs/adr/)** — every design decision, documented
