; nsis-check.nsi — compile-only harness for nsis-hooks.nsh (ADR-0011).
;
; Verified with `makensis nsis-check.nsi` (Linux ships makensis; no wine
; needed — this checks syntax, macros, labels, and includes). The string
; logic itself is hand-traced in the hooks file; keep this harness in sync
; so the hook file always compiles against the same NSIS feature set the
; Tauri installer uses (LogicLib, no plugins).
;
; Structure mirrors Tauri's installer.nsi consumption of the hooks file:
;   - the template `!include`s the hooks file whole at the top (line ~35),
;   - NSIS_HOOK_POSTINSTALL is inserted inside `Section` (installer ctx),
;   - NSIS_HOOK_POSTUNINSTALL is inserted inside `Section Uninstall`
;     (uninstaller ctx — where only `un.` functions may be Called).
; Expanding both macros in their real contexts is what catches the
; installer/uninstaller function-context mismatch.
;
; Run from the repo: makensis crates/tauri-app/src-tauri/nsis-check.nsi

!include "LogicLib.nsh"
OutFile "nsis-check.exe"
Name "epher nsis check"

; The harness compiles from src-tauri; the real installer.nsi compiles from
; target/<triple>/release/nsis/<arch>. Override the console-exe source with
; a repo-relative path (the CI check creates a dummy file there) so both
; locations compile.
!define EPHER_CONSOLE_SRC "../../../target/release/epher.exe"

!include "nsis-hooks.nsh"

Section
  ; The install hooks, in installer context (mirrors the template order:
  ; PREINSTALL, then the main-binary File, then POSTINSTALL).
  !insertmacro NSIS_HOOK_PREINSTALL
  !insertmacro NSIS_HOOK_POSTINSTALL

  ; Direct helper calls with representative stack usage.
  Push "C:\tools;C:\Program Files\epher;C:\bin"
  Push "C:\Program Files\epher"
  Call epher_path_contains
  Pop $0

  Push "C:\tools;C:\Program Files\epher;C:\bin"
  Push "C:\Program Files\epher"
  Call epher_remove_from_path
  Pop $1
SectionEnd

Section Uninstall
  ; The uninstall hooks, in uninstaller context (un. helpers only).
  !insertmacro NSIS_HOOK_PREUNINSTALL
  !insertmacro NSIS_HOOK_POSTUNINSTALL

  Push "C:\tools;C:\Program Files\epher;C:\bin"
  Push "C:\Program Files\epher"
  Call un.epher_path_contains
  Pop $0

  Push "C:\tools;C:\Program Files\epher;C:\bin"
  Push "C:\Program Files\epher"
  Call un.epher_remove_from_path
  Pop $1
SectionEnd
