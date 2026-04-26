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

You only need this section if you want **Start Node** to launch a real local provider daemon while running `npm run tauri dev`.

A "real provider" means the actual Python provider from the `smainer-backend` repository, or a standalone `smainer-provider.exe` built from it. If you only want to work on the desktop UI, you can ignore this section.

#### Option A: Run the Python provider from a local `smainer-backend` clone

Example setup:

- `python.exe` = the Python interpreter that has the backend provider dependencies installed.
  Example: `C:\Users\you\code\smainer-backend\.venv\Scripts\python.exe`
- `SMAINER_PROVIDER_CWD` = the `provider` folder inside your local `smainer-backend` clone.
  Example: `C:\Users\you\code\smainer-backend\provider`

```powershell
$env:SMAINER_PROVIDER_CMD = "C:\Users\you\code\smainer-backend\.venv\Scripts\python.exe"
$env:SMAINER_PROVIDER_ARGS = '["-m","src.provider.main"]'
$env:SMAINER_PROVIDER_CWD = "C:\Users\you\code\smainer-backend\provider"
npm run tauri dev
```

#### Option B: Run a standalone provider binary

If you already have a built `smainer-provider.exe`, point the desktop app to it directly:

```powershell
$env:SMAINER_PROVIDER_CMD = "C:\Users\you\Downloads\smainer-provider.exe"
npm run tauri dev
```

If you do not have `smainer-backend` cloned and you do not have a standalone provider binary, skip this section and use the CI-built desktop installer instead.

### Build MSI Locally

```powershell
npm run tauri build -- --bundles msi
```

This creates an MSI without the bundled provider daemon. The resulting installer will show the same "not bundled" message.

---

Releases page: https://github.com/Smainer/smainer-desktop/releases
