#Requires -Version 5.1

<#
.SYNOPSIS
    Cleanup utility for Smainer desktop application data
.DESCRIPTION
    Removes wallet and configuration files from user profile when requested.
    This script is designed to be run during uninstallation or manually by users
    who want to completely remove their Smainer data.
.PARAMETER Force
    Skip confirmation prompts
.PARAMETER WhatIf
    Show what would be deleted without actually deleting
.EXAMPLE
    .\cleanup-app-data.ps1
    .\cleanup-app-data.ps1 -Force
    .\cleanup-app-data.ps1 -WhatIf
#>

[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter()]
    [switch]$Force
)

# Strict error handling
$ErrorActionPreference = "Stop"

# Define app data directory
$SMAINER_USER_DIR = Join-Path -Path $env:USERPROFILE -ChildPath ".smainer"

# Files to remove
$FILES_TO_REMOVE = @(
    "wallet.json",
    "ai_config.json"
)

function Write-ColoredMessage {
    param(
        [string]$Message,
        [string]$Level = "INFO"
    )
    
    $color = switch ($Level) {
        "ERROR" { "Red" }
        "WARNING" { "Yellow" }
        "SUCCESS" { "Green" }
        "INFO" { "Cyan" }
        default { "White" }
    }
    
    Write-Host $Message -ForegroundColor $color
}

function Test-SmaineRDataExists {
    if (-not (Test-Path -Path $SMAINER_USER_DIR)) {
        return $false
    }
    
    foreach ($file in $FILES_TO_REMOVE) {
        $fullPath = Join-Path -Path $SMAINER_USER_DIR -ChildPath $file
        if (Test-Path -Path $fullPath) {
            return $true
        }
    }
    
    return $false
}

function Show-DataPreview {
    Write-ColoredMessage "`n=== Smainer Application Data Cleanup ===" -Level "INFO"
    Write-ColoredMessage "`nThe following directory will be checked:" -Level "INFO"
    Write-Host "  $SMAINER_USER_DIR`n"
    
    if (-not (Test-Path -Path $SMAINER_USER_DIR)) {
        Write-ColoredMessage "Directory does not exist - nothing to clean up" -Level "WARNING"
        return $false
    }
    
    Write-ColoredMessage "Files that will be deleted:" -Level "WARNING"
    
    $foundFiles = @()
    foreach ($file in $FILES_TO_REMOVE) {
        $fullPath = Join-Path -Path $SMAINER_USER_DIR -ChildPath $file
        if (Test-Path -Path $fullPath) {
            $fileInfo = Get-Item -Path $fullPath
            $foundFiles += $fullPath
            Write-Host "  ✓ $file" -NoNewline
            Write-Host " ($([math]::Round($fileInfo.Length / 1KB, 2)) KB)" -ForegroundColor Gray
        } else {
            Write-Host "  - $file (not found)" -ForegroundColor DarkGray
        }
    }
    
    # Check for other files in directory
    $otherFiles = Get-ChildItem -Path $SMAINER_USER_DIR -File | Where-Object {
        $_.Name -notin $FILES_TO_REMOVE
    }
    
    if ($otherFiles) {
        Write-ColoredMessage "`nOther files in directory (will also be removed):" -Level "WARNING"
        foreach ($file in $otherFiles) {
            Write-Host "  • $($file.Name)" -ForegroundColor Yellow
        }
    }
    
    if ($foundFiles.Count -eq 0 -and -not $otherFiles) {
        Write-ColoredMessage "`nNo Smainer data files found to delete" -Level "WARNING"
        return $false
    }
    
    Write-ColoredMessage "`nWARNING: This will permanently delete your wallet private keys!" -Level "ERROR"
    Write-ColoredMessage "This action CANNOT be undone!`n" -Level "ERROR"
    
    return $true
}

function Confirm-Deletion {
    if ($Force) {
        return $true
    }
    
    if ($WhatIfPreference) {
        return $false
    }
    
    Write-Host "Do you want to proceed with deletion? " -NoNewline -ForegroundColor Yellow
    Write-Host "(Type 'DELETE' to confirm): " -NoNewline
    $confirmation = Read-Host
    
    return ($confirmation -eq "DELETE")
}

function Remove-SmaineRAppData {
    if (-not (Test-Path -Path $SMAINER_USER_DIR)) {
        Write-ColoredMessage "Directory does not exist: $SMAINER_USER_DIR" -Level "INFO"
        return
    }
    
    try {
        $deletedFiles = @()
        $failedFiles = @()
        
        # Delete specific known files
        foreach ($file in $FILES_TO_REMOVE) {
            $fullPath = Join-Path -Path $SMAINER_USER_DIR -ChildPath $file
            
            if (Test-Path -Path $fullPath) {
                if ($PSCmdlet.ShouldProcess($fullPath, "Delete file")) {
                    try {
                        Remove-Item -Path $fullPath -Force -ErrorAction Stop
                        $deletedFiles += $file
                        Write-ColoredMessage "  ✓ Deleted: $file" -Level "SUCCESS"
                    }
                    catch {
                        $failedFiles += @{File = $file; Error = $_.Exception.Message}
                        Write-ColoredMessage "  ✗ Failed to delete: $file - $($_.Exception.Message)" -Level "ERROR"
                    }
                }
            }
        }
        
        # Remove the entire directory if possible (including any other files)
        if ($PSCmdlet.ShouldProcess($SMAINER_USER_DIR, "Remove directory")) {
            if (Test-Path -Path $SMAINER_USER_DIR) {
                try {
                    # Get count of remaining files
                    $remainingFiles = Get-ChildItem -Path $SMAINER_USER_DIR -Recurse -File
                    
                    if ($remainingFiles.Count -gt 0) {
                        Write-ColoredMessage "  Removing $($remainingFiles.Count) remaining file(s)..." -Level "INFO"
                    }
                    
                    Remove-Item -Path $SMAINER_USER_DIR -Recurse -Force -ErrorAction Stop
                    Write-ColoredMessage "  ✓ Removed directory: $SMAINER_USER_DIR" -Level "SUCCESS"
                }
                catch {
                    Write-ColoredMessage "  ✗ Could not remove directory: $($_.Exception.Message)" -Level "WARNING"
                    Write-ColoredMessage "    Some files may still remain" -Level "WARNING"
                }
            }
        }
        
        # Summary
        Write-ColoredMessage "`n=== Cleanup Summary ===" -Level "INFO"
        Write-ColoredMessage "Files deleted: $($deletedFiles.Count)" -Level "SUCCESS"
        
        if ($failedFiles.Count -gt 0) {
            Write-ColoredMessage "Files failed: $($failedFiles.Count)" -Level "ERROR"
            foreach ($failed in $failedFiles) {
                Write-Host "  • $($failed.File): $($failed.Error)" -ForegroundColor Red
            }
        }
        
        if (Test-Path -Path $SMAINER_USER_DIR) {
            Write-ColoredMessage "`nNote: Directory still exists (may contain system files)" -Level "WARNING"
        } else {
            Write-ColoredMessage "`nAll Smainer application data has been removed" -Level "SUCCESS"
        }
    }
    catch {
        Write-ColoredMessage "Cleanup failed: $($_.Exception.Message)" -Level "ERROR"
        exit 1
    }
}

# Main execution
try {
    Write-Host "`n"
    
    # Show what will be deleted
    $hasData = Show-DataPreview
    
    if (-not $hasData) {
        Write-ColoredMessage "`nNothing to clean up. Exiting.`n" -Level "INFO"
        exit 0
    }
    
    # Confirm deletion
    if (-not (Confirm-Deletion)) {
        Write-ColoredMessage "`nCleanup cancelled by user`n" -Level "INFO"
        exit 0
    }
    
    # Perform deletion
    Write-ColoredMessage "`nProceeding with cleanup..." -Level "INFO"
    Remove-SmaineRAppData
    
    Write-Host "`n"
}
catch {
    Write-ColoredMessage "`nUnexpected error: $($_.Exception.Message)" -Level "ERROR"
    Write-ColoredMessage "Stack trace: $($_.ScriptStackTrace)" -Level "ERROR"
    exit 1
}
