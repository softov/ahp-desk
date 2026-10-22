; NSIS hooks for the desk's installer (tauri.conf.json, bundle.windows.nsis.installerHooks).

; Whether a process runs from the desk's bundled Node, set by AHP_DESK_FIND_NODE: "1" when one does.
Var AhpDeskNodeRuns

; A local ahpd the desk installed runs on $INSTDIR\node\node.exe, and keeps
; running after the desk quits unless its host says to stop it. Windows keeps
; that file open, so an install cannot replace it and an uninstall cannot
; remove it. The folder goes to PowerShell through the environment, so a
; quote in a user's name cannot break the command.
!macro AHP_DESK_NODE_DIR
  System::Call 'Kernel32::SetEnvironmentVariable(t "AHP_DESK_NODE_DIR", t "$INSTDIR\node\")'
!macroend

!macro AHP_DESK_FIND_NODE
  !insertmacro AHP_DESK_NODE_DIR
  nsExec::Exec `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "if (@(Get-CimInstance Win32_Process | Where-Object { $$_.ExecutablePath -and $$_.ExecutablePath.StartsWith($$env:AHP_DESK_NODE_DIR, 'OrdinalIgnoreCase') }).Count) { exit 1 }"`
  Pop $AhpDeskNodeRuns
!macroend

; Ends every process that runs from the bundled Node, and waits up to 5 s for them to go.
!macro AHP_DESK_STOP_NODE
  !insertmacro AHP_DESK_NODE_DIR
  nsExec::Exec `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$$ids = @(Get-CimInstance Win32_Process | Where-Object { $$_.ExecutablePath -and $$_.ExecutablePath.StartsWith($$env:AHP_DESK_NODE_DIR, 'OrdinalIgnoreCase') } | ForEach-Object { $$_.ProcessId }); foreach ($$id in $$ids) { Stop-Process -Id $$id -Force -ErrorAction SilentlyContinue }; foreach ($$i in 1..20) { if (-not @(Get-Process -Id $$ids -ErrorAction SilentlyContinue).Count) { break }; Start-Sleep -Milliseconds 250 }"`
  Pop $AhpDeskNodeRuns
!macroend

; Before the files are written. The desk is closed first, as the template does
; just after this hook, so nothing below runs under a desk that is still open.
; The local ahpd stops whatever desk ran this installer, so an update from a
; desk without that stop (0.1.0) also goes through; the new desk starts it
; again by its host's start setting. The UI folder is emptied, since an update
; writes over files of the same name and would keep the old release's others.
!macro NSIS_HOOK_PREINSTALL
  !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  !insertmacro AHP_DESK_STOP_NODE
  RMDir /r "$INSTDIR\ui"
!macroend

; Before the files are removed: a local ahpd still running is stopped, after
; asking, so its Node can go with the rest. An update's run of the old
; uninstaller (/UPDATE) and a silent one stop it without asking.
!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro AHP_DESK_FIND_NODE
  ${If} $AhpDeskNodeRuns == "1"
    ${If} $UpdateMode <> 1
    ${AndIfNot} ${Silent}
      MessageBox MB_OKCANCEL|MB_ICONQUESTION "The local ahpd that ahp-desk installed is still running. It stops now, so it can be removed with ahp-desk.$\r$\n$\r$\nIts sessions and settings stay in your user folder." IDOK +2
      Abort "ahp-desk was not removed: the local ahpd is still running."
    ${EndIf}
    !insertmacro AHP_DESK_STOP_NODE
  ${EndIf}
!macroend

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
