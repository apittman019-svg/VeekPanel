; Compile-only check of the actual hook in an uninstaller section.
; Never execute this synthetic installer; Windows behavior is tested by smoke.ps1.
Unicode true
!include LogicLib.nsh
!define MAINBINARYNAME "veekpanel"
!include "../../packaging/windows/installer-hooks.nsh"
Name "VeekPanel hook syntax check"
OutFile "${VEEK_HOOK_OUTPUT}"
RequestExecutionLevel user
Var UpdateMode
Function un.onInit
  StrCpy $UpdateMode 0
FunctionEnd
Section
  WriteUninstaller "$TEMP\veek-hook-syntax-uninstall.exe"
SectionEnd
Section "Uninstall"
  !insertmacro NSIS_HOOK_POSTUNINSTALL
SectionEnd
