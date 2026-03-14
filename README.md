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