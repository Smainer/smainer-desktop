# Smainer Node — Windows

Earn STRK by running AI compute on your PC.

## Download

**[⬇ Download Smainer-setup.exe](https://github.com/Smainer/smainer-desktop/releases/latest/download/Smainer_0.1.2_x64-setup.exe)**

> Double-click → Next → Install → Done. No command line. No admin needed.

---

## Requirements

### Basic Node Operation
- Windows 10 / 11 (64-bit)
- 8 GB RAM
- 20 GB free disk space
- Internet connection (10+ Mbps recommended)
- GPU optional but earns more

### AI Inference Capabilities (Optional)

AI serving enables your node to run language models for higher rewards but requires additional resources.

#### Model Requirements

| Model | VRAM | RAM | Disk | GPU Required | Network | Performance | Earning Potential |
|-------|------|-----|------|--------------|---------|-------------|-------------------|
| **phi3:mini** | 2 GB | 4 GB | 2 GB | No (CPU compatible) | 10+ Mbps | Basic tasks | Low-Medium |
| **mistral:7b** | 4 GB | 8 GB | 4 GB | Yes | 25+ Mbps | General purpose | Medium |  
| **llama3.1:8b** | 6 GB | 8 GB | 5 GB | Yes | 50+ Mbps | Advanced tasks | High |
| **llama3.1:70b** | 48 GB | 64 GB | 40 GB | Yes (Multi-GPU) | 100+ Mbps | Professional | Very High |

#### System Recommendations by Use Case

**Laptop/Basic PC (CPU-only)**
- phi3:mini only
- 8+ GB RAM, integrated graphics OK
- Suitable for text tasks, basic inference

**Gaming PC (Single GPU)**
- RTX 3060 (12GB) or RTX 4060 (16GB): mistral:7b + llama3.1:8b
- RTX 3080/4070 (16GB+): All models except 70B
- 16+ GB RAM recommended

**Workstation/Server (High-end)**
- RTX 4090 (24GB) or multiple GPUs: All models including 70B variants
- 32+ GB RAM for optimal performance
- NVMe SSD recommended for model loading

#### Privacy Modes

| Mode | Description | Task Eligibility | Data Sharing |
|------|-------------|------------------|--------------|
| **Standard** | Normal operation with standard telemetry | All tasks available | Basic metrics, no conversation content |
| **Enhanced** | Minimal logging, reduced telemetry | Most tasks (95%+) | Performance metrics only |
| **Maximum** | Local processing only, no external calls | Limited tasks (60-80%) | None (local only) |

#### Prerequisites

- **Ollama Runtime**: Required for all AI serving. Auto-installed by desktop app or available at [ollama.ai](https://ollama.ai)
- **Model Downloads**: Range from 2GB (phi3:mini) to 40GB (llama3.1:70b). Downloaded automatically on first use.
- **Network Stability**: Consistent connection required during inference tasks to maintain SLA commitments.

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
