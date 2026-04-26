# Smainer Node — Windows

Earn STRK by running AI compute on your PC.

## Download

**[⬇ Download Smainer-setup.exe](https://github.com/Smainer/smainer-desktop/releases/latest/download/Smainer_0.1.0_x64-setup.exe)**

> Double-click → Next → Install → Done. No command line. No admin needed.

---

## Requirements

- Windows 10 / 11 (64-bit)
- 8 GB RAM
- GPU optional but earns more

## What happens after install

1. App opens automatically
2. Connect your Starknet wallet (Argent X or Braavos)
3. Your node registers and starts accepting tasks
4. Earnings appear in the dashboard in real time

---

## Run locally on Windows

For development on Windows, clone this repository and run:

```powershell
npm ci
npm run tauri dev
```

The app will start in development mode. When you click **Start Node**, you'll see:

> "Provider sidecar not bundled in this build. Set SMAINER_PROVIDER_CMD to a provider executable, or run a release build produced by CI."

### Provider Override (Optional)

To test with a real provider, set these environment variables before running `npm run tauri dev`:

```powershell
# Use Python provider from backend clone
$env:SMAINER_PROVIDER_CMD = "C:\path\to\python.exe"
$env:SMAINER_PROVIDER_ARGS = '["-m","src.provider.main"]'  
$env:SMAINER_PROVIDER_CWD = "C:\path\to\smainer-backend\provider"

# Or use a standalone binary
$env:SMAINER_PROVIDER_CMD = "C:\path\to\smainer-provider.exe"
```

### Build MSI Locally

```powershell
npm run tauri build -- --bundles msi
```

This creates an MSI without the bundled provider daemon. The resulting installer will show the same "not bundled" message.

---

Releases page: https://github.com/Smainer/smainer-desktop/releases
