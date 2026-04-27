# Ollama Installation Guide

Quick setup guide for installing Ollama runtime to enable AI inference tasks on your Smainer provider node.

## Auto Install (Recommended)

The Smainer Desktop app can attempt automatic installation on supported systems.

### Windows Auto Install
1. **Enable Auto Install**: Check "Install Ollama automatically" in AI Setup tab
2. **Administrator Required**: Windows will prompt for admin permissions  
3. **Success Indicators**: 
   - Green "Available" badge appears in Ollama Runtime card
   - System Check shows "AI Runtime: Ready"
4. **If Auto Install Fails**: See Manual Install section below

### Linux Auto Install  
1. **Enable Auto Install**: Check "Install Ollama automatically" in AI Setup tab
2. **Installation Path**: Ollama installs to `/usr/local/bin/ollama`
3. **Success Indicators**:
   - Service starts automatically on port 11434
   - Smainer app detects runtime within 30 seconds
4. **If Auto Install Fails**: Run manual commands below

### macOS Auto Install
1. **Enable Auto Install**: Check "Install Ollama automatically" in AI Setup tab  
2. **Homebrew Required**: Auto install uses `brew install ollama`
3. **Success Indicators**: Ollama app appears in Applications folder
4. **If Auto Install Fails**: Download installer manually

## Manual Install (Fallback)

### Windows Manual Steps
```bash
# Download installer
curl -fsSL https://ollama.ai/install.sh | sh

# Or download from ollama.ai website
# Run OllamaSetup.exe as Administrator
```

**Verify Installation**:
- Open PowerShell: `ollama --version`
- Should return version number (e.g., "0.1.32")

### Linux Manual Steps  
```bash
# Single command install
curl -fsSL https://ollama.ai/install.sh | sh

# Start service  
sudo systemctl enable ollama
sudo systemctl start ollama

# Verify running
curl http://localhost:11434/api/version
```

**Troubleshooting**: If port 11434 blocked, check firewall settings.

### macOS Manual Steps
```bash
# Using Homebrew (recommended)
brew install ollama

# Or download .app from ollama.ai
# Drag Ollama.app to Applications folder
# Launch from Applications
```

## Post-Install Verification

After installation (auto or manual), verify Ollama is working:

### In Smainer Desktop App
- AI Setup tab shows "Ollama Runtime: Available" 
- System Check passes AI validation
- Model selection becomes available

### Command Line Check
```bash
# Check service status
ollama --version

# Test API endpoint  
curl http://localhost:11434/api/version
```

**Expected Response**: JSON with version information

## Common Issues

### "Ollama Not Found" Error
**Cause**: Installation incomplete or service not started  
**Fix**: Restart Smainer Desktop app after ensuring Ollama service is running

### "Port 11434 Unavailable"  
**Cause**: Another application using the port  
**Fix**: Restart system or kill conflicting processes

### "Auto Install Failed"
**Cause**: Network restrictions or insufficient permissions  
**Fix**: Use manual installation commands above

### Model Download Slow
**Cause**: Large model files (several GB each)  
**Fix**: Ensure stable internet connection, downloads continue in background

## Technical Details

- **Default Port**: 11434
- **API Endpoint**: http://localhost:11434  
- **Config Location**: `~/.ollama/` (all platforms)
- **Model Storage**: `~/.ollama/models/`
- **Service Name**: `ollama` (Linux), OllamaService (Windows)

## Next Steps

After successful installation:
1. Return to Smainer Desktop AI Setup tab
2. Select models compatible with your hardware
3. Complete node registration 
4. Monitor earnings in Dashboard tab

**Support**: For installation issues, check Ollama documentation at ollama.ai/docs or report via Smainer support channels.