use serde::{Deserialize, Serialize};
use tauri::{command, State};
use anyhow::Result;
use std::process::{Command, Stdio, Child};
use std::sync::Mutex;
use std::fs;
use std::path::PathBuf;
use crate::models::{ProviderStatus, NodeRegistration, AICapabilityConfig, AICapabilityReport, SystemValidation, ModelValidation, CompatibilityStatus};

// Global state managed by Tauri
pub struct ProviderState {
    pub process: Mutex<Option<Child>>,
    pub relayer_url: Mutex<String>,
}

impl Default for ProviderState {
    fn default() -> Self {
        Self {
            process: Mutex::new(None),
            relayer_url: Mutex::new("https://api.smainer.io".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub wallet_address: String,
    pub relayer_url: String,
    pub port: u16,
    pub max_tasks: u32,
    pub gpu_enabled: bool,
    pub auto_start: bool,
}

/// Convert HTTP(S) URL to WebSocket URL for provider sidecar
fn http_to_ws_url(url: &str) -> String {
    if url.starts_with("https://") {
        format!("wss://{}", &url[8..])
    } else if url.starts_with("http://") {
        format!("ws://{}", &url[7..])
    } else {
        url.to_string()
    }
}

/// Read Starknet private key from local wallet file (~/.smainer/wallet.json)
fn read_wallet_private_key() -> Option<String> {
    let mut path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push(".smainer");
    path.push("wallet.json");
    let content = std::fs::read_to_string(path).ok()?;
    let stored: serde_json::Value = serde_json::from_str(&content).ok()?;
    stored.get("private_key")?.as_str().map(|s| s.to_string())
}

/// Derive a short alphanumeric node_id from wallet address
fn node_id_from_address(addr: &str) -> String {
    let stripped = addr.trim_start_matches("0x");
    let id: String = stripped.chars().filter(|c| c.is_alphanumeric()).take(24).collect();
    if id.is_empty() { "default-node".to_string() } else { id }
}

#[command]
pub async fn start_provider(
    config: ProviderConfig,
    state: State<'_, ProviderState>
) -> Result<bool, String> {
    tracing::info!("Starting provider with config: {:?}", config);
    
    let mut process_guard = state.process.lock().map_err(|e| e.to_string())?;
    
    if process_guard.is_some() {
        return Ok(true); // Already running
    }

    // Update relayer URL in state
    if let Ok(mut url) = state.relayer_url.lock() {
        *url = config.relayer_url.clone();
    }
    
    // Determine command to run based on environment
    // In strict production, this would be a bundled binary
    // In dev, we can try to find the python script
    
    // Resolve exe directory (works in both dev and installed app)
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    // 1. Bundled sidecar with target triple suffix (Tauri externalBin convention)
    let sidecar_candidates = if cfg!(target_os = "windows") {
        vec![
            exe_dir.join("smainer-provider-x86_64-pc-windows-msvc.exe"),
            exe_dir.join("smainer-provider.exe"),
        ]
    } else {
        vec![
            exe_dir.join("smainer-provider-x86_64-unknown-linux-gnu"),
            exe_dir.join("smainer-provider"),
        ]
    };

    let sidecar_path = sidecar_candidates.iter().find(|p| p.exists());

    // Check for environment variable override first
    let mut cmd = if let Ok(custom_cmd) = std::env::var("SMAINER_PROVIDER_CMD") {
        // Environment override: use custom provider command
        let mut env_cmd = Command::new(&custom_cmd);
        
        // Apply custom args if provided
        if let Ok(custom_args) = std::env::var("SMAINER_PROVIDER_ARGS") {
            if let Ok(args_vec) = serde_json::from_str::<Vec<String>>(&custom_args) {
                env_cmd.args(&args_vec);
            }
        }
        
        // Apply custom working directory if provided
        if let Ok(custom_cwd) = std::env::var("SMAINER_PROVIDER_CWD") {
            env_cmd.current_dir(&custom_cwd);
        }
        
        env_cmd
    } else if let Some(path) = sidecar_path {
        // Production: use bundled sidecar binary
        Command::new(path)
    } else {
        // No sidecar and no environment override - return clear error
        return Err("Provider daemon not found. Use the installer build or set SMAINER_PROVIDER_CMD environment variable.".to_string());
    };
    
    // Pass required env vars to sidecar
    // Convert HTTP(S) → WS(S) — provider config.py rejects non-ws:// URLs
    let ws_url = http_to_ws_url(&config.relayer_url);
    cmd.env("RELAYER_WS_URL", &ws_url);
    cmd.env("STARKNET_ACCOUNT_ADDRESS", &config.wallet_address);
    // NODE_ID: derive from wallet address for stable identity
    let node_id = node_id_from_address(&config.wallet_address);
    cmd.env("NODE_ID", &node_id);
    // STARKNET_PRIVATE_KEY: required for WS auth signature — read from local wallet file
    if let Some(pk) = read_wallet_private_key() {
        cmd.env("STARKNET_PRIVATE_KEY", pk);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match cmd.spawn() {
        Ok(child) => {
            *process_guard = Some(child);
            tracing::info!("Provider started successfully");
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Failed to start provider: {}", e);
            Err(e.to_string())
        }
    }
}

#[command]
pub async fn stop_provider(state: State<'_, ProviderState>) -> Result<bool, String> {
    tracing::info!("Stopping provider...");
    let mut process_guard = state.process.lock().map_err(|e| e.to_string())?;
    
    if let Some(mut child) = process_guard.take() {
        child.kill().map_err(|e| e.to_string())?;
        child.wait().ok(); // Avoid zombie process
        tracing::info!("Provider stopped");
    }
    
    Ok(true)
}

#[command]
pub async fn get_provider_status(state: State<'_, ProviderState>) -> Result<ProviderStatus, String> {
    // Check local process status
    let is_running = {
        let mut guard = state.process.lock().map_err(|e| e.to_string())?;
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    *guard = None; // Process exited
                    false
                }
                Ok(None) => true, // Still running
                Err(_) => false,
            }
        } else {
            false
        }
    };
    
    let _relayer_url = state.relayer_url.lock().map_err(|e| e.to_string())?.clone();
    
    // If running, query Relayer for detailed status
    if is_running {
         // TODO: Add real HTTP call to Relayer here to get deeper stats
         // relying on monitoring::get_node_status for that detail
    }
    
    Ok(ProviderStatus {
        is_running,
        uptime: if is_running { 100 } else { 0 }, // Placeholder
        tasks_completed: 0,
        tasks_active: 0, 
        last_heartbeat: chrono::Utc::now(),
        earnings_today: 0,
        cpu_usage: 0.0,
        memory_usage: 0.0,
        gpu_usage: None,
        network_status: if is_running { "connected".to_string() } else { "disconnected".to_string() },
        relayer_connected: is_running, // Optimistic
        last_task_time: None,
        error_message: None,
    })
}

/// Register node by starting the provider daemon.
/// WebSocket registration to the relayer is handled automatically by the provider daemon
/// after it starts (using RELAYER_WS_URL, NODE_ID, and wallet credentials).
#[command]
pub async fn register_node(
    registration: NodeRegistration,
    state: State<'_, ProviderState>
) -> Result<String, String> {
    tracing::info!("Registering node: {:?}", registration);
    
    // Build provider configuration from registration data
    let config = ProviderConfig {
        wallet_address: registration.wallet_address.clone(),
        relayer_url: registration.relayer_endpoint
            .unwrap_or_else(|| "https://api.smainer.io".to_string()),
        port: 8080,
        max_tasks: 1,
        gpu_enabled: true,
        auto_start: true,
    };
    
    // Start the provider daemon - it will perform WebSocket registration automatically
    start_provider(config.clone(), state).await
        .map_err(|e| format!("Failed to start provider daemon: {}. Ensure the provider binary is installed.", e))?;
    
    // Return the derived node ID
    let node_id = node_id_from_address(&registration.wallet_address);
    tracing::info!("Provider daemon started successfully with node_id: {}", node_id);
    Ok(node_id)
}

#[command]
pub async fn check_registration_status(wallet_address: String) -> Result<String, String> {
    tracing::debug!("Checking registration status for wallet: {}...", &wallet_address[..std::cmp::min(6, wallet_address.len())]);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    let relayer_url = "https://api.smainer.io";
    
    // Derive node_id from wallet address
    let node_id = node_id_from_address(&wallet_address);
    let url = format!("{}/api/v1/nodes/{}", relayer_url, node_id);
    
    match client.get(&url).send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            if response.status().is_success() {
                tracing::debug!("Node {} found in relayer", node_id);
                Ok("registered".to_string())
            } else if status_code == 404 {
                tracing::debug!("Node {} not found in relayer (404)", node_id);
                Ok("not_registered".to_string()) // Not registered - this is a valid state
            } else if status_code == 401 {
                tracing::debug!("Authentication required for node lookup (401)");
                Ok("auth_required".to_string()) // Unauthenticated - cannot determine status
            } else {
                tracing::debug!("Unexpected status code {} for node lookup", status_code);
                Ok("unknown".to_string()) // Unknown state
            }
        }
        Err(e) => {
            tracing::debug!("Network error checking registration status: {}", e);
            Ok("network_error".to_string()) // Network error - cannot determine status
        }
    }
}

/// Get the AI capability configuration file path
fn get_ai_config_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push(".smainer");
    path.push("ai_config.json");
    path
}

/// Ensure the .smainer directory exists
fn ensure_config_directory() -> Result<(), String> {
    let mut path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push(".smainer");
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    Ok(())
}

#[command]
pub async fn save_ai_config(config: AICapabilityConfig) -> Result<bool, String> {
    tracing::info!("Saving AI capability config");
    
    ensure_config_directory()?;
    let config_path = get_ai_config_path();
    
    let mut updated_config = config;
    updated_config.updated_at = chrono::Utc::now();
    
    let json_content = serde_json::to_string_pretty(&updated_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&config_path, json_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    tracing::info!("AI capability config saved to {:?}", config_path);
    Ok(true)
}

#[command]
pub async fn load_ai_config() -> Result<AICapabilityConfig, String> {
    let config_path = get_ai_config_path();
    
    if !config_path.exists() {
        tracing::info!("AI config file not found, returning default config");
        return Ok(AICapabilityConfig::default());
    }
    
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let config: AICapabilityConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;
    
    tracing::info!("AI capability config loaded from {:?}", config_path);
    Ok(config)
}

#[command]
pub async fn validate_ai_capabilities(config: AICapabilityConfig) -> Result<AICapabilityReport, String> {
    tracing::info!("Validating AI capabilities");
    
    // Get current hardware info for validation
    let system_info = crate::commands::hardware::get_system_info().await
        .map_err(|e| format!("Failed to get hardware info: {}", e))?;
    let hardware = crate::models::HardwareInfo {
        cpu_name: system_info.cpu_name,
        cpu_cores: system_info.cpu_cores,
        total_ram: system_info.total_ram,
        available_ram: system_info.available_ram,
        gpus: system_info.gpus,
        os: system_info.os,
        os_version: system_info.os_version,
    };
    
    let mut system_validation = SystemValidation {
        meets_ai_requirements: true,
        ollama_available: false,
        models_validated: std::collections::HashMap::new(),
        warnings: vec![],
        errors: vec![],
    };
    
    // Check Ollama availability
    if config.ai_serving_enabled {
        if let Some(ollama_config) = &config.ollama_config {
            if ollama_config.install_requested {
                // Check if Ollama is installed and accessible
                system_validation.ollama_available = check_ollama_available(&ollama_config.api_endpoint).await;
                
                if !system_validation.ollama_available {
                    system_validation.errors.push(
                        "Ollama is required but not available. Please install Ollama or disable AI serving.".to_string()
                    );
                    system_validation.meets_ai_requirements = false;
                }
            }
        } else {
            system_validation.errors.push(
                "AI serving enabled but no Ollama configuration found.".to_string()
            );
            system_validation.meets_ai_requirements = false;
        }
    }
    
    // Validate each enabled model
    for model in &config.model_preferences {
        if model.enabled {
            let validation = validate_model_requirements(model, &hardware);
            
            if !validation.vram_sufficient || !validation.ram_sufficient {
                system_validation.meets_ai_requirements = false;
            }
            
            if !validation.vram_sufficient {
                system_validation.warnings.push(
                    format!("Model {} may run slowly: insufficient VRAM", model.name)
                );
            }
            
            if !validation.ram_sufficient {
                system_validation.warnings.push(
                    format!("Model {} may run slowly: insufficient RAM", model.name)
                );
            }
            
            system_validation.models_validated.insert(model.name.clone(), validation);
        }
    }
    
    let compatibility_status = if system_validation.meets_ai_requirements && system_validation.errors.is_empty() {
        if system_validation.warnings.is_empty() {
            CompatibilityStatus::Optimal
        } else {
            CompatibilityStatus::Acceptable
        }
    } else if system_validation.errors.len() < 2 {
        CompatibilityStatus::Limited
    } else {
        CompatibilityStatus::Incompatible
    };
    
    Ok(AICapabilityReport {
        config,
        system_validation,
        compatibility_status,
    })
}

async fn check_ollama_available(endpoint: &str) -> bool {
    let client = reqwest::Client::new();
    let url = format!("{}/api/version", endpoint);
    
    match client.get(&url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

fn validate_model_requirements(
    model: &crate::models::ModelConfig, 
    hardware: &crate::models::HardwareInfo
) -> ModelValidation {
    let ram_gb = hardware.total_ram / (1024 * 1024 * 1024);
    let best_gpu = hardware.gpus.iter()
        .filter(|gpu| gpu.is_supported)
        .max_by_key(|gpu| gpu.memory);
    
    let vram_gb = best_gpu.map(|gpu| gpu.memory / 1024).unwrap_or(0) as u32;
    
    let vram_sufficient = if model.requirements.requires_gpu {
        vram_gb >= model.requirements.min_vram_gb
    } else {
        true // GPU not required
    };
    
    let ram_sufficient = ram_gb as u32 >= model.requirements.min_ram_gb;
    let disk_sufficient = true; // Assume disk space is available for now
    
    let performance_tier = if vram_sufficient && ram_sufficient && vram_gb >= model.requirements.min_vram_gb + 2 {
        "optimal"
    } else if vram_sufficient && ram_sufficient {
        "acceptable" 
    } else if ram_sufficient {
        "limited"
    } else {
        "insufficient"
    };
    
    ModelValidation {
        available: true, // Assume model can be downloaded
        vram_sufficient,
        ram_sufficient,
        disk_sufficient,
        performance_tier: performance_tier.to_string(),
    }
}

/// BUG FIX: Install Ollama on Windows
#[command]
pub async fn install_ollama() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        tracing::info!("Starting Ollama installation for Windows...");
        
        // Download installer to temp directory
        let temp_dir = std::env::temp_dir();
        let installer_path = temp_dir.join("OllamaSetup.exe");
        
        tracing::info!("Downloading Ollama installer to {:?}...", installer_path);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 min timeout for download
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
            
        let response = client
            .get("https://ollama.com/download/OllamaSetup.exe")
            .send()
            .await
            .map_err(|e| format!("Failed to download Ollama installer: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to download Ollama installer: HTTP {}", response.status()));
        }
        
        let bytes = response.bytes().await.map_err(|e| format!("Failed to read installer bytes: {}", e))?;
        
        fs::write(&installer_path, bytes).map_err(|e| format!("Failed to save installer: {}", e))?;
        
        tracing::info!("Running Ollama installer silently...");
        
        // Run installer silently
        let output = std::process::Command::new(&installer_path)
            .arg("/S") // Silent install flag for NSIS installer
            .output()
            .map_err(|e| format!("Failed to run Ollama installer: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Ollama installation failed: {}", stderr));
        }
        
        // Clean up installer
        let _ = fs::remove_file(&installer_path);
        
        tracing::info!("Ollama installation completed successfully");
        Ok("Ollama installed successfully".to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Automatic Ollama installation is only supported on Windows. Please install Ollama manually from https://ollama.com/download".to_string())
    }
}
