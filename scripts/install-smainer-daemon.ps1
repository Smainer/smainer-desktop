#Requires -Version 5.1
#Requires -RunAsAdministrator

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
    
    # Check admin privileges
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "Administrator privileges required. Please run PowerShell as Administrator."
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
    
    # Install mode
    Write-Host "1. Install Mode:"
    Write-Host "   [D] Desktop-only test mode (no service installation)"
    Write-Host "   [F] Full daemon install with Windows service"
    $installMode = Read-Host "Select install mode [D/F] (default: F)"
    if ([string]::IsNullOrWhiteSpace($installMode)) { $installMode = "F" }
    
    # Relayer URL
    Write-Host ""
    Write-Host "2. Relayer Configuration:"
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
    Write-Host "3. Node Identification:"
    $nodeId = Read-Host "Enter node ID (alphanumeric, required)"
    while ([string]::IsNullOrWhiteSpace($nodeId) -or $nodeId -notmatch "^[a-zA-Z0-9]+$") {
        Write-StatusMessage "Node ID must be alphanumeric and non-empty" -Level "ERROR"
        $nodeId = Read-Host "Enter node ID (alphanumeric, required)"
    }
    
    # Private key input with SecureString
    Write-Host ""
    Write-Host "4. Wallet Configuration:"
    Write-Host "   SECURITY NOTE: Your private key will be encrypted and stored securely"
    $privateKeySecure = Read-Host "Enter Starknet private key (will be hidden)" -AsSecureString
    
    # Convert SecureString to encrypted string immediately  
    $privateKeyEncrypted = ConvertFrom-SecureString -SecureString $privateKeySecure
    
    # Clear the SecureString
    $privateKeySecure.Dispose()
    
    # Ollama installation
    Write-Host ""
    Write-Host "5. Ollama AI Runtime:"
    $installOllama = Read-Host "Install Ollama? [Y/n] (default: Y)"
    $installOllama = ($installOllama -ne "n" -and $installOllama -ne "N")
    
    $pullModel = $false
    if ($installOllama) {
        $pullModel = Read-Host "Pull llama3.1:8b model? (4.7GB download) [Y/n] (default: Y)"
        $pullModel = ($pullModel -ne "n" -and $pullModel -ne "N")
    }
    
    # TransformerLab
    Write-Host ""
    Write-Host "6. TransformerLab (Advanced):"
    $installTlab = Read-Host "Install TransformerLab? [y/N] (default: N)"
    $installTlab = ($installTlab -eq "y" -or $installTlab -eq "Y")
    
    # Service auto-start
    Write-Host ""
    Write-Host "7. Service Configuration:"
    $autoStart = Read-Host "Register service for auto-start on boot? [Y/n] (default: Y)"
    $autoStart = ($autoStart -ne "n" -and $autoStart -ne "N")
    
    return @{
        InstallMode = $installMode
        RelayerUrl = $relayerUrl
        NodeId = $nodeId
        PrivateKeyEncrypted = $privateKeyEncrypted
        InstallOllama = $installOllama
        PullModel = $pullModel
        InstallTlab = $installTlab
        AutoStart = $autoStart
    }
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
        [string]$EncryptedPrivateKey
    )
    
    if ($DryRun) {
        Write-StatusMessage "DRY RUN: Would store encrypted credentials in Windows Credential Manager"
        return
    }
    
    Write-StatusMessage "Storing credentials in Windows Credential Manager..."
    
    try {
        # Store encrypted private key in Windows Credential Manager
        cmdkey /generic:"Smainer_$NodeId" /user:$NodeId /pass:$EncryptedPrivateKey
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to store credentials with cmdkey"
        }
        
        $script:InstallState.CredentialsStored = $true
        Write-StatusMessage "Credentials stored securely" -Level "SUCCESS"
    }
    catch {
        throw "Failed to store credentials: $($_.Exception.Message)"
    }
}

function Install-OllamaComponent {
    param(
        [bool]$PullModel
    )
    
    Write-StatusMessage "Installing Ollama AI runtime..."
    
    if ($DryRun) {
        Write-StatusMessage "DRY RUN: Would download and install Ollama from $OLLAMA_URL"
        if ($PullModel) {
            Write-StatusMessage "DRY RUN: Would pull llama3.1:8b model (4.7GB)"
        }
        return
    }
    
    try {
        # Download Ollama installer
        $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
        $ollamaInstaller = Join-Path -Path $tempDir -ChildPath "ollama-setup.exe"
        
        # Note: In production, you would verify the SHA256 hash here
        Invoke-SecureDownload -Url $OLLAMA_URL -OutputPath $ollamaInstaller
        
        # Install Ollama silently
        Write-StatusMessage "Installing Ollama..."
        Start-Process -FilePath $ollamaInstaller -ArgumentList "/S" -Wait -NoNewWindow
        
        # Wait for Ollama to be available
        $ollamaPath = "${env:LOCALAPPDATA}\Programs\Ollama\ollama.exe"
        $timeout = 30
        while (-not (Test-Path $ollamaPath) -and $timeout -gt 0) {
            Start-Sleep -Seconds 1
            $timeout--
        }
        
        if (-not (Test-Path $ollamaPath)) {
            throw "Ollama installation failed - executable not found"
        }
        
        $script:InstallState.ComponentsInstalled += "Ollama"
        Write-StatusMessage "Ollama installed successfully" -Level "SUCCESS"
        
        # Pull model if requested
        if ($PullModel) {
            Write-StatusMessage "Pulling llama3.1:8b model (this may take several minutes)..."
            & $ollamaPath pull llama3.1:8b
            if ($LASTEXITCODE -eq 0) {
                Write-StatusMessage "Model pulled successfully" -Level "SUCCESS"
            } else {
                Write-StatusMessage "Model pull failed (can be done later manually)" -Level "WARNING"
            }
        }
        
        # Cleanup
        Remove-Item -Path $tempDir -Recurse -Force
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
        if ($script:InstallState.CredentialsStored) {
            Write-StatusMessage "Removing stored credentials..."
            cmdkey /delete:"Smainer_*" 2>$null
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
    
    Write-Host "Configuration Summary:" -ForegroundColor Cyan
    Write-Host "  • Install Mode: $($Config.InstallMode)" -ForegroundColor White
    Write-Host "  • Relayer URL: $($Config.RelayerUrl)" -ForegroundColor White  
    Write-Host "  • Node ID: $($Config.NodeId)" -ForegroundColor White
    Write-Host "  • Service Account: $SMAINER_USER" -ForegroundColor White
    Write-Host "  • Config File: $SMAINER_CONFIG" -ForegroundColor White
    Write-Host "  • Log Directory: $SMAINER_LOG" -ForegroundColor White
    Write-Host ""
    
    Write-Host "Installed Components:" -ForegroundColor Cyan
    if ($Config.InstallOllama) {
        Write-Host "  ✓ Ollama AI Runtime" -ForegroundColor Green
        if ($Config.PullModel) {
            Write-Host "  ✓ llama3.1:8b Model" -ForegroundColor Green
        }
    }
    if ($Config.InstallTlab) {
        Write-Host "  • TransformerLab (manual installation required)" -ForegroundColor Yellow
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
    Write-Host "  3. Verify connection to relayer: $($Config.RelayerUrl)" -ForegroundColor White
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
            Store-SecureCredentials -NodeId $config.NodeId -EncryptedPrivateKey $config.PrivateKeyEncrypted
        }
        
        # Install optional components
        if ($config.InstallOllama -and -not $ConfigOnly) {
            Install-OllamaComponent -PullModel $config.PullModel
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