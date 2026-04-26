use serde::{Deserialize, Serialize};
use tauri::{command, State};
use anyhow::Result;
use std::process::{Command, Stdio, Child};
use std::sync::Mutex;
use crate::models::{ProviderStatus, NodeRegistration};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub wallet_address: String,
    pub hardware_info: String,
    pub stake_amount: Option<u64>,
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

    // 1. Bundled sidecar next to the app exe (production install path)
    let sidecar_name = if cfg!(target_os = "windows") { "smainer-provider.exe" } else { "smainer-provider" };
    let sidecar_path = exe_dir.join(sidecar_name);

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
    } else if sidecar_path.exists() {
        // Production: use bundled sidecar binary
        Command::new(&sidecar_path)
    } else {
        // No sidecar and no environment override - return clear error
        return Err("This development build does not include the provider daemon. To start a node, use the Windows installer build or connect a local provider via SMAINER_PROVIDER_CMD environment variable.".to_string());
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

#[command]
pub async fn register_node(registration: NodeRegistration) -> Result<String, String> {
    tracing::info!("Registering node: {:?}", registration);
    
    let client = reqwest::Client::new();
    
    let registration_request = RegistrationRequest {
        wallet_address: registration.wallet_address.clone(),
        hardware_info: serde_json::to_string(&registration.hardware_capabilities)
            .map_err(|e| format!("Failed to serialize hardware info: {}", e))?,
        stake_amount: registration.stake_amount,
    };
    
    let relayer_url = registration.relayer_endpoint
        .unwrap_or_else(|| "https://api.smainer.io".to_string());
        
    let url = format!("{}/register", relayer_url); // Endpoint assumption
    
    let response = client.post(&url)
        .json(&registration_request)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to relayer: {}", e))?;
        
    if response.status().is_success() {
        Ok("success".to_string())
    } else {
        Err(format!("Registration failed: {}", response.status()))
    }
}
