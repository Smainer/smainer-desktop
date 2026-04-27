# Smainer Desktop App Data Cleanup - Verification Script
# This script verifies the cleanup functionality works correctly

$ErrorActionPreference = "Stop"

Write-Host "`n=== Smainer Cleanup Verification Test ===" -ForegroundColor Cyan
Write-Host "This script will verify app data cleanup functionality`n"

$SMAINER_DIR = Join-Path -Path $env:USERPROFILE -ChildPath ".smainer"
$TEST_MARKER = "___VERIFICATION_TEST___"

function Write-TestStep {
    param([string]$Message, [string]$Status = "INFO")
    $color = switch ($Status) {
        "PASS" { "Green" }
        "FAIL" { "Red" }
        "INFO" { "Cyan" }
        default { "White" }
    }
    Write-Host "[$Status] $Message" -ForegroundColor $color
}

function Test-CleanupScriptExists {
    Write-Host "`n--- Test 1: Cleanup Script Exists ---"
    $scriptPath = Join-Path -Path $PSScriptRoot -ChildPath "cleanup-app-data.ps1"
    
    if (Test-Path -Path $scriptPath) {
        Write-TestStep "Cleanup script found at: $scriptPath" -Status "PASS"
        return $true
    } else {
        Write-TestStep "Cleanup script NOT found at: $scriptPath" -Status "FAIL"
        return $false
    }
}

function Test-CreateTestData {
    Write-Host "`n--- Test 2: Create Test Data ---"
    
    try {
        # Create test directory
        if (-not (Test-Path -Path $SMAINER_DIR)) {
            New-Item -Path $SMAINER_DIR -ItemType Directory -Force | Out-Null
        }
        
        # Create test wallet.json
        $testWallet = @{
            private_key = "$TEST_MARKER-PRIVATE-KEY"
            public_key = "$TEST_MARKER-PUBLIC-KEY"
            address = "$TEST_MARKER-ADDRESS"
            encrypted = $false
        } | ConvertTo-Json
        
        $walletPath = Join-Path -Path $SMAINER_DIR -ChildPath "wallet.json"
        Set-Content -Path $walletPath -Value $testWallet -Force
        
        # Create test ai_config.json
        $testConfig = @{
            model_type = "$TEST_MARKER-MODEL"
            backend_url = "http://localhost:11434"
        } | ConvertTo-Json
        
        $configPath = Join-Path -Path $SMAINER_DIR -ChildPath "ai_config.json"
        Set-Content -Path $configPath -Value $testConfig -Force
        
        # Verify created
        if ((Test-Path -Path $walletPath) -and (Test-Path -Path $configPath)) {
            Write-TestStep "Test data created successfully" -Status "PASS"
            Write-Host "  - $walletPath" -ForegroundColor Gray
            Write-Host "  - $configPath" -ForegroundColor Gray
            return $true
        } else {
            Write-TestStep "Failed to create test data" -Status "FAIL"
            return $false
        }
    }
    catch {
        Write-TestStep "Error creating test data: $($_.Exception.Message)" -Status "FAIL"
        return $false
    }
}

function Test-DataExistsCheck {
    Write-Host "`n--- Test 3: Verify Data Exists ---"
    
    $walletPath = Join-Path -Path $SMAINER_DIR -ChildPath "wallet.json"
    $configPath = Join-Path -Path $SMAINER_DIR -ChildPath "ai_config.json"
    
    $walletExists = Test-Path -Path $walletPath
    $configExists = Test-Path -Path $configPath
    $dirExists = Test-Path -Path $SMAINER_DIR
    
    if ($walletExists -and $configExists -and $dirExists) {
        Write-TestStep "All test files exist and are accessible" -Status "PASS"
        return $true
    } else {
        Write-TestStep "Some test files are missing" -Status "FAIL"
        Write-Host "  wallet.json: $walletExists" -ForegroundColor Gray
        Write-Host "  ai_config.json: $configExists" -ForegroundColor Gray
        Write-Host "  .smainer dir: $dirExists" -ForegroundColor Gray
        return $false
    }
}

function Test-CleanupWithConfirmation {
    Write-Host "`n--- Test 4: Run Cleanup Script (Interactive) ---"
    Write-Host "This test requires manual interaction..." -ForegroundColor Yellow
    
    $scriptPath = Join-Path -Path $PSScriptRoot -ChildPath "cleanup-app-data.ps1"
    
    Write-Host "`nRunning: $scriptPath" -ForegroundColor Cyan
    Write-Host "INSTRUCTIONS: When prompted, type 'DELETE' to confirm cleanup`n" -ForegroundColor Yellow
    
    Start-Sleep -Seconds 2
    
    try {
        & $scriptPath
        $cleanupExitCode = $LASTEXITCODE
        
        if ($cleanupExitCode -eq 0) {
            Write-TestStep "Cleanup script executed successfully" -Status "PASS"
            return $true
        } else {
            Write-TestStep "Cleanup script exited with code: $cleanupExitCode" -Status "FAIL"
            return $false
        }
    }
    catch {
        Write-TestStep "Error running cleanup script: $($_.Exception.Message)" -Status "FAIL"
        return $false
    }
}

function Test-CleanupForced {
    Write-Host "`n--- Test 5: Run Cleanup Script (Force Mode) ---"
    
    $scriptPath = Join-Path -Path $PSScriptRoot -ChildPath "cleanup-app-data.ps1"
    
    Write-Host "Running: $scriptPath -Force" -ForegroundColor Cyan
    
    try {
        & $scriptPath -Force
        $cleanupExitCode = $LASTEXITCODE
        
        if ($cleanupExitCode -eq 0) {
            Write-TestStep "Forced cleanup executed successfully" -Status "PASS"
            return $true
        } else {
            Write-TestStep "Forced cleanup exited with code: $cleanupExitCode" -Status "FAIL"
            return $false
        }
    }
    catch {
        Write-TestStep "Error running forced cleanup: $($_.Exception.Message)" -Status "FAIL"
        return $false
    }
}

function Test-DataRemoved {
    Write-Host "`n--- Test 6: Verify Data Removed ---"
    
    $walletPath = Join-Path -Path $SMAINER_DIR -ChildPath "wallet.json"
    $configPath = Join-Path -Path $SMAINER_DIR -ChildPath "ai_config.json"
    
    $walletExists = Test-Path -Path $walletPath
    $configExists = Test-Path -Path $configPath
    $dirExists = Test-Path -Path $SMAINER_DIR
    
    if (-not $walletExists -and -not $configExists -and -not $dirExists) {
        Write-TestStep "All app data successfully removed" -Status "PASS"
        return $true
    } else {
        Write-TestStep "Some data still exists after cleanup" -Status "FAIL"
        Write-Host "  wallet.json exists: $walletExists" -ForegroundColor Gray
        Write-Host "  ai_config.json exists: $configExists" -ForegroundColor Gray
        Write-Host "  .smainer dir exists: $dirExists" -ForegroundColor Gray
        return $false
    }
}

function Test-PreserveOption {
    Write-Host "`n--- Test 7: Test Preserve Option ---"
    
    # Create test data again
    if (-not (Test-CreateTestData)) {
        Write-TestStep "Could not create test data for preserve test" -Status "FAIL"
        return $false
    }
    
    Write-Host "`nRunning cleanup script..." -ForegroundColor Cyan
    Write-Host "INSTRUCTIONS: When prompted, press Enter (DO NOT type DELETE)`n" -ForegroundColor Yellow
    
    Start-Sleep -Seconds 2
    
    $scriptPath = Join-Path -Path $PSScriptRoot -ChildPath "cleanup-app-data.ps1"
    
    try {
        & $scriptPath
        
        # Check if data still exists
        $walletPath = Join-Path -Path $SMAINER_DIR -ChildPath "wallet.json"
        $stillExists = Test-Path -Path $walletPath
        
        if ($stillExists) {
            Write-TestStep "Data correctly preserved when cleanup cancelled" -Status "PASS"
            return $true
        } else {
            Write-TestStep "Data was deleted despite cancellation" -Status "FAIL"
            return $false
        }
    }
    catch {
        Write-TestStep "Error during preserve test: $($_.Exception.Message)" -Status "FAIL"
        return $false
    }
}

function Cleanup-TestData {
    Write-Host "`n--- Cleanup Test Data ---"
    
    if (Test-Path -Path $SMAINER_DIR) {
        try {
            Remove-Item -Path $SMAINER_DIR -Recurse -Force
            Write-TestStep "Test data cleaned up successfully" -Status "PASS"
        }
        catch {
            Write-TestStep "Could not clean up test data: $($_.Exception.Message)" -Status "FAIL"
        }
    }
}

# Run tests
Write-Host "`nStarting verification tests...`n"

$results = @{}
$results["script_exists"] = Test-CleanupScriptExists

if ($results["script_exists"]) {
    $results["create_data"] = Test-CreateTestData
    $results["data_check"] = Test-DataExistsCheck
    
    if ($results["data_check"]) {
        # Run forced cleanup test
        $results["cleanup_forced"] = Test-CleanupForced
        $results["data_removed"] = Test-DataRemoved
        
        # Run preserve test (requires recreation of data)
        $results["preserve_test"] = Test-PreserveOption
    }
}

# Final cleanup
Cleanup-TestData

# Summary
Write-Host "`n=== Test Summary ===" -ForegroundColor Cyan
$passCount = ($results.Values | Where-Object { $_ -eq $true }).Count
$totalCount = $results.Count

foreach ($test in $results.Keys) {
    $status = if ($results[$test]) { "✓ PASS" } else { "✗ FAIL" }
    $color = if ($results[$test]) { "Green" } else { "Red" }
    Write-Host "$status - $test" -ForegroundColor $color
}

Write-Host "`nTotal: $passCount / $totalCount tests passed" -ForegroundColor Cyan

if ($passCount -eq $totalCount) {
    Write-Host "`n✓ All tests passed! Cleanup functionality verified." -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n✗ Some tests failed. Please review output above." -ForegroundColor Red
    exit 1
}
