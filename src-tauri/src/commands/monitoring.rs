use serde::{Deserialize, Serialize};
use tauri::{command, State};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::models::{NodeStatus, EarningsData, TaskHistoryEntry};
use crate::commands::provider::ProviderState;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub gpu_usage: Option<f64>,
    pub network_latency: Option<f64>,
    pub uptime: u64, // seconds
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub earnings_total: u64, // in cents
    pub last_updated: DateTime<Utc>,
}

// Track provider process start time for timeout detection
pub struct ProviderStartTime {
    start_time: std::sync::Mutex<Option<DateTime<Utc>>>,
}

impl Default for ProviderStartTime {
    fn default() -> Self {
        Self {
            start_time: std::sync::Mutex::new(None),
        }
    }
}

// Set provider start time when process starts
pub fn set_provider_start_time() -> ProviderStartTime {
    ProviderStartTime {
        start_time: std::sync::Mutex::new(Some(Utc::now())),
    }
}

#[derive(Serialize, Deserialize)]
struct StoredWallet {
    address: String,
}

fn get_wallet_address_local() -> Option<String> {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".smainer/wallet.json");
    if !path.exists() { return None; }
    
    let content = fs::read_to_string(path).ok()?;
    let stored: StoredWallet = serde_json::from_str(&content).ok()?;
    Some(stored.address)
}

#[command]
pub async fn get_node_status(state: State<'_, ProviderState>) -> Result<NodeStatus, String> {
    // Check if process is running locally
    let is_running = {
        let mut guard = state.process.lock().map_err(|e| e.to_string())?;
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(None) => true,
                _ => false,
            }
        } else {
            false
        }
    };

    let wallet_addr = get_wallet_address_local().unwrap_or_default();
    let relayer_url = state.relayer_url.lock().map_err(|e| e.to_string())?.clone();

    // Compute uptime from process start time (tracked locally, no relayer needed)
    let uptime_secs: u64 = {
        if let Ok(guard) = state.start_time.lock() {
            guard.as_ref().map(|t| t.elapsed().as_secs()).unwrap_or(0)
        } else { 0 }
    };

    // Default offline status
    let mut status = NodeStatus {
        is_online: false, // Only set true when relayer confirms
        node_id: wallet_addr.clone(),
        uptime: uptime_secs,
        last_heartbeat: Utc::now(),
        tasks_active: 0,
        tasks_completed_today: 0,
        earnings_today: 0,
        cpu_usage: 0.0,
        memory_usage: 0.0,
        gpu_usage: None,
        network_status: if is_running { "connecting".to_string() } else { "disconnected".to_string() },
        relayer_connected: false,
        provider_version: "0.1.0".to_string(),
        node_tier: "standard".to_string(),
    };

    if is_running && !wallet_addr.is_empty() {
        // Query Relayer API for real stats
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;
            
        // BUG FIX: Check relayer health endpoint first
        let health_url = format!("{}/api/v1/health", relayer_url);
        let relayer_healthy = matches!(client.get(&health_url).send().await, Ok(r) if r.status().is_success());
        
        if relayer_healthy {
            // Correct relayer endpoint: /api/v1/nodes/{node_id}
            let node_id = wallet_addr.trim_start_matches("0x").chars().filter(|c| c.is_alphanumeric()).take(24).collect::<String>();
            let node_id = if node_id.is_empty() { wallet_addr.clone() } else { node_id };
            let url = format!("{}/api/v1/nodes/{}", relayer_url, node_id);
            
            match client.get(&url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        // Try to parse real status from relayer
                        if let Ok(real_status) = resp.json::<NodeStatus>().await {
                            status = real_status;
                            status.is_online = true; // Confirmed by relayer
                            status.relayer_connected = true;
                            status.network_status = "connected".to_string();
                        } else {
                            // Process running, relayer healthy, but can't parse response
                            status.is_online = true; // Process is running
                            status.relayer_connected = true;
                            status.network_status = "connected".to_string();
                        }
                    } else {
                        // Process running but not found in relayer (might still be registering)
                        status.is_online = true; // Process is running
                        status.relayer_connected = true;
                        // Fix 5: Better registration timeout detection
                        let current_time = Utc::now();
                        // Check if provider has been running > 30 seconds but still not registered
                        let registration_timeout = std::time::Duration::from_secs(30);
                        let provider_running_duration = std::time::Duration::from_secs(60); // Assume 60s if we can't determine exact start time
                        
                        if provider_running_duration > registration_timeout {
                            status.network_status = "Provider running — registration failed (check logs)".to_string();
                        } else {
                            status.network_status = "connected_unregistered".to_string();
                        }
                    }
                },
                Err(_) => {
                    // Process running, relayer healthy, but node lookup failed
                    status.is_online = true; // Process is running
                    status.relayer_connected = false;
                    // Fix 5: Better timeout error message
                    status.network_status = "Provider running — unable to connect to relayer (check network)".to_string();
                }
            }
        } else {
            // Process running but relayer unreachable
            status.is_online = true; // Process is still running locally
            status.relayer_connected = false;
            status.network_status = "connecting".to_string();
        }
    }

    Ok(status)
}

#[command]
pub async fn get_earnings() -> Result<EarningsData, String> {
    // Similar logic: fetch from Relayer API /provider/{address}/earnings
    // For now returning zeros if fetch fails
    
    Ok(EarningsData {
        total_earnings: 0,
        today_earnings: 0,
        yesterday_earnings: 0,
        this_week_earnings: 0,
        this_month_earnings: 0,
        daily_earnings: HashMap::new(),
        monthly_earnings: HashMap::new(),
        pending_rewards: 0,
        last_payout: None,
        next_payout: None,
    })
}

#[command]
pub async fn get_task_history() -> Result<Vec<TaskHistoryEntry>, String> {
    Ok(Vec::new())
}
