# Smainer Desktop

Windows desktop application for easy Smainer provider node onboarding and management.

## Features

- 🔍 **Hardware Detection**: Automatic GPU, CPU, and RAM detection
- 👛 **Wallet Management**: Generate or import Starknet wallets securely
- 🚀 **Node Registration**: Easy registration with the Smainer network
- 📊 **Real-time Dashboard**: Monitor node status, earnings, and performance
- 📋 **Task History**: Track completed AI tasks and earnings
- ⚙️ **Settings Management**: Configure provider behavior and system resources

## Prerequisites

- Windows 10/11
- NVIDIA or AMD GPU with 4GB+ VRAM (recommended)
- 8GB+ RAM
- Smainer relayer running on localhost:8000

## Installation

### Easy Installer (Recommended)

Use the interactive PowerShell installer for a complete setup with Windows service:

```powershell
# Run as Administrator
.\scripts\install-smainer-daemon.ps1
```

**Dry-run mode** (test without making changes):
```powershell
.\scripts\install-smainer-daemon.ps1 -DryRun
```

**Configuration-only** (no service installation):
```powershell
.\scripts\install-smainer-daemon.ps1 -ConfigOnly
```

### Installation Questions

The installer will ask you to configure:

1. **Install Mode**:
   - `[D]` Desktop-only test mode (no Windows service)
   - `[F]` Full daemon install with Windows service (default)

2. **Relayer URL**: Default is `https://api.smainer.io`
   - Use `http://localhost:8000` for local development
   - HTTPS required for production relayers (security validation)

3. **Node ID**: Alphanumeric identifier for your provider node

4. **Wallet Configuration**: 
   - Enter your Starknet private key (input hidden)
   - Automatically encrypted and stored in Windows Credential Manager
   - **Security**: Never echoed to console or logs

5. **Ollama Installation**: 
   - `[Y]` Install Ollama AI runtime (recommended)
   - Optionally pull llama3.1:8b model (4.7GB download)

6. **TransformerLab**: 
   - `[y/N]` Install TransformerLab (advanced users)
   - Manual installation steps provided

7. **Service Auto-start**: 
   - `[Y]` Start daemon automatically on Windows boot

### What Gets Installed

- **Program Files**: `C:\Program Files\Smainer\`
- **Configuration**: `C:\ProgramData\Smainer\config.json` 
- **Logs**: `C:\ProgramData\Smainer\Logs\`
- **Service Account**: `SmaineRProvider` (dedicated low-privilege user)
- **Windows Service**: `SmaiserProviderDaemon`
- **Encrypted Credentials**: Windows Credential Manager

### Managing the Service

Use the service management utility:

```powershell
# Start the daemon
.\scripts\manage-smainer-service.ps1 start

# Stop the daemon  
.\scripts\manage-smainer-service.ps1 stop

# Check status and view recent logs
.\scripts\manage-smainer-service.ps1 status

# View detailed logs
.\scripts\manage-smainer-service.ps1 logs

# Enable auto-start on boot
.\scripts\manage-smainer-service.ps1 enable

# Disable auto-start
.\scripts\manage-smainer-service.ps1 disable
```

Or use Windows service commands directly:
```cmd
sc.exe start SmaiserProviderDaemon
sc.exe stop SmaiserProviderDaemon  
sc.exe query SmaiserProviderDaemon
```

### Uninstallation

Complete removal of all components:

```powershell
# Full uninstall (requires confirmation)
.\scripts\uninstall-smainer-daemon.ps1

# Force uninstall (skip prompts)
.\scripts\uninstall-smainer-daemon.ps1 -Force

# Keep logs during uninstall
.\scripts\uninstall-smainer-daemon.ps1 -KeepLogs

# Preserve Ollama installation
.\scripts\uninstall-smainer-daemon.ps1 -KeepOllama
```

**Security Note**: Uninstallation permanently removes encrypted private keys from Windows Credential Manager. This action cannot be undone.

## Development Setup

Install dependencies:
```bash
npm install
```

Start development server:
```bash
npm run tauri dev
```

Build production app:
```bash
npm run tauri build
```

## Project Structure

```
desktop/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── commands/    # Tauri commands
│   │   ├── models/      # Data structures
│   │   └── utils/       # Helper functions
│   └── Cargo.toml
├── src/                 # React frontend
│   ├── components/
│   │   ├── onboarding/  # Setup wizard
│   │   ├── dashboard/   # Main dashboard
│   │   ├── settings/    # Configuration
│   │   └── ui/          # Reusable components
│   ├── hooks/           # Custom React hooks
│   └── lib/             # Utilities
└── package.json
```

## Core Commands

### Hardware Detection
- `detect_gpus()` - Enumerate available GPUs
- `get_system_info()` - Get complete system specifications
- `check_requirements()` - Validate system capabilities

### Wallet Management
- `generate_wallet(password?)` - Create new Starknet wallet
- `import_wallet(privateKey, password?)` - Import existing wallet
- `sign_message(message, password?)` - Sign messages with wallet

### Provider Management
- `start_provider(config)` - Launch provider daemon
- `stop_provider()` - Stop provider daemon
- `get_provider_status()` - Get current provider state
- `register_node(registration)` - Register node with relayer

### Monitoring
- `get_node_status()` - Real-time node metrics
- `get_earnings()` - Earnings data and history
- `get_task_history(limit?)` - Completed task history

## Architecture

- **Frontend**: React + TypeScript + Tailwind CSS
- **Backend**: Rust with Tauri framework
- **Communication**: JSON-based IPC between frontend and backend
- **Security**: Windows Credential Manager for key storage
- **Packaging**: MSI installer for Windows distribution

## Security Features

- Private keys encrypted with user passwords
- Secure storage using Windows Credential Manager
- Process isolation between frontend and provider daemon
- Input sanitization and validation
- Sandboxed provider execution

## Building for Production

1. Ensure all dependencies are installed
2. Run `npm run tauri build`
3. MSI installer will be created in `src-tauri/target/release/bundle/msi/`
4. For code signing, configure certificate in `tauri.conf.json`

## Troubleshooting

### GPU Not Detected
- Update GPU drivers
- Check Windows Device Manager
- Restart application

### Provider Won't Start
- Check if relayer is running on localhost:8000
- Verify Windows Firewall settings
- Check provider logs in Settings > Export Logs

### Wallet Issues
- Ensure password is correctly entered
- Try importing wallet again
- Check Windows Credential Manager for stored keys

## Contributing

This desktop application focuses specifically on Windows provider node onboarding. 
For web interfaces or Telegram integration, see other components in the Smainer ecosystem.

## License

MIT License - see LICENSE file for details.