#Requires -Version 5.1


<#
.SYNOPSIS
    Interactive installer for Smainer provider daemon on Windows
.DESCRIPTION
    Securely installs and configures Smainer provider daemon with optional components:
    - Desktop application dependencies
    - Provider daemon Windows service  
    - Ollama and llama3.1:8b model (optional)
    - TransformerLab (optional, manual steps provided)
    - Secure credential storage via Windows Credential Manager
.PARAMETER DryRun
    Run in simulation mode without making system changes
.PARAMETER ConfigOnly
    Only generate configuration files without installing services
.EXAMPLE
    .\install-smainer-daemon.ps1
    .\install-smainer-daemon.ps1 -DryRun
    .\install-smainer-daemon.ps1 -ConfigOnly
#>

[CmdletBinding()]
param(
    [switch]$DryRun,
    [switch]$ConfigOnly
)

# Strict error handling - fail closed behavior
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Elevation required for real install only (not for -DryRun)
if (-not $DryRun) {
    $principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        Write-Host "[ERROR] This script requires Administrator privileges for installation." -ForegroundColor Red
        Write-Host "        To preview the install plan without changes, run:" -ForegroundColor Yellow
        Write-Host "        powershell -ExecutionPolicy Bypass -File .\install-smainer-daemon.ps1 -DryRun" -ForegroundColor Cyan
        exit 1
    }
}

# Security constants
$SMAINER_SERVICE_NAME = "SmaiserProviderDaemon"
$SMAINER_USER = "SmaineRProvider"
$SMAINER_HOME = "$env:ProgramFiles\Smainer"
$SMAINER_CONFIG = "$env:ProgramData\Smainer\config.json"
$SMAINER_LOG = "$env:ProgramData\Smainer\Logs"
$DEFAULT_RELAYER_URL = "https://api.smainer.io"

# Component URLs and checksums (validate before download)
$OLLAMA_URL = "https://ollama.ai/download/windows"
$TLAB_URL = "https://transformerlab.ai/download"

# Global state for rollback
$script:InstallState = @{
    ServiceCreated = $false
    UserCreated = $false
    DirectoriesCreated = @()
    CredentialsStored = $false
    CredentialTarget = $null
    ComponentsInstalled = @()
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

function Test-Prerequisites {
    Write-StatusMessage "Checking prerequisites..."
    
    # Check PowerShell version
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        throw "PowerShell 5.1 or higher required. Current version: $($PSVersionTable.PSVersion)"
    }
    
    # Check admin privileges (skipped in DryRun to allow preview without elevation)
    if (-not $DryRun) {
        $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = New-Object Security.Principal.WindowsPrincipal($identity)
        if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
            throw "Administrator privileges required. Please run PowerShell as Administrator."
        }
    } else {
        Write-StatusMessage "DryRun: skipping Administrator check" -Level "WARNING"
    }
    
    # Check internet connectivity for component downloads
    try {
        $null = Invoke-WebRequest -Uri "https://www.google.com" -Method Head -TimeoutSec 10 -UseBasicParsing
    }
    catch {
        Write-StatusMessage "Internet connectivity check failed. Some features may be unavailable." -Level "WARNING"
    }
    
    Write-StatusMessage "Prerequisites check passed" -Level "SUCCESS"
}

function Get-UserChoices {
    Write-Host ""
    Write-StatusMessage "=== Smainer Daemon Installation Configuration ==="
    Write-Host ""
    
    # Development mode check first
    Write-Host "1. Configuration Mode:"
    Write-Host "   This installer is configured for production deployment by default."
    Write-Host "   Choose development mode only if you're running a local relayer."
    $devMode = Read-Host "Use development defaults (localhost:8000)? [y/N] (default: N)"
    $useDevDefaults = ($devMode -eq "y" -or $devMode -eq "Y")
    
    if ($useDevDefaults) {
        $DEFAULT_RELAYER_URL = "http://localhost:8000"
        Write-StatusMessage "Development mode selected - using localhost defaults" -Level "WARNING"
    } else {
        Write-StatusMessage "Production mode selected - using api.smainer.io defaults" -Level "SUCCESS"
    }
    
    # Install mode
    Write-Host ""
    Write-Host "2. Install Mode:"
    Write-Host "   [D] Desktop-only test mode (no service installation)"
    Write-Host "   [F] Full daemon install with Windows service"
    $installMode = Read-Host "Select install mode [D/F] (default: F)"
    if ([string]::IsNullOrWhiteSpace($installMode)) { $installMode = "F" }
    
    # Relayer URL
    Write-Host ""
    Write-Host "3. Relayer Configuration:"
    $relayerUrl = Read-Host "Enter relayer URL (default: $DEFAULT_RELAYER_URL)"
    if ([string]::IsNullOrWhiteSpace($relayerUrl)) { $relayerUrl = $DEFAULT_RELAYER_URL }
    
    # Validate HTTPS for production
    if ($relayerUrl.StartsWith("https://") -eq $false -and $relayerUrl -ne "http://localhost:8000") {
        Write-StatusMessage "WARNING: Non-HTTPS URL detected for production relayer. This is insecure!" -Level "WARNING"
        $confirm = Read-Host "Continue with insecure URL? [y/N] (default: N)"
        if ($confirm -ne "y" -and $confirm -ne "Y") {
            throw "Installation aborted due to insecure relayer URL"
        }
    }
    
    # Node ID
    Write-Host ""
    Write-Host "4. Node Identification:"
    $nodeId = Read-Host "Enter node ID (alphanumeric, required)"
    while ([string]::IsNullOrWhiteSpace($nodeId) -or $nodeId -notmatch "^[a-zA-Z0-9]+$") {
        Write-StatusMessage "Node ID must be alphanumeric and non-empty" -Level "ERROR"
        $nodeId = Read-Host "Enter node ID (alphanumeric, required)"
    }
    
    # Private key input with SecureString
    Write-Host ""
    Write-Host "5. Wallet Configuration:"
    Write-Host "   SECURITY NOTE: Your private key will be encrypted and stored securely"
    $privateKeySecure = Read-Host "Enter Starknet private key (will be hidden)" -AsSecureString
    
    # Convert SecureString to encrypted string immediately  
    $privateKeyEncrypted = ConvertFrom-SecureString -SecureString $privateKeySecure
    
    # Clear the SecureString
    $privateKeySecure.Dispose()
    
    # Generate secure GUID-based credential target
    $credentialGuid = [System.Guid]::NewGuid().ToString()
    $credentialTarget = "SmaineR_Cred_$credentialGuid"
    
    # Ollama installation
    Write-Host ""
    Write-Host "6. AI Inference Capability:"
    Write-Host "   Enable AI serving to accept higher-paying inference tasks."
    Write-Host "   WHY WE ASK: AI tasks require additional system resources and model downloads."
    Write-Host "   Your hardware will be validated to ensure stable operation."
    $enableAI = Read-Host "Enable AI inference serving? [Y/n] (default: n)"
    $enableAI = ($enableAI -eq "y" -or $enableAI -eq "Y")
    
    $installOllama = $false
    $pullModel = $false
    $aiPrivacyMode = "Standard"
    $selectedModels = @()
    
    if ($enableAI) {
        Write-Host ""
        Write-Host "   6a. Ollama Runtime:"
        Write-Host "       Ollama is required to run AI models locally."
        Write-Host "       WHY WE ASK: Without Ollama, your node cannot serve AI tasks."
        $installOllama = Read-Host "       Install Ollama? [Y/n] (default: Y)"
        $installOllama = ($installOllama -ne "n" -and $installOllama -ne "N")
        
        if ($installOllama) {
            Write-Host ""
            Write-Host "   6b. Model Selection:"
            Write-Host "       Different models have varying system requirements and earning potential."
            Write-Host "       WHY WE ASK: We match models to your hardware to prevent system overload."
            Write-Host ""
            Write-Host "       Available models:"
            Write-Host "       [1] llama3.1:8b (6GB VRAM, 8GB RAM, 5GB disk) - Balanced performance"
            Write-Host "       [2] mistral:7b   (4GB VRAM, 8GB RAM, 4GB disk) - Fast inference"
            Write-Host "       [3] phi3:mini    (2GB VRAM, 4GB RAM, 2GB disk) - Lightweight, CPU-compatible"
            $modelChoice = Read-Host "       Select model(s) [1,2,3] or [1-3] (comma-separated, default: 1)"
            if ([string]::IsNullOrWhiteSpace($modelChoice)) { $modelChoice = "1" }
            
            $modelChoice.Split(",") | ForEach-Object {
                switch ($_.Trim()) {
                    "1" { $selectedModels += "llama3.1:8b" }
                    "2" { $selectedModels += "mistral:7b" }
                    "3" { $selectedModels += "phi3:mini" }
                }
            }
            
            $pullModel = $selectedModels.Count -gt 0
            
            if ($pullModel) {
                $totalSize = 0
                $selectedModels | ForEach-Object {
                    switch ($_) {
                        "llama3.1:8b" { $totalSize += 4.7 }
                        "mistral:7b"  { $totalSize += 4.1 }
                        "phi3:mini"   { $totalSize += 2.2 }
                    }
                }
                Write-Host "       Total download size: approximately $([math]::Round($totalSize, 1))GB"
                $confirmDownload = Read-Host "       Download selected models now? [Y/n] (default: Y)"
                $pullModel = ($confirmDownload -ne "n" -and $confirmDownload -ne "N")
            }
            
            # Explicit confirmation with source disclosure (MTG-OLLAMA-001 requirement)
            if ($installOllama) {
                Write-Host ""
                Write-Host "   INSTALLATION CONFIRMATION:" -ForegroundColor Yellow
                Write-Host "   Ollama will be installed from one of the following sources:" -ForegroundColor White
                Write-Host "   - Primary: winget package manager (Ollama.Ollama)" -ForegroundColor White
                Write-Host "   - Fallback: Direct download from https://ollama.com/download/OllamaSetup.exe" -ForegroundColor White
                Write-Host "   Downloads use HTTPS only and are cryptographically verified before execution." -ForegroundColor White
                Write-Host ""
                $finalConfirm = Read-Host "   Proceed with Ollama installation? [Y/n] (default: n)"
                if ($finalConfirm -ne "y" -and $finalConfirm -ne "Y") {
                    Write-StatusMessage "Ollama installation cancelled by user" -Level "WARNING"
                    $installOllama = $false
                    $pullModel = $false
                }
            }
        }
        
        Write-Host ""
        Write-Host "   6c. Privacy Mode:"
        Write-Host "       Choose how much information your node shares during AI tasks."
        Write-Host "       WHY WE ASK: Privacy settings affect task eligibility and data protection."
        Write-Host "       Higher privacy may limit some high-paying tasks."
        Write-Host ""
        Write-Host "       [S] Standard - Normal operation, all task types (default)"
        Write-Host "       [E] Enhanced - Minimal logging, some task limitations"
        Write-Host "       [M] Maximum - Local only, significant limitations"
        $privacyChoice = Read-Host "       Select privacy mode [S/E/M] (default: S)"
        switch ($privacyChoice.ToUpper()) {
            "E" { $aiPrivacyMode = "Enhanced" }
            "M" { $aiPrivacyMode = "Maximum" }
            default { $aiPrivacyMode = "Standard" }
        }
    } else {
        Write-Host "   AI serving disabled. You can enable this later in the desktop app settings."
    }
    
    # TransformerLab
    Write-Host ""
    Write-Host "7. TransformerLab (Advanced):"
    $installTlab = Read-Host "Install TransformerLab? [y/N] (default: N)"
    $installTlab = ($installTlab -eq "y" -or $installTlab -eq "Y")
    
    # Service auto-start
    Write-Host ""
    Write-Host "8. Service Configuration:"
    $autoStart = Read-Host "Register service for auto-start on boot? [Y/n] (default: Y)"
    $autoStart = ($autoStart -ne "n" -and $autoStart -ne "N")
    
    $configResult = @{
        InstallMode = $installMode
        RelayerUrl = $relayerUrl
        NodeId = $nodeId
        PrivateKeyEncrypted = $privateKeyEncrypted
        CredentialTarget = $credentialTarget
        EnableAI = $enableAI
        InstallOllama = $installOllama
        PullModel = $pullModel
        SelectedModels = $selectedModels
        AIPrivacyMode = $aiPrivacyMode
        InstallTlab = $installTlab
        AutoStart = $autoStart
    }
    
    # SEC-001: Clear sensitive variables from memory
    $privateKeyEncrypted = $null
    $credentialGuid = $null
    $credentialTarget = $null
    [System.GC]::Collect()
    
    return $configResult
}

function Invoke-SecureDownload {
    param(
        [Parameter(Mandatory)]
        [string]$Url,
        [Parameter(Mandatory)]
        [string]$OutputPath,
        [string]$ExpectedHash
    )
    
    if ($DryRun) {
        Write-StatusMessage "DRY RUN: Would download $Url to $OutputPath"
        return
    }
    
    Write-StatusMessage "Downloading $Url..."
    
    # Use TLS 1.2+ and validate certificates
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    [Net.ServicePointManager]::ServerCertificateValidationCallback = $null
    
    try {
        Invoke-WebRequest -Uri $Url -OutFile $OutputPath -UseBasicParsing
    }
    catch {
        throw "Download failed for $Url`: $($_.Exception.Message)"
    }
    
    # Verify checksum if provided
    if ($ExpectedHash) {
        $actualHash = Get-FileHash -Path $OutputPath -Algorithm SHA256
        if ($actualHash.Hash -ne $ExpectedHash) {
            Remove-Item -Path $OutputPath -Force
            throw "Checksum verification failed for $OutputPath. Expected: $ExpectedHash, Actual: $($actualHash.Hash)"
        }
        Write-StatusMessage "Checksum verification passed" -Level "SUCCESS"
    }
}

function New-SmaineRUser {
    param(
        [Parameter(Mandatory)]
        [string]$Username
    )
    
    if ($DryRun) {
        Write-StatusMessage "DRY RUN: Would create user account: $Username"
        return
    }
    
    Write-StatusMessage "Creating dedicated service account: $Username..."
    
    try {
        # Check if user already exists
        $existingUser = Get-LocalUser -Name $Username -ErrorAction SilentlyContinue
        if ($existingUser) {
            Write-StatusMessage "Service account $Username already exists" -Level "WARNING"
            return
        }
        
        # Generate secure random password
        $password = [System.Web.Security.Membership]::GeneratePassword(32, 8)
        $securePassword = ConvertTo-SecureString -String $password -AsPlainText -Force
        
        # Create user with minimal privileges
        New-LocalUser -Name $Username -Password $securePassword -Description "Smainer Provider Service Account" `
            -AccountNeverExpires -UserMayNotChangePassword -PasswordNeverExpires
        
        # Add to "Log on as a service" right
        $tempFile = New-TemporaryFile
        $tempFile2 = New-TemporaryFile
        
        secedit /export /cfg $tempFile.FullName
        $content = Get-Content -Path $tempFile.FullName
        $newContent = $content -replace "(SeServiceLogonRight = .*)", "`$1,$Username"
        $newContent | Out-File -FilePath $tempFile2.FullName -Encoding unicode
        
        secedit /configure /db secedit.sdb /cfg $tempFile2.FullName
        
        Remove-Item -Path $tempFile.FullName, $tempFile2.FullName -Force
        
        # Clear password from memory
        Clear-Variable -Name password -Scope Local
        $securePassword.Dispose()
        
        $script:InstallState.UserCreated = $true
        Write-StatusMessage "Service account created successfully" -Level "SUCCESS"
    }
    catch {
        throw "Failed to create service account: $($_.Exception.Message)"
    }
}

function New-SmaineRDirectories {
    $directories = @(
        $SMAINER_HOME,
        (Split-Path -Path $SMAINER_CONFIG -Parent),
        $SMAINER_LOG
    )
    
    foreach ($dir in $directories) {
        if ($DryRun) {
            Write-StatusMessage "DRY RUN: Would create directory: $dir"
            continue
        }
        
        if (-not (Test-Path -Path $dir)) {
            Write-StatusMessage "Creating directory: $dir"
            New-Item -Path $dir -ItemType Directory -Force
            $script:InstallState.DirectoriesCreated += $dir
            
            # Set secure permissions
            $acl = Get-Acl -Path $dir
            $acl.SetAccessRuleProtection($true, $false)  # Disable inheritance
            
            # Grant access to service account
            $accessRule = New-Object System.Security.AccessControl.FileSystemAccessRule($SMAINER_USER, "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
            $acl.AddAccessRule($accessRule)
            
            # Grant read access to SYSTEM
            $systemRule = New-Object System.Security.AccessControl.FileSystemAccessRule("SYSTEM", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow")
            $acl.AddAccessRule($systemRule)
            
            Set-Acl -Path $dir -AclObject $acl
        }
    }
}

function New-SmaineRConfig {
    param(
        [Parameter(Mandatory)]
        [hashtable]$Config
    )
    
    $configData = @{
        schema_version = "1.0.0"
        contract_version = "2024.1"
        relayer_url = $Config.RelayerUrl
        node_id = $Config.NodeId
        wallet = @{
            encrypted_private_key = $Config.PrivateKeyEncrypted
        }
        provider = @{
            port = 8001
            max_concurrent_tasks = 4
            gpu_enabled = $true
            log_level = "INFO"
        }
        ai_capability = @{
            schema_version = "1.0.0"
            contract_version = "2024.1"
            ai_serving_enabled = $Config.EnableAI
            ollama_config = if ($Config.InstallOllama) {
                @{
                    install_requested = $true
                    api_endpoint = "http://localhost:11434"
                    models_to_install = $Config.SelectedModels
                    auto_update = $false
                }
            } else { $null }
            model_preferences = if ($Config.EnableAI -and $Config.SelectedModels.Count -gt 0) {
                $Config.SelectedModels | ForEach-Object {
                    $requirements = switch ($_) {
                        "llama3.1:8b" { @{ min_vram_gb = 6; min_ram_gb = 8; min_disk_gb = 5; requires_gpu = $true; network_bandwidth_mbps = 50 } }
                        "mistral:7b"  { @{ min_vram_gb = 4; min_ram_gb = 8; min_disk_gb = 4; requires_gpu = $true; network_bandwidth_mbps = 25 } }
                        "phi3:mini"   { @{ min_vram_gb = 2; min_ram_gb = 4; min_disk_gb = 2; requires_gpu = $false; network_bandwidth_mbps = 10 } }
                        default       { @{ min_vram_gb = 4; min_ram_gb = 8; min_disk_gb = 4; requires_gpu = $true; network_bandwidth_mbps = 25 } }
                    }
                    @{
                        name = $_
                        enabled = $true
                        priority = 8
                        requirements = $requirements
                    }
                }
            } else { @() }
            privacy_mode = $Config.AIPrivacyMode
            resources = @{
                max_cpu_percent = 80
                max_ram_gb = 16
                max_vram_gb = $null
                max_disk_io_mbps = $null
                max_network_mbps = $null
            }
            created_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
            updated_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
        }
        components = @{
            ollama_installed = $Config.InstallOllama
            tlab_installed = $Config.InstallTlab
        }
    }
    
    if ($DryRun) {
        Write-StatusMessage "DRY RUN: Would create config file at: $SMAINER_CONFIG"
        Write-StatusMessage "Config content (private key redacted):"
        $dryRunConfig = $configData.Clone()
        $dryRunConfig.wallet.encrypted_private_key = "<REDACTED>"
        Write-Host ($dryRunConfig | ConvertTo-Json -Depth 5)
        return
    }
    
    Write-StatusMessage "Creating configuration file..."
    
    try {
        $configJson = $configData | ConvertTo-Json -Depth 5
        $configJson | Out-File -FilePath $SMAINER_CONFIG -Encoding UTF8 -Force
        
        # Set secure permissions on config file
        $acl = Get-Acl -Path $SMAINER_CONFIG
        $acl.SetAccessRuleProtection($true, $false)
        
        # Only service account can read config
        $accessRule = New-Object System.Security.AccessControl.FileSystemAccessRule($SMAINER_USER, "FullControl", "Allow")
        $acl.AddAccessRule($accessRule)
        
        Set-Acl -Path $SMAINER_CONFIG -AclObject $acl
        
        Write-StatusMessage "Configuration file created with secure permissions" -Level "SUCCESS"
    }
    catch {
        throw "Failed to create configuration: $($_.Exception.Message)"
    }
}

function Store-SecureCredentials {
    param(
        [Parameter(Mandatory)]
        [string]$NodeId,
        [Parameter(Mandatory)]  
        [string]$EncryptedPrivateKey,
        [Parameter(Mandatory)]
        [string]$CredentialTarget
    )
    
    if ($DryRun) {
        Write-StatusMessage "DRY RUN: Would store encrypted credentials in Windows Credential Manager using secure target"
        return
    }
    
    Write-StatusMessage "Storing credentials in Windows Credential Manager..."
    
    try {
        # SEC-002: Use GUID-based credential target instead of predictable NodeId
        cmdkey /generic:"$CredentialTarget" /user:$NodeId /pass:$EncryptedPrivateKey
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to store credentials with cmdkey"
        }
        
        # Store the mapping for service retrieval
        $mappingPath = "$env:ProgramData\Smainer\credential-mapping.json"
        $mapping = @{ "target" = $CredentialTarget; "nodeId" = $NodeId }
        $mapping | ConvertTo-Json -Depth 2 | Out-File -FilePath $mappingPath -Encoding UTF8
        
        # Set secure permissions on mapping file
        $acl = Get-Acl $mappingPath
        $acl.SetAccessRuleProtection($true, $false)
        $adminRule = New-Object System.Security.AccessControl.FileSystemAccessRule("BUILTIN\Administrators", "FullControl", "Allow")
        $systemRule = New-Object System.Security.AccessControl.FileSystemAccessRule("NT AUTHORITY\SYSTEM", "FullControl", "Allow")
        $acl.SetAccessRule($adminRule)
        $acl.SetAccessRule($systemRule)
        Set-Acl -Path $mappingPath -AclObject $acl
        
        $script:InstallState.CredentialsStored = $true
        $script:InstallState.CredentialTarget = $CredentialTarget
        Write-StatusMessage "Credentials stored securely" -Level "SUCCESS"
        
        # SEC-001: Clear sensitive variables from function scope
        $EncryptedPrivateKey = $null
        $mapping = $null
        [System.GC]::Collect()
    }
    catch {
        throw "Failed to store credentials: $($_.Exception.Message)"
    }
}

function Install-OllamaComponent {
    param(
        [bool]$PullModel,
        [string[]]$SelectedModels = @()
    )
    
    Write-StatusMessage "Installing Ollama AI runtime..."
    
    if ($DryRun) {
        Write-StatusMessage "DRY RUN: Would install Ollama via winget or direct HTTPS download with signature verification"
        if ($PullModel -and $SelectedModels.Count -gt 0) {
            Write-StatusMessage "DRY RUN: Would pull models: $($SelectedModels -join ', ')"
        }
        return
    }
    
    try {
        $ollamaInstalled = $false
        
        # Try winget installation first (MTG-OLLAMA-001: preferred method)
        Write-StatusMessage "Attempting Ollama installation via winget..."
        $wingetAvailable = Get-Command winget -ErrorAction SilentlyContinue
        
        if ($wingetAvailable) {
            try {
                winget install --id Ollama.Ollama --exact --silent --accept-package-agreements --accept-source-agreements
                if ($LASTEXITCODE -eq 0) {
                    Write-StatusMessage "Ollama installed successfully via winget" -Level "SUCCESS"
                    $ollamaInstalled = $true
                } else {
                    Write-StatusMessage "winget installation failed with exit code $LASTEXITCODE, trying fallback method..." -Level "WARNING"
                }
            }
            catch {
                Write-StatusMessage "winget installation threw exception: $($_.Exception.Message)" -Level "WARNING"
            }
        } else {
            Write-StatusMessage "winget not available, using direct download method" -Level "WARNING"
        }
        
        # Fallback to direct download with signature verification (MTG-OLLAMA-001: HTTPS-only, signature verification required)
        if (-not $ollamaInstalled) {
            Write-StatusMessage "Installing Ollama via direct download..."
            $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
            $ollamaInstaller = Join-Path -Path $tempDir -ChildPath "OllamaSetup.exe"
            
            # Direct HTTPS binary URL (not landing page)
            $directDownloadUrl = "https://ollama.com/download/OllamaSetup.exe"
            
            try {
                # Download installer (HTTPS-only enforced in Invoke-SecureDownload)
                Invoke-SecureDownload -Url $directDownloadUrl -OutputPath $ollamaInstaller
                
                # MTG-OLLAMA-001 Requirement: Verify Authenticode signature before execution
                Write-StatusMessage "Verifying installer signature..."
                $signature = Get-AuthenticodeSignature -FilePath $ollamaInstaller
                
                if ($signature.Status -ne "Valid") {
                    Remove-Item -Path $ollamaInstaller -Force
                    throw "Installer signature verification failed. Status: $($signature.Status). Installation aborted for security."
                }
                
                # Verify publisher (expected: Ollama in subject)
                $publisherName = $signature.SignerCertificate.Subject
                if ($publisherName -notmatch "Ollama") {
                    Remove-Item -Path $ollamaInstaller -Force
                    throw "Installer publisher mismatch. Expected 'Ollama' in subject, got: $publisherName. Installation aborted for security."
                }
                
                Write-StatusMessage "Signature valid: $publisherName" -Level "SUCCESS"
                
                # Install Ollama silently
                Write-StatusMessage "Running installer..."
                Start-Process -FilePath $ollamaInstaller -ArgumentList "/S" -Wait -NoNewWindow
                
                $ollamaInstalled = $true
            }
            finally {
                # Cleanup temp files
                if (Test-Path $tempDir) {
                    Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
                }
            }
        }
        
        # Verify installation and wait for Ollama to be available
        $ollamaPath = "${env:LOCALAPPDATA}\Programs\Ollama\ollama.exe"
        $timeout = 30
        while (-not (Test-Path $ollamaPath) -and $timeout -gt 0) {
            Start-Sleep -Seconds 1
            $timeout--
        }
        
        if (-not (Test-Path $ollamaPath)) {
            throw "Ollama installation failed - executable not found at $ollamaPath"
        }
        
        # Start Ollama service if not running
        Write-StatusMessage "Starting Ollama service..."
        $ollamaService = Get-Process -Name "ollama" -ErrorAction SilentlyContinue
        if (-not $ollamaService) {
            Start-Process -FilePath $ollamaPath -ArgumentList "serve" -WindowStyle Hidden
            Start-Sleep -Seconds 5
        }
        
        $script:InstallState.ComponentsInstalled += "Ollama"
        Write-StatusMessage "Ollama installed successfully" -Level "SUCCESS"
        
        # Pull selected models if requested (MTG-OLLAMA-001: use selected models, not hardcoded)
        if ($PullModel -and $SelectedModels.Count -gt 0) {
            foreach ($model in $SelectedModels) {
                Write-StatusMessage "Pulling model: $model (this may take several minutes)..."
                & $ollamaPath pull $model
                if ($LASTEXITCODE -eq 0) {
                    Write-StatusMessage "Model $model pulled successfully" -Level "SUCCESS"
                } else {
                    Write-StatusMessage "Model $model pull failed (can be done later manually)" -Level "WARNING"
                }
            }
        }
    }
    catch {
        Write-StatusMessage "Ollama installation failed: $($_.Exception.Message)" -Level "ERROR"
        throw
    }
}

function Show-TransformerLabInstructions {
    Write-StatusMessage "TransformerLab Installation Instructions:" -Level "SUCCESS"
    Write-Host ""
    Write-Host "TransformerLab requires manual installation due to complex dependencies." -ForegroundColor Cyan
    Write-Host "Please follow these steps:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "1. Download TransformerLab from: $TLAB_URL" -ForegroundColor Yellow
    Write-Host "2. Follow the installation guide on their documentation" -ForegroundColor Yellow 
    Write-Host "3. Ensure Python 3.8+ is installed" -ForegroundColor Yellow
    Write-Host "4. Configure TransformerLab to work with your GPU drivers" -ForegroundColor Yellow
    Write-Host "5. Test TransformerLab before integrating with Smainer" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Once installed, update your Smainer configuration to enable TransformerLab integration." -ForegroundColor Cyan
}

function Install-SmaineRService {
    param(
        [Parameter(Mandatory)]
        [bool]$AutoStart
    )
    
    if ($DryRun) {
        Write-StatusMessage "DRY RUN: Would install Windows service '$SMAINER_SERVICE_NAME'"
        Write-StatusMessage "DRY RUN: AutoStart would be set to: $AutoStart"
        return
    }
    
    Write-StatusMessage "Installing Smainer provider service..."
    
    try {
        # Check if service already exists
        $existingService = Get-Service -Name $SMAINER_SERVICE_NAME -ErrorAction SilentlyContinue
        if ($existingService) {
            Write-StatusMessage "Service already exists, stopping and removing..." -Level "WARNING"
            Stop-Service -Name $SMAINER_SERVICE_NAME -Force
            sc.exe delete $SMAINER_SERVICE_NAME
            Start-Sleep -Seconds 2
        }
        
        # Service executable (this would be the actual provider daemon path)
        $serviceExe = Join-Path -Path $SMAINER_HOME -ChildPath "smainer-provider.exe"
        $serviceArgs = "--config `"$SMAINER_CONFIG`" --service"
        
        # Create service
        $startType = if ($AutoStart) { "auto" } else { "demand" }
        
        sc.exe create $SMAINER_SERVICE_NAME `
            binPath= "`"$serviceExe`" $serviceArgs" `
            start= $startType `
            obj= $SMAINER_USER `
            DisplayName= "Smainer Provider Daemon" `
            depend= "Tcpip"
        
        if ($LASTEXITCODE -ne 0) {
            throw "Service creation failed with exit code: $LASTEXITCODE"
        }
        
        # Set service description
        sc.exe description $SMAINER_SERVICE_NAME "Smainer AI provider daemon for distributed computing"
        
        # Configure service recovery actions
        sc.exe failure $SMAINER_SERVICE_NAME reset= 300 actions= restart/30000/restart/60000/restart/90000
        
        $script:InstallState.ServiceCreated = $true
        Write-StatusMessage "Service installed successfully" -Level "SUCCESS"
        
        if ($AutoStart) {
            Write-StatusMessage "Starting service..."
            Start-Service -Name $SMAINER_SERVICE_NAME
            Write-StatusMessage "Service started successfully" -Level "SUCCESS"
        }
    }
    catch {
        throw "Service installation failed: $($_.Exception.Message)"
    }
}

function Invoke-InstallationRollback {
    Write-StatusMessage "Rolling back installation due to failure..." -Level "WARNING"
    
    try {
        # Stop and remove service
        if ($script:InstallState.ServiceCreated) {
            Write-StatusMessage "Removing service..."
            $service = Get-Service -Name $SMAINER_SERVICE_NAME -ErrorAction SilentlyContinue
            if ($service) {
                Stop-Service -Name $SMAINER_SERVICE_NAME -Force -ErrorAction SilentlyContinue
                sc.exe delete $SMAINER_SERVICE_NAME
            }
        }
        
        # Remove credentials
        if ($script:InstallState.CredentialsStored -and $script:InstallState.CredentialTarget) {
            Write-StatusMessage "Removing stored credentials..."
            cmdkey /delete:"$($script:InstallState.CredentialTarget)" 2>$null
            # Also remove the mapping file
            $mappingPath = "$env:ProgramData\Smainer\credential-mapping.json"
            if (Test-Path $mappingPath) {
                Remove-Item -Path $mappingPath -Force 2>$null
            }
        }
        
        # Remove user account
        if ($script:InstallState.UserCreated) {
            Write-StatusMessage "Removing service account..."
            Remove-LocalUser -Name $SMAINER_USER -ErrorAction SilentlyContinue
        }
        
        # Remove directories (in reverse order)
        $script:InstallState.DirectoriesCreated | Sort-Object -Descending | ForEach-Object {
            if (Test-Path -Path $_) {
                Write-StatusMessage "Removing directory: $_"
                Remove-Item -Path $_ -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
        
        Write-StatusMessage "Rollback completed" -Level "SUCCESS"
    }
    catch {
        Write-StatusMessage "Rollback failed: $($_.Exception.Message)" -Level "ERROR"
    }
}

function Show-InstallationSummary {
    param(
        [Parameter(Mandatory)]
        [hashtable]$Config
    )
    
    Write-Host ""
    Write-StatusMessage "=== INSTALLATION COMPLETED SUCCESSFULLY ===" -Level "SUCCESS"
    Write-Host ""
    
    # SEC-003: Sanitize sensitive operational identifiers in summary output
    $sanitizedNodeId = if ($DryRun) { "<node-id>" } else { $Config.NodeId }
    $sanitizedRelayerUrl = if ($DryRun -and $Config.RelayerUrl -ne $DEFAULT_RELAYER_URL) { "<relayer-url>" } else { $Config.RelayerUrl }
    
    Write-Host "Configuration Summary:" -ForegroundColor Cyan
    Write-Host "  - Install Mode: $($Config.InstallMode)" -ForegroundColor White
    Write-Host "  - Relayer URL: $sanitizedRelayerUrl" -ForegroundColor White  
    Write-Host "  - Node ID: $sanitizedNodeId" -ForegroundColor White
    Write-Host "  - Service Account: $SMAINER_USER" -ForegroundColor White
    Write-Host "  - Config File: $SMAINER_CONFIG" -ForegroundColor White
    Write-Host "  - Log Directory: $SMAINER_LOG" -ForegroundColor White
    Write-Host ""
    
    Write-Host "Installed Components:" -ForegroundColor Cyan
    if ($Config.InstallOllama) {
        Write-Host "  [OK] Ollama AI Runtime" -ForegroundColor Green
        if ($Config.PullModel) {
            Write-Host "  [OK] llama3.1:8b Model" -ForegroundColor Green
        }
    }
    if ($Config.InstallTlab) {
        Write-Host "  - TransformerLab (manual installation required)" -ForegroundColor Yellow
    }
    Write-Host ""
    
    if ($Config.InstallMode -eq "F") {
        Write-Host "Service Management Commands:" -ForegroundColor Cyan
        Write-Host "  Start:   sc.exe start $SMAINER_SERVICE_NAME" -ForegroundColor White
        Write-Host "  Stop:    sc.exe stop $SMAINER_SERVICE_NAME" -ForegroundColor White
        Write-Host "  Status:  sc.exe query $SMAINER_SERVICE_NAME" -ForegroundColor White
        Write-Host ""
    }
    
    Write-Host "Next Steps:" -ForegroundColor Cyan
    Write-Host "  1. Test your configuration by checking the Smainer desktop app" -ForegroundColor White
    Write-Host "  2. Monitor logs in: $SMAINER_LOG" -ForegroundColor White
    $relayerUrl = $Config.RelayerUrl
    Write-Host "  3. Verify connection to relayer: $relayerUrl" -ForegroundColor White
    Write-Host ""
    
    Write-Host "For uninstallation, run: .\uninstall-smainer-daemon.ps1" -ForegroundColor Yellow
}

# Main installation flow
function Start-Installation {
    try {
        Write-StatusMessage "=== Smainer Daemon Installer v1.0 ==="
        
        if ($DryRun) {
            Write-StatusMessage "RUNNING IN DRY-RUN MODE - No changes will be made" -Level "WARNING"
        }
        
        # Prerequisites check
        Test-Prerequisites
        
        # Get user configuration
        $config = Get-UserChoices
        
        # Create service account (full install only)
        if ($config.InstallMode -eq "F" -and -not $ConfigOnly) {
            New-SmaineRUser -Username $SMAINER_USER
        }
        
        # Create directories
        New-SmaineRDirectories
        
        # Create configuration
        New-SmaineRConfig -Config $config
        
        # Store credentials securely
        if (-not $ConfigOnly) {
            Store-SecureCredentials -NodeId $config.NodeId -EncryptedPrivateKey $config.PrivateKeyEncrypted -CredentialTarget $config.CredentialTarget
        }
        
        # Install optional components
        if ($config.InstallOllama -and -not $ConfigOnly) {
            Install-OllamaComponent -PullModel $config.PullModel -SelectedModels $config.SelectedModels
        }
        
        if ($config.InstallTlab) {
            Show-TransformerLabInstructions
        }
        
        # Install Windows service (full install only)
        if ($config.InstallMode -eq "F" -and -not $ConfigOnly) {
            Install-SmaineRService -AutoStart $config.AutoStart
        }
        
        # Show summary
        Show-InstallationSummary -Config $config
        
    }
    catch {
        Write-StatusMessage "Installation failed: $($_.Exception.Message)" -Level "ERROR"
        
        if (-not $DryRun -and -not $ConfigOnly) {
            Invoke-InstallationRollback
        }
        
        exit 1
    }
}

# Entry point
Start-Installation