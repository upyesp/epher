; nsis-hooks.nsh — PATH management for the unified epher installer (ADR-0011).
;
; What this does: adds the install directory (which contains epher.exe) to
; the *user* PATH (HKCU "Environment\Path" — no administrator needed, which
; matches Tauri's default currentUser install mode). After install, `epher`
; works from CMD, PowerShell, and Windows Terminal; already-open terminals
; must be restarted to inherit the change. On uninstall the entry is
; removed again.
;
; Implementation notes: written in plain NSIS + LogicLib only. Tauri's NSIS
; bundle ships no environment plugins (no EnVar), so PATH membership and
; entry removal are hand-rolled below and verified with makensis in
; nsis-check.nsi (compile check) — keep them dependency-free.
;
; How Tauri consumes this file: the template `!include`s it whole at the
; top of installer.nsi, then inserts each `NSIS_HOOK_*` macro body at its
; marked point. `NSIS_HOOK_POSTUNINSTALL` lands inside `Section Uninstall`,
; and NSIS forbids an uninstall Section from Calling installer-context
; functions — the uninstaller needs `un.`-prefixed copies. Hence each
; helper is written once as a macro, parameterized by prefix, and
; instantiated twice. Label names can stay identical across the copies
; because NSIS labels are scoped to their function.
;
; The functions deliberately clobber only $R0–$R6 and $0/$1 (standard NSIS
; section temporaries); the macros push/pop what they touch.

!include "WinMessages.nsh" ; HWND_BROADCAST, WM_SETTINGCHANGE

; Where the installer finds the console build to File next to the GUI main
; binary (ADR-0011, W2). Tauri writes installer.nsi to
; target/<triple>/release/nsis/<arch> and compiles it there; the windows
; overlay's beforeBundleCommand copies cargo's console build into the
; PARENT (target/release/nsis/epher.exe) right before bundling — the NSIS
; bundler wipes nsis/<arch> itself when it starts, the parent survives —
; and `..\epher.exe` reaches it under either NSIS path-resolution
; semantic. The nsis-check harness overrides this define with a
; repo-relative path (and the CI check drops a dummy file there).
!ifndef EPHER_CONSOLE_SRC
  !define EPHER_CONSOLE_SRC "..\epher.exe"
!endif

; ---------------------------------------------------------------------------
; Hooks (called by Tauri's installer.nsi at the marked points)
; ---------------------------------------------------------------------------

!macro NSIS_HOOK_PREINSTALL
  ; The console build rides along with the GUI main binary: PATH resolves
  ; `epher` to this file, so CLI/REPL/TUI keep full terminal semantics
  ; while double-clicks hit the GUI-subsystem main binary.
  File "/oname=$INSTDIR\epher.exe" "${EPHER_CONSOLE_SRC}"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Tauri's uninstaller deletes the main binary and RMDir's $INSTDIR —
  ; remove our extra file first so the directory can be removed.
  Delete "$INSTDIR\epher.exe"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ReadRegStr $0 HKCU "Environment" "Path"
  Push $0
  Push "$INSTDIR"
  Call epher_path_contains
  Pop $1
  ${If} $1 == 0
    ${If} $0 == ""
      WriteRegStr HKCU "Environment" "Path" "$INSTDIR"
    ${Else}
      WriteRegStr HKCU "Environment" "Path" "$0;$INSTDIR"
    ${EndIf}
    ; Tell Windows the user environment changed. New terminals (and
    ; Explorer-launched apps) pick it up immediately.
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ReadRegStr $0 HKCU "Environment" "Path"
  Push $0
  Push "$INSTDIR"
  Call un.epher_path_contains
  Pop $1
  ${If} $1 == 1
    Push $0
    Push "$INSTDIR"
    Call un.epher_remove_from_path
    Pop $0
    WriteRegStr HKCU "Environment" "Path" "$0"
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
  ${EndIf}
!macroend

; ---------------------------------------------------------------------------
; Helpers (installer context: plain; uninstaller context: `un.` prefix)
; ---------------------------------------------------------------------------

; epher_path_contains — stack in: [haystack, needle] → out: 1 if needle
; occurs anywhere in haystack, else 0 (substring match; the install dir is
; specific enough that substring granularity is safe here).
!macro epher_path_contains_body prefix
Function ${prefix}epher_path_contains
  Exch $R0            ; needle
  Exch
  Exch $R1            ; haystack
  Push $R2
  Push $R3
  Push $R4
  Push $R5
  Push $R6
  StrCpy $R2 0        ; offset
  StrLen $R3 $R0      ; needle length
  StrLen $R4 $R1      ; haystack length
  StrCpy $R9 0
  ${If} $R3 == 0
    StrCpy $R9 1      ; empty needle trivially found
    Goto epc_done
  ${EndIf}
  eph_loop:
    IntOp $R5 $R2 + $R3
    IntCmp $R5 $R4 0 0 epc_notfound      ; window end past the end → give up
    StrCpy $R6 $R1 $R3 $R2              ; window of needle length at offset
    ${If} $R6 == $R0
      StrCpy $R9 1
      Goto epc_done
    ${EndIf}
    IntOp $R2 $R2 + 1
    Goto eph_loop
  epc_notfound:
    StrCpy $R9 0
  epc_done:
  Pop $R6
  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Pop $R1
  Pop $R0
  Push $R9
FunctionEnd
!macroend
!insertmacro epher_path_contains_body ""
!insertmacro epher_path_contains_body "un."

; epher_remove_from_path — stack in: [path, entry] → out: path rebuilt
; without any ;-delimited entry equal to `entry` (case-insensitive, the
; StrCmp default). Empty entries (from `;;` or leading/trailing `;`) are
; dropped, normalizing the result.
!macro epher_remove_from_path_body prefix
Function ${prefix}epher_remove_from_path
  Exch $R0            ; entry to remove
  Exch
  Exch $R1            ; path
  Push $R2            ; remaining text (kept ;-terminated while looping)
  Push $R3            ; current entry
  Push $R4            ; rebuilt result
  Push $R5            ; scan index / offset after ';'
  Push $R6            ; current character
  StrCpy $R2 "$R1;"
  StrCpy $R4 ""
  erp_next:
    StrCmp $R2 "" erp_done 0
    StrCpy $R5 0
  erp_scan:
    StrCpy $R6 $R2 1 $R5
    ${If} $R6 == ";"
      Goto erp_found
    ${ElseIf} $R6 == ""
      ; Defensive: no ';' before the end (cannot happen while the
      ; ;-terminated invariant holds). Treat the rest as the entry.
      StrCpy $R3 $R2
      StrCpy $R2 ""
      Goto erp_keep_check
    ${EndIf}
    IntOp $R5 $R5 + 1
    Goto erp_scan
  erp_found:
    StrCpy $R3 $R2 $R5            ; entry = chars before the ';'
    IntOp $R5 $R5 + 1             ; step past the ';'
    StrCpy $R2 $R2 "" $R5         ; remaining = everything after it
  erp_keep_check:
    StrCmp $R3 $R0 erp_next 0     ; equal (case-insensitive) → drop it
    StrCmp $R3 "" erp_next 0      ; drop empty entries (normalizes ;;)
    ${If} $R4 == ""
      StrCpy $R4 $R3
    ${Else}
      StrCpy $R4 "$R4;$R3"
    ${EndIf}
    Goto erp_next
  erp_done:
  Pop $R6
  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Pop $R1
  Pop $R0
  Push $R4
FunctionEnd
!macroend
!insertmacro epher_remove_from_path_body ""
!insertmacro epher_remove_from_path_body "un."
