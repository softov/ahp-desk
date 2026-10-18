; NSIS hooks for the desk's installer (tauri.conf.json, bundle.windows.nsis.installerHooks).

; Removing the desk removes its login entry: the value tauri-plugin-autostart
; writes, named after the product, and the one Task Manager keeps beside it.
; An update runs the old uninstaller with /UPDATE, so the entry is kept then,
; or every update would turn the start with Windows off.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "ahp-desk"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "ahp-desk"
  ${EndIf}
!macroend
