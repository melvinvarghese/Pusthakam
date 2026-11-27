; Pusthakam additions to the Tauri NSIS installer (bundle.windows.nsis.installerHooks).
; Everything is per-user (HKCU), matching installMode "currentUser"; no administrator rights needed.
;
; HKCU\Software\Pusthakam
;   BackupFolder      folder chosen for daily backups; the app copies it on first start
;   StartupConfigured set once the startup entry has been created, so updates do not
;                     re-enable starting with Windows after the user turned it off

!define PUSTHAKAM_KEY "Software\Pusthakam"
!define PUSTHAKAM_RUN_KEY "Software\Microsoft\Windows\CurrentVersion\Run"
!define PUSTHAKAM_APPROVED_KEY "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"

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

  ; Ask once where daily backups go. Updates and silent installs keep the earlier choice.
  ReadRegStr $R0 HKCU "${PUSTHAKAM_KEY}" "BackupFolder"
  StrCmp $R0 "" 0 pusthakam_backup_done
  IfSilent pusthakam_backup_done
  MessageBox MB_OK|MB_ICONINFORMATION "Pusthakam backs up your library the first time it opens each day and keeps the last 7 days of backups.$\r$\n$\r$\nOn the next screen, choose the folder for these backups. A different drive or a USB drive is safest: if this computer's disk fails, your backups are still there.$\r$\n$\r$\nIf you cancel, backups are kept in Pusthakam's own data folder. You can change the folder later under Database Backups."
  nsDialogs::SelectFolderDialog "Choose the folder for Pusthakam's daily backups" "$DOCUMENTS"
  Pop $R0
  StrCmp $R0 "error" pusthakam_backup_done
  StrCmp $R0 "" pusthakam_backup_done
  CreateDirectory "$R0"
  WriteRegStr HKCU "${PUSTHAKAM_KEY}" "BackupFolder" "$R0"
  pusthakam_backup_done:

  Pop $R0
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Remove the startup entry, except when the uninstaller runs as part of an update
  ; ($UpdateMode comes from Tauri's installer template). The backup folder choice and the
  ; backups themselves are kept.
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "${PUSTHAKAM_RUN_KEY}" "Pusthakam"
    DeleteRegValue HKCU "${PUSTHAKAM_APPROVED_KEY}" "Pusthakam"
    DeleteRegValue HKCU "${PUSTHAKAM_KEY}" "StartupConfigured"
  ${EndIf}
!macroend
