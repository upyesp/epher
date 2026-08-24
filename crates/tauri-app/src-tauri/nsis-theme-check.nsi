; nsis-theme-check.nsi — compile-only harness for the theme additions
; in nsis/installer.nsi (ADR-0025, revised by ADR-0026, lightened by
; ADR-0028).
;
; The vendored installer template itself only compiles inside the Windows
; bundling job (it is a handlebars template rendered with build data, and
; tauri's bundled makensis 3.11 toolset); this harness compiles the exact
; theme constructs that remain — the MUI color defines, the light header
; and sidebar bitmaps (the header bitmap is the top-left logo on every
; page), the default-checked "delete app data" checkbox, and the
; ~/.epher removal — against the system makensis and MUI2 so a syntax
; slip in the additions fails CI before any bundle ships.
;
; ADR-0026: the per-control SetCtlColors repaint that lived here is gone.
; It corrupted the window procs of MUI-managed controls on the directory
; page (vanished controls, click lock-up), so the theme now uses the
; official MUI2 mechanism only. Nothing in this harness may call
; SetCtlColors or walk window children.
;
; Run from the repo: makensis crates/tauri-app/src-tauri/nsis-theme-check.nsi
Unicode true
OutFile "nsis-theme-check.exe"
Name "epher nsis theme check"
!include MUI2.nsh
!include WinMessages.nsh

!define MUI_BGCOLOR F0F0F0
!define MUI_TEXTCOLOR 000000
!define MUI_INSTFILESPAGE_COLORS "FFFFFF 000000"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "nsis\header.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP "nsis\sidebar.bmp"
!define MUI_HEADERIMAGE_UNBITMAP "nsis\header.bmp"

; ADR-0028: the finish-page checkboxes draw with the classic button
; colors after MUI2 strips their visual theme, and SetCtlColors cannot
; recolor checkbox text (NSIS bug #443) — so COLOR_BTNTEXT is pinned to
; black, which the light theme needs (see installer.nsi .onInit). The
; finish page below compiles the same constructs the installer uses.
Var DeleteAppDataCheckbox
Var DeleteAppDataCheckboxState

; page show hook + the default-checked checkbox
Function un.ConfirmShow
  FindWindow $1 "#32770" "" $HWNDPARENT
  System::Call 'user32::CreateWindowEx(i 0, w "Button", w "Delete app data", i 0x50010003, i 0, i 0, i 100, i 25, p r1, i0, i0, i0) i .s'
  Pop $DeleteAppDataCheckbox
  SendMessage $DeleteAppDataCheckbox ${BM_SETCHECK} ${BST_CHECKED} 0
FunctionEnd

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_WELCOME
!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.ConfirmShow
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
; The installer's finish page: the Run and Create-desktop-shortcut
; checkboxes live here (MUI strips their theme; COLOR_BTNTEXT recolors
; their labels — see ADR-0027).
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "Run epher"
!define MUI_FINISHPAGE_SHOWREADME
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Create desktop shortcut"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section
  DetailPrint "install"
SectionEnd

Section Uninstall
  ${If} $DeleteAppDataCheckboxState = 1
    RmDir /r "$PROFILE\.epher"
  ${EndIf}
SectionEnd

Function .onInit
  System::Call 'user32::SetSysColors(i 1, *i 18, *i 0x000000)'
  StrCpy $DeleteAppDataCheckboxState 0
  WriteUninstaller "$TEMP\uninst-check.exe"
FunctionEnd
