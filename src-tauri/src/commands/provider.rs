use serde::{Deserialize, Serialize};
use tauri::{command, State};
use anyhow::Result;
use std::process::{Command, Stdio, Child};
use std::sync::Mutex;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
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
    
    // Try to find smainer-provider binary first (bundled)
    let binary_path = if cfg!(target_os = "windows") { "smainer-provider.exe" } else { "smainer-provider" };
    
    // Command construction
    let mut cmd = if std::path::Path::new(binary_path).exists() {
        Command::new(binary_path)
    } else {
        // Dev fallback: locate backend/provider relative to cwd or exe location
        let cwd = std::env::current_dir().unwrap_or_default();
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();

        // Candidates: walk up from cwd and exe_dir looking for backend/provider
        let candidates: Vec<std::path::PathBuf> = vec![
            cwd.join("backend").join("provider"),
            cwd.parent().map(|p| p.join("backend").join("provider")).unwrap_or_default(),
            cwd.parent().and_then(|p| p.parent()).map(|p| p.join("backend").join("provider")).unwrap_or_default(),
            exe_dir.join("backend").join("provider"),
            exe_dir.parent().map(|p| p.join("backend").join("provider")).unwrap_or_default(),
            exe_dir.parent().and_then(|p| p.parent()).map(|p| p.join("backend").join("provider")).unwrap_or_default(),
            exe_dir.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.join("backend").join("provider")).unwrap_or_default(),
        ];

        let backend_path = candidates
            .into_iter()
            .find(|p| p.exists() && p.is_dir())
            .ok_or_else(|| "Provider backend not found. Download the full Smainer package from https://github.com/Smainer/smainer-desktop/releases".to_string())?;

        let python_cmd = if cfg!(target_os = "windows") { "python" } else { "python3" };
        let mut py_cmd = Command::new(python_cmd);
        py_cmd.current_dir(&backend_path);
        py_cmd.arg("-m");
        py_cmd.arg("provider.main");
        py_cmd
    };
    
    // Pass args
    cmd.env("STARKNET_ACCOUNT_ADDRESS", &config.wallet_address);
    cmd.env("RELAYER_WS_URL", &config.relayer_url);
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
    
    let relayer_url = state.relayer_url.lock().map_err(|e| e.to_string())?.clone();
    
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
