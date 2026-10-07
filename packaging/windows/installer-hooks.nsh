; Original VeekPanel extension. Match app/src/startup.rs registration.
; Tauri's template removes PRODUCTNAME; our runtime uses the bundle identifier.
; Never enable startup during install or delete an entry owned by another path.
!ifndef VEEKPANEL_INSTALLER_HOOKS
!define VEEKPANEL_INSTALLER_HOOKS

!macro NSIS_HOOK_POSTUNINSTALL
  ; /UPDATE preserves per-user opt-in. Ordinary removal clears only the exact
  ; command for this installation, after uninstall has proceeded.
  ${If} $UpdateMode <> 1
    Push $R0
    Push $R1
    ClearErrors
    ReadRegStr $R0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "org.veekpanel.desktop"
    StrCpy $R1 '$\"$INSTDIR\${MAINBINARYNAME}.exe$\" --autostart'
    StrCmpS $R0 $R1 0 veek_startup_cleanup_done
    ClearErrors
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "org.veekpanel.desktop"
    IfErrors 0 veek_startup_cleanup_done
    DetailPrint "Could not remove VeekPanel login entry. Review Windows Startup settings."
    SetErrorLevel 1
veek_startup_cleanup_done:
    ClearErrors
    Pop $R1
    Pop $R0
  ${EndIf}
!macroend

!endif
