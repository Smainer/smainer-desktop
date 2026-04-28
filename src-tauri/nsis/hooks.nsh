; NSIS installer hooks for Smainer desktop application
; This file provides custom uninstall behavior for app data removal

!macro NSIS_HOOK_PREUNINSTALL
  ; Only remove app data if:
  ; - User checked "Delete app data" checkbox (DeleteAppDataCheckboxState == 1)
  ; - This is not an update operation (UpdateMode != 1)
  ${If} $DeleteAppDataCheckboxState == 1
  ${AndIf} $UpdateMode <> 1
    DetailPrint "Removing Smainer app data from user profile..."
    ; Remove the .smainer directory and all contents recursively
    RMDir /r "$PROFILE\.smainer"
    ; Log the removal for diagnostics
    ${If} ${Errors}
      DetailPrint "Warning: Could not completely remove $PROFILE\.smainer"
    ${Else}
      DetailPrint "Successfully removed $PROFILE\.smainer"
    ${EndIf}
  ${Else}
    DetailPrint "Keeping app data (checkbox not selected or update mode)"
  ${EndIf}
!macroend