use serde::{Deserialize, Serialize};
use tauri::command;
use anyhow::Result;
use std::process::{Command, Stdio};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use crate::models::{ProviderStatus, NodeRegistration};

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
    pub hardware_info: String, // JSON string of hardware capabilities
    pub stake_amount: Option<u64>,
}

#[command]
pub async fn start_provider(config: ProviderConfig) -> Result<bool, String> {
    tracing::info!("Starting provider with config: {:?}", config);
    
    // For now, simulate provider startup
    // In real implementation, this would launch the actual provider process
    
    // Validate configuration
    if config.wallet_address.is_empty() {
        return Err("Wallet address required".to_string());
    }
    
    if config.relayer_url.is_empty() {
        return Err("Relayer URL required".to_string());
    }
    
    // Simulate startup delay
    sleep(Duration::from_millis(1000)).await;
    
    // Mock success for development
    Ok(true)
}

#[command]
pub async fn stop_provider() -> Result<bool, String> {
    tracing::info!("Stopping provider...");
    
    // Simulate shutdown delay
    sleep(Duration::from_millis(500)).await;
    
    // Mock success for development
    Ok(true)
}

#[command]
pub async fn get_provider_status() -> Result<ProviderStatus, String> {
    tracing::info!("Getting provider status...");
    
    // Mock provider status for development
    Ok(ProviderStatus {
        is_running: true,
        uptime: 3600, // 1 hour
        tasks_completed: 42,
        tasks_active: 2,
        last_heartbeat: chrono::Utc::now(),
        earnings_today: 150, // $1.50 in cents
        cpu_usage: 25.5,
        memory_usage: 45.2,
        gpu_usage: Some(78.9),
        network_status: "connected".to_string(),
        relayer_connected: true,
        last_task_time: Some(chrono::Utc::now() - chrono::Duration::minutes(5)),
        error_message: None,
    })
}

#[command]
pub async fn register_node(registration: NodeRegistration) -> Result<String, String> {
    tracing::info!("Registering node: {:?}", registration);
    
    // Create registration request to local relayer
    let client = reqwest::Client::new();
    
    let registration_request = RegistrationRequest {
        wallet_address: registration.wallet_address.clone(),
        hardware_info: serde_json::to_string(&registration.hardware_capabilities)
            .map_err(|e| format!("Failed to serialize hardware info: {}", e))?,
        stake_amount: registration.stake_amount,
    };
    
    // Try to register with the relayer at localhost:8000
    let relayer_url = registration.relayer_endpoint
        .unwrap_or_else(|| "http://localhost:8000".to_string());
    
    let response = client
        .post(&format!("{}/api/v1/providers/register", relayer_url))
        .json(&registration_request)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let node_id = resp.text().await
                    .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
                tracing::info!("Node registered successfully with ID: {}", node_id);
                Ok(node_id)
            } else {
                let error_msg = resp.text().await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!("Registration failed: {}", error_msg))
            }
        }
        Err(e) => {
            tracing::warn!("Failed to connect to relayer, using mock registration: {}", e);
            // Return mock node ID for development when relayer is not running
            let mock_node_id = uuid::Uuid::new_v4().to_string();
            Ok(mock_node_id)
        }
    }
}

#[command]
pub async fn get_provider_logs() -> Result<Vec<String>, String> {
    tracing::info!("Getting provider logs...");
    
    // Mock logs for development
    Ok(vec![
        "2024-03-10 10:00:00 [INFO] Provider started successfully".to_string(),
        "2024-03-10 10:01:23 [INFO] Connected to relayer at localhost:8000".to_string(),
        "2024-03-10 10:02:45 [INFO] Received task assignment: image-generation-001".to_string(),
        "2024-03-10 10:05:12 [INFO] Task completed successfully, reward: $0.05".to_string(),
        "2024-03-10 10:07:33 [INFO] GPU utilization: 85%".to_string(),
        "2024-03-10 10:10:01 [INFO] Heartbeat sent to relayer".to_string(),
    ])
}

#[command] 
pub async fn update_provider_config(config: ProviderConfig) -> Result<bool, String> {
    tracing::info!("Updating provider config: {:?}", config);
    
    // Validate new configuration
    if config.port < 1024 || config.port > 65535 {
        return Err("Port must be between 1024 and 65535".to_string());
    }
    
    if config.max_tasks == 0 {
        return Err("Max tasks must be greater than 0".to_string());
    }
    
    // Save configuration (in real implementation, would persist to file)
    // For now, just return success
    Ok(true)
}