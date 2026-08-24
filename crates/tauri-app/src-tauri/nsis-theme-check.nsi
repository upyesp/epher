; nsis-theme-check.nsi — compile-only harness for the dark-theme additions
; in nsis/installer.nsi (ADR-0025).
;
; The vendored installer template itself only compiles inside the Windows
; bundling job (it is a handlebars template rendered with build data, and
; tauri's bundled makensis 3.11 toolset); this harness compiles the exact
; theme constructs — the MUI color defines, the epherPaint window walk,
; the default-checked "delete app data" checkbox, and the ~/.epher
; removal — against the system makensis and MUI2 so a syntax slip in the
; additions fails CI before any bundle ships.
;
; Run from the repo: makensis crates/tauri-app/src-tauri/nsis-theme-check.nsi
Unicode true
OutFile "nsis-theme-check.exe"
Name "epher nsis theme check"
!include MUI2.nsh
!include WinMessages.nsh

!define MUI_BGCOLOR 141416
!define MUI_TEXTCOLOR F5F6F7
!define MUI_INSTFILESPAGE_COLORS "141416 F5F6F7"

!macro epher_paint_body prefix
Function ${prefix}epherPaint
  SetCtlColors $HWNDPARENT 0xF5F6F7 0x141416
  System::Call 'user32::GetWindow(p $HWNDPARENT, i 5) p .r2'
  epher_outer_loop:
    IntCmp $r2 0 epher_outer_done 0 0
    SetCtlColors $r2 0xF5F6F7 0x141416
    System::Call 'user32::GetWindow(p r2, i 2) p .r2'
    Goto epher_outer_loop
  epher_outer_done:
  FindWindow $r2 "#32770" "" $HWNDPARENT
  IntCmp $r2 0 epher_done 0 0
  SetCtlColors $r2 0xF5F6F7 0x141416
  System::Call 'user32::GetWindow(p r2, i 5) p .r3'
  epher_inner_loop:
    IntCmp $r3 0 epher_done 0 0
    SetCtlColors $r3 0xF5F6F7 0x141416
    System::Call 'user32::GetWindow(p r3, i 2) p .r3'
    Goto epher_inner_loop
  epher_done:
FunctionEnd
!macroend
!insertmacro epher_paint_body ""
!insertmacro epher_paint_body "un."

Var DeleteAppDataCheckbox
Var DeleteAppDataCheckboxState

; page show hook + the default-checked checkbox + paint-after-create
Function un.ConfirmShow
  FindWindow $1 "#32770" "" $HWNDPARENT
  System::Call 'user32::CreateWindowEx(i 0, w "Button", w "Delete app data", i 0x50010003, i 0, i 0, i 100, i 25, p r1, i0, i0, i0) i .s'
  Pop $DeleteAppDataCheckbox
  SendMessage $DeleteAppDataCheckbox ${BM_SETCHECK} ${BST_CHECKED} 0
  Call un.epherPaint
FunctionEnd

!define MUI_PAGE_CUSTOMFUNCTION_SHOW epherPaint
!insertmacro MUI_PAGE_DIRECTORY
!define MUI_PAGE_CUSTOMFUNCTION_SHOW epherPaint
!insertmacro MUI_PAGE_INSTFILES
!define MUI_PAGE_CUSTOMFUNCTION_SHOW epherPaint
!insertmacro MUI_PAGE_WELCOME
!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.ConfirmShow
!insertmacro MUI_UNPAGE_CONFIRM
!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.epherPaint
!insertmacro MUI_UNPAGE_INSTFILES

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
  StrCpy $DeleteAppDataCheckboxState 0
  WriteUninstaller "$TEMP\uninst-check.exe"
FunctionEnd
