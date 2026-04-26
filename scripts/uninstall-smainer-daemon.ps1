#Requires -Version 5.1
#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Uninstaller for Smainer provider daemon and components
.DESCRIPTION
    Completely removes Smainer provider daemon installation including:
    - Windows service and service account
    - Encrypted credentials from Windows Credential Manager
    - Configuration files and directories
    - Optional component cleanup (Ollama, TransformerLab)
.PARAMETER Force
    Skip confirmation prompts
.PARAMETER KeepLogs
    Preserve log files during uninstallation
.PARAMETER KeepOllama
    Skip Ollama removal (if installed by Smainer)
.EXAMPLE
    .\uninstall-smainer-daemon.ps1
    .\uninstall-smainer-daemon.ps1 -Force
    .\uninstall-smainer-daemon.ps1 -KeepLogs -KeepOllama
#>

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$KeepLogs,
    [switch]$KeepOllama
)

# Strict error handling
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Constants (must match installer)
$SMAINER_SERVICE_NAME = "SmaiserProviderDaemon"  
$SMAINER_USER = "SmaineRProvider"
$SMAINER_HOME = "$env:ProgramFiles\Smainer"
$SMAINER_DATA = "$env:ProgramData\Smainer"
$SMAINER_LOG = "$env:ProgramData\Smainer\Logs"

# Uninstall state tracking
$script:UninstallResults = @{
    ServiceRemoved = $false
    UserRemoved = $false
    CredentialsCleared = $false
    DirectoriesRemoved = @()
    ComponentsRemoved = @()
    Errors = @()
}

function Write-StatusMessage {
    param(
        [Parameter(Mandatory)]
        [string]$Message,
        [string]$Level = "INFO"
    )
    
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $color = switch ($Level) {
        "ERROR" { "Red" }
        "WARNING" { "Yellow" }
        "SUCCESS" { "Green" }
        default { "White" }
    }
    
    Write-Host "[$timestamp] [$Level] $Message" -ForegroundColor $color
}

function Confirm-Uninstallation {
    if ($Force) {
        return $true
    }
    
    Write-Host ""
    Write-StatusMessage "=== Smainer Daemon Uninstaller ===" 
    Write-Host ""
    Write-Host "This will PERMANENTLY remove:" -ForegroundColor Yellow
    Write-Host "  • Smainer provider service and service account" -ForegroundColor White
    Write-Host "  • All encrypted credentials and private keys" -ForegroundColor White  
    Write-Host "  • Configuration files and application data" -ForegroundColor White
    
    if (-not $KeepLogs) {
        Write-Host "  • All log files and history" -ForegroundColor White
    }
    
    if (-not $KeepOllama) {
        Write-Host "  • Ollama installation (if installed by Smainer)" -ForegroundColor White
    }
    
    Write-Host ""
    Write-Host "WARNING: This action CANNOT be undone!" -ForegroundColor Red
    Write-Host ""
    
    $confirmation = Read-Host "Type 'UNINSTALL' to proceed, or anything else to cancel"
    return ($confirmation -eq "UNINSTALL")
}

function Stop-SmaineRService {
    Write-StatusMessage "Stopping Smainer service..."
    
    try {
        $service = Get-Service -Name $SMAINER_SERVICE_NAME -ErrorAction SilentlyContinue
        
        if ($service) {
            if ($service.Status -eq "Running") {
                Write-StatusMessage "Stopping service $SMAINER_SERVICE_NAME..."
                Stop-Service -Name $SMAINER_SERVICE_NAME -Force -NoWait
                
                # Wait for service to stop with timeout
                $timeout = 30
                while ($service.Status -eq "Running" -and $timeout -gt 0) {
                    Start-Sleep -Seconds 1
                    $service.Refresh()
                    $timeout--
                }
                
                if ($service.Status -eq "Running") {
                    Write-StatusMessage "Service did not stop gracefully, forcing termination..." -Level "WARNING"
                    # Force kill the process if needed
                    $processes = Get-WmiObject -Class Win32_Service | Where-Object {$_.Name -eq $SMAINER_SERVICE_NAME}
                    if ($processes.ProcessId) {
                        Stop-Process -Id $processes.ProcessId -Force -ErrorAction SilentlyContinue
                    }
                }
            }
            
            Write-StatusMessage "Removing service $SMAINER_SERVICE_NAME..."
            sc.exe delete $SMAINER_SERVICE_NAME
            
            if ($LASTEXITCODE -eq 0) {
                $script:UninstallResults.ServiceRemoved = $true
                Write-StatusMessage "Service removed successfully" -Level "SUCCESS"
            } else {
                throw "Service removal failed with exit code: $LASTEXITCODE"
            }
        } else {
            Write-StatusMessage "Service not found - may already be uninstalled" -Level "WARNING"
        }
    }
    catch {
        $script:UninstallResults.Errors += "Service removal: $($_.Exception.Message)"
        Write-StatusMessage "Failed to remove service: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Remove-SmaineRCredentials {
    Write-StatusMessage "Removing stored credentials..."
    
    try {
        # Remove all Smainer-related credentials from Windows Credential Manager
        $credList = cmdkey /list | Select-String "Target: Smainer_"
        
        foreach ($credLine in $credList) {
            $targetName = ($credLine -split "Target: ")[1].Trim()
            Write-StatusMessage "Removing credential: $targetName"
            cmdkey /delete:$targetName
        }
        
        # Also try pattern-based removal
        cmdkey /delete:"Smainer_*" 2>$null
        
        $script:UninstallResults.CredentialsCleared = $true
        Write-StatusMessage "Credentials removed from Windows Credential Manager" -Level "SUCCESS"
    }
    catch {
        $script:UninstallResults.Errors += "Credential removal: $($_.Exception.Message)"
        Write-StatusMessage "Failed to remove credentials: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Remove-SmaineRUser {
    Write-StatusMessage "Removing service account..."
    
    try {
        $user = Get-LocalUser -Name $SMAINER_USER -ErrorAction SilentlyContinue
        
        if ($user) {
            Remove-LocalUser -Name $SMAINER_USER -Confirm:$false
            $script:UninstallResults.UserRemoved = $true
            Write-StatusMessage "Service account '$SMAINER_USER' removed successfully" -Level "SUCCESS"
        } else {
            Write-StatusMessage "Service account not found - may already be removed" -Level "WARNING"
        }
    }
    catch {
        $script:UninstallResults.Errors += "User removal: $($_.Exception.Message)"
        Write-StatusMessage "Failed to remove service account: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Remove-SmaineRDirectories {
    $directoriesToRemove = @()
    
    # Always remove program files
    if (Test-Path -Path $SMAINER_HOME) {
        $directoriesToRemove += $SMAINER_HOME
    }
    
    # Remove data directory (but preserve logs if requested)
    if (Test-Path -Path $SMAINER_DATA) {
        if ($KeepLogs -and (Test-Path -Path $SMAINER_LOG)) {
            # Remove config files but keep logs
            $configPath = Join-Path -Path $SMAINER_DATA -ChildPath "config.json"
            if (Test-Path -Path $configPath) {
                Remove-Item -Path $configPath -Force -ErrorAction SilentlyContinue
            }
            Write-StatusMessage "Preserving log directory: $SMAINER_LOG" -Level "INFO"
        } else {
            $directoriesToRemove += $SMAINER_DATA
        }
    }
    
    foreach ($directory in $directoriesToRemove) {
        try {
            Write-StatusMessage "Removing directory: $directory"
            Remove-Item -Path $directory -Recurse -Force -ErrorAction Stop
            $script:UninstallResults.DirectoriesRemoved += $directory
        }
        catch {
            $script:UninstallResults.Errors += "Directory removal ($directory): $($_.Exception.Message)"
            Write-StatusMessage "Failed to remove directory $directory`: $($_.Exception.Message)" -Level "ERROR"
        }
    }
    
    if ($script:UninstallResults.DirectoriesRemoved.Count -gt 0) {
        Write-StatusMessage "Directories removed successfully" -Level "SUCCESS"
    }
}

function Remove-OllamaComponent {
    if ($KeepOllama) {
        Write-StatusMessage "Skipping Ollama removal as requested" -Level "INFO"
        return
    }
    
    Write-StatusMessage "Checking for Ollama installation..."
    
    try {
        # Check for Ollama in common installation paths
        $ollamaPaths = @(
            "${env:LOCALAPPDATA}\Programs\Ollama",
            "${env:ProgramFiles}\Ollama"
        )
        
        $ollamaFound = $false
        foreach ($path in $ollamaPaths) {
            if (Test-Path -Path $path) {
                $ollamaFound = $true
                
                # Check if it was installed by Smainer (simple heuristic)
                $smaineRMarker = Join-Path -Path $path -ChildPath ".smainer-installed"
                if (Test-Path -Path $smaineRMarker) {
                    Write-StatusMessage "Removing Ollama installation (installed by Smainer)..."
                    
                    # Stop Ollama processes
                    Get-Process -Name "ollama*" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
                    
                    # Uninstall via Programs and Features if possible
                    $uninstaller = Get-ChildItem -Path $path -Name "*uninstall*" -ErrorAction SilentlyContinue | Select-Object -First 1
                    if ($uninstaller) {
                        $uninstallPath = Join-Path -Path $path -ChildPath $uninstaller
                        Start-Process -FilePath $uninstallPath -ArgumentList "/S" -Wait -NoNewWindow -ErrorAction SilentlyContinue
                    }
                    
                    # Remove directory manually if uninstaller didn't work
                    if (Test-Path -Path $path) {
                        Remove-Item -Path $path -Recurse -Force -ErrorAction SilentlyContinue
                    }
                    
                    $script:UninstallResults.ComponentsRemoved += "Ollama"
                    Write-StatusMessage "Ollama removed successfully" -Level "SUCCESS"
                } else {
                    Write-StatusMessage "Ollama found but not installed by Smainer - skipping removal" -Level "WARNING"
                }
                break
            }
        }
        
        if (-not $ollamaFound) {
            Write-StatusMessage "Ollama not found - may already be removed" -Level "INFO"
        }
    }
    catch {
        $script:UninstallResults.Errors += "Ollama removal: $($_.Exception.Message)"
        Write-StatusMessage "Failed to remove Ollama: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Clear-WindowsRegistryEntries {
    Write-StatusMessage "Cleaning Windows registry entries..."
    
    try {
        # Remove any registry entries that might have been created
        $registryPaths = @(
            "HKLM:\SOFTWARE\Smainer",
            "HKLM:\SYSTEM\CurrentControlSet\Services\$SMAINER_SERVICE_NAME"
        )
        
        foreach ($regPath in $registryPaths) {
            if (Test-Path -Path $regPath) {
                Write-StatusMessage "Removing registry key: $regPath"
                Remove-Item -Path $regPath -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
        
        Write-StatusMessage "Registry cleanup completed" -Level "SUCCESS"
    }
    catch {
        $script:UninstallResults.Errors += "Registry cleanup: $($_.Exception.Message)"
        Write-StatusMessage "Registry cleanup failed: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Show-UninstallSummary {
    Write-Host ""
    Write-StatusMessage "=== UNINSTALLATION COMPLETED ===" -Level "SUCCESS"
    Write-Host ""
    
    Write-Host "Removal Summary:" -ForegroundColor Cyan
    
    if ($script:UninstallResults.ServiceRemoved) {
        Write-Host "  ✓ Windows service removed" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Windows service (not found or failed)" -ForegroundColor Yellow
    }
    
    if ($script:UninstallResults.UserRemoved) {
        Write-Host "  ✓ Service account removed" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Service account (not found or failed)" -ForegroundColor Yellow
    }
    
    if ($script:UninstallResults.CredentialsCleared) {
        Write-Host "  ✓ Stored credentials cleared" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Credentials clearing (not found or failed)" -ForegroundColor Yellow
    }
    
    foreach ($dir in $script:UninstallResults.DirectoriesRemoved) {
        Write-Host "  ✓ Removed: $dir" -ForegroundColor Green
    }
    
    foreach ($component in $script:UninstallResults.ComponentsRemoved) {
        Write-Host "  ✓ Component removed: $component" -ForegroundColor Green
    }
    
    if ($KeepLogs) {
        Write-Host "  ℹ Logs preserved in: $SMAINER_LOG" -ForegroundColor Cyan
    }
    
    if ($KeepOllama) {
        Write-Host "  ℹ Ollama preserved (skipped removal)" -ForegroundColor Cyan
    }
    
    Write-Host ""
    
    if ($script:UninstallResults.Errors.Count -gt 0) {
        Write-Host "Errors encountered:" -ForegroundColor Yellow
        foreach ($error in $script:UninstallResults.Errors) {
            Write-Host "  • $error" -ForegroundColor Red
        }
        Write-Host ""
        Write-StatusMessage "Uninstallation completed with errors" -Level "WARNING"
    } else {
        Write-StatusMessage "All components removed successfully" -Level "SUCCESS"
    }
    
    Write-Host "Smainer daemon has been removed from this system." -ForegroundColor Green
    Write-Host "To reinstall, run: .\install-smainer-daemon.ps1" -ForegroundColor Cyan
}

function Test-Prerequisites {
    Write-StatusMessage "Checking prerequisites..."
    
    # Check PowerShell version
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        throw "PowerShell 5.1 or higher required. Current version: $($PSVersionTable.PSVersion)"
    }
    
    # Check admin privileges
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "Administrator privileges required. Please run PowerShell as Administrator."
    }
    
    Write-StatusMessage "Prerequisites check passed" -Level "SUCCESS"
}

# Main uninstallation flow
function Start-Uninstallation {
    try {
        # Prerequisites check
        Test-Prerequisites
        
        # Confirmation
        if (-not (Confirm-Uninstallation)) {
            Write-StatusMessage "Uninstallation cancelled by user" -Level "INFO"
            exit 0
        }
        
        Write-StatusMessage "Beginning uninstallation process..."
        Write-Host ""
        
        # Stop and remove service
        Stop-SmaineRService
        
        # Remove credentials
        Remove-SmaineRCredentials
        
        # Remove service account
        Remove-SmaineRUser
        
        # Remove directories  
        Remove-SmaineRDirectories
        
        # Remove optional components
        Remove-OllamaComponent
        
        # Clean registry
        Clear-WindowsRegistryEntries
        
        # Show summary
        Show-UninstallSummary
        
        exit 0
        
    }
    catch {
        Write-StatusMessage "Uninstallation failed: $($_.Exception.Message)" -Level "ERROR"
        exit 1
    }
}

# Entry point
Start-Uninstallation