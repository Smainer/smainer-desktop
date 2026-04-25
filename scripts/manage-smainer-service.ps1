#Requires -Version 5.1

<#
.SYNOPSIS
    Smainer daemon service management utility
.DESCRIPTION
    Provides easy commands to start, stop, restart, and check status of the Smainer provider daemon service
.PARAMETER Action
    Service action: start, stop, restart, status, logs
.EXAMPLE
    .\manage-smainer-service.ps1 start
    .\manage-smainer-service.ps1 status
    .\manage-smainer-service.ps1 logs
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet("start", "stop", "restart", "status", "logs", "enable", "disable")]
    [string]$Action
)

# Constants
$SMAINER_SERVICE_NAME = "SmaiserProviderDaemon"
$SMAINER_LOG = "$env:ProgramData\Smainer\Logs"

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

function Get-ServiceInfo {
    try {
        $service = Get-Service -Name $SMAINER_SERVICE_NAME -ErrorAction Stop
        return $service
    }
    catch {
        Write-StatusMessage "Smainer service not found. Is it installed?" -Level "ERROR"
        exit 1
    }
}

function Show-ServiceStatus {
    $service = Get-ServiceInfo
    
    Write-Host ""
    Write-Host "=== Smainer Provider Daemon Status ===" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Service Name: $($service.Name)" -ForegroundColor White
    Write-Host "Display Name: $($service.DisplayName)" -ForegroundColor White
    Write-Host "Status: $($service.Status)" -ForegroundColor $(if ($service.Status -eq "Running") { "Green" } else { "Red" })
    Write-Host "Start Type: $($service.StartType)" -ForegroundColor White
    
    if ($service.Status -eq "Running") {
        # Get process information
        $processes = Get-WmiObject -Class Win32_Service | Where-Object {$_.Name -eq $SMAINER_SERVICE_NAME}
        if ($processes.ProcessId) {
            $process = Get-Process -Id $processes.ProcessId -ErrorAction SilentlyContinue
            if ($process) {
                Write-Host "Process ID: $($process.Id)" -ForegroundColor White
                Write-Host "CPU Time: $($process.TotalProcessorTime)" -ForegroundColor White
                Write-Host "Memory Usage: $([math]::Round($process.WorkingSet64 / 1MB, 2)) MB" -ForegroundColor White
                Write-Host "Started: $($process.StartTime)" -ForegroundColor White
            }
        }
    }
    
    Write-Host ""
    
    # Check recent logs for errors
    $logFile = Join-Path -Path $SMAINER_LOG -ChildPath "provider.log"
    if (Test-Path -Path $logFile) {
        $recentLogs = Get-Content -Path $logFile -Tail 5 -ErrorAction SilentlyContinue
        if ($recentLogs) {
            Write-Host "Recent Log Entries:" -ForegroundColor Cyan
            foreach ($line in $recentLogs) {
                if ($line -match "ERROR|FATAL") {
                    Write-Host "  $line" -ForegroundColor Red
                } elseif ($line -match "WARN") {
                    Write-Host "  $line" -ForegroundColor Yellow
                } else {
                    Write-Host "  $line" -ForegroundColor Gray
                }
            }
        }
    }
}

function Start-SmaineRService {
    $service = Get-ServiceInfo
    
    if ($service.Status -eq "Running") {
        Write-StatusMessage "Service is already running" -Level "INFO"
        return
    }
    
    Write-StatusMessage "Starting Smainer provider daemon..."
    
    try {
        Start-Service -Name $SMAINER_SERVICE_NAME
        
        # Wait for service to start
        $timeout = 30
        do {
            Start-Sleep -Seconds 1
            $service.Refresh()
            $timeout--
        } while ($service.Status -ne "Running" -and $timeout -gt 0)
        
        if ($service.Status -eq "Running") {
            Write-StatusMessage "Service started successfully" -Level "SUCCESS"
        } else {
            Write-StatusMessage "Service failed to start within timeout" -Level "ERROR"
        }
    }
    catch {
        Write-StatusMessage "Failed to start service: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Stop-SmaineRService {
    $service = Get-ServiceInfo
    
    if ($service.Status -eq "Stopped") {
        Write-StatusMessage "Service is already stopped" -Level "INFO"
        return
    }
    
    Write-StatusMessage "Stopping Smainer provider daemon..."
    
    try {
        Stop-Service -Name $SMAINER_SERVICE_NAME -Force
        
        # Wait for service to stop
        $timeout = 30
        do {
            Start-Sleep -Seconds 1
            $service.Refresh()
            $timeout--
        } while ($service.Status -ne "Stopped" -and $timeout -gt 0)
        
        if ($service.Status -eq "Stopped") {
            Write-StatusMessage "Service stopped successfully" -Level "SUCCESS"
        } else {
            Write-StatusMessage "Service failed to stop within timeout" -Level "WARNING"
        }
    }
    catch {
        Write-StatusMessage "Failed to stop service: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Restart-SmaineRService {
    Write-StatusMessage "Restarting Smainer provider daemon..."
    Stop-SmaineRService
    Start-Sleep -Seconds 2
    Start-SmaineRService
}

function Enable-SmaineRService {
    Write-StatusMessage "Enabling auto-start for Smainer service..."
    
    try {
        Set-Service -Name $SMAINER_SERVICE_NAME -StartupType Automatic
        Write-StatusMessage "Service set to start automatically on boot" -Level "SUCCESS"
    }
    catch {
        Write-StatusMessage "Failed to enable auto-start: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Disable-SmaineRService {
    Write-StatusMessage "Disabling auto-start for Smainer service..."
    
    try {
        Set-Service -Name $SMAINER_SERVICE_NAME -StartupType Manual
        Write-StatusMessage "Service set to manual start mode" -Level "SUCCESS"
    }
    catch {
        Write-StatusMessage "Failed to disable auto-start: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Show-ServiceLogs {
    $logFile = Join-Path -Path $SMAINER_LOG -ChildPath "provider.log"
    
    if (-not (Test-Path -Path $logFile)) {
        Write-StatusMessage "Log file not found: $logFile" -Level "ERROR"
        return
    }
    
    Write-Host ""
    Write-Host "=== Recent Smainer Provider Logs ===" -ForegroundColor Cyan
    Write-Host "Log file: $logFile" -ForegroundColor Gray
    Write-Host ""
    
    try {
        # Show last 50 lines with color coding
        $logs = Get-Content -Path $logFile -Tail 50
        foreach ($line in $logs) {
            if ($line -match "ERROR|FATAL") {
                Write-Host $line -ForegroundColor Red
            } elseif ($line -match "WARN") {
                Write-Host $line -ForegroundColor Yellow
            } elseif ($line -match "INFO") {
                Write-Host $line -ForegroundColor White
            } else {
                Write-Host $line -ForegroundColor Gray
            }
        }
        
        Write-Host ""
        Write-Host "To view live logs, run: Get-Content '$logFile' -Wait -Tail 10" -ForegroundColor Cyan
    }
    catch {
        Write-StatusMessage "Failed to read log file: $($_.Exception.Message)" -Level "ERROR"
    }
}

# Main execution
switch ($Action) {
    "start" { Start-SmaineRService; Show-ServiceStatus }
    "stop" { Stop-SmaineRService; Show-ServiceStatus }
    "restart" { Restart-SmaineRService; Show-ServiceStatus }
    "status" { Show-ServiceStatus }
    "logs" { Show-ServiceLogs }
    "enable" { Enable-SmaineRService }
    "disable" { Disable-SmaineRService }
}