; Pusthakam additions to the Tauri NSIS installer (bundle.windows.nsis.installerHooks).
; The installer runs for all users (installMode "perMachine", so Windows asks for administrator
; rights). The startup entry is per-user (HKCU) so each person can turn it off in Settings.
; The daily backup folder is chosen in the app's library setup wizard, not here.
;
; HKCU\Software\Pusthakam
;   StartupConfigured set once the startup entry has been created, so updates do not
;                     re-enable starting with Windows after the user turned it off

!define PUSTHAKAM_KEY "Software\Pusthakam"
!define PUSTHAKAM_RUN_KEY "Software\Microsoft\Windows\CurrentVersion\Run"
!define PUSTHAKAM_APPROVED_KEY "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"

; Owner decision (28 November 2026): offer C:\Program Files (x86)\Pusthakam as the install folder.
; Tauri's template defaults a 64-bit build to C:\Program Files; this runs before the first page
; and changes only that untouched default, so an earlier install location or /D= still wins.
!define MUI_CUSTOMFUNCTION_GUIINIT PusthakamGuiInit
Function PusthakamGuiInit
  StrCmp $INSTDIR "$PROGRAMFILES64\Pusthakam" 0 +2
    StrCpy $INSTDIR "$PROGRAMFILES32\Pusthakam"
FunctionEnd

!macro NSIS_HOOK_POSTINSTALL
  Push $R0

  ; Start in the system tray when the user signs in to Windows. On by default; staff can
  ; turn it off in Settings. An update only refreshes the path of an existing entry.
  ReadRegStr $R0 HKCU "${PUSTHAKAM_KEY}" "StartupConfigured"
  StrCmp $R0 "1" 0 pusthakam_startup_write
  ReadRegStr $R0 HKCU "${PUSTHAKAM_RUN_KEY}" "Pusthakam"
  StrCmp $R0 "" pusthakam_startup_done
  pusthakam_startup_write:
    WriteRegStr HKCU "${PUSTHAKAM_RUN_KEY}" "Pusthakam" '"$INSTDIR\${MAINBINARYNAME}.exe" --background'
    WriteRegStr HKCU "${PUSTHAKAM_KEY}" "StartupConfigured" "1"
  pusthakam_startup_done:

  ; Earlier test builds asked for the backup folder here; that choice is no longer read.
  DeleteRegValue HKCU "${PUSTHAKAM_KEY}" "BackupFolder"

  Pop $R0
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Remove the startup entry, except when the uninstaller runs as part of an update
  ; ($UpdateMode comes from Tauri's installer template). Backups are kept.
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "${PUSTHAKAM_RUN_KEY}" "Pusthakam"
    DeleteRegValue HKCU "${PUSTHAKAM_APPROVED_KEY}" "Pusthakam"
    DeleteRegValue HKCU "${PUSTHAKAM_KEY}" "StartupConfigured"
  ${EndIf}
!macroend
