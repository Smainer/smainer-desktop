use crate::commands::provider::{recent_provider_error_summary, ProviderState};
use crate::models::{EarningsData, NodeStatus, TaskHistoryEntry};
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{command, State};

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
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    let stored: StoredWallet = serde_json::from_str(&content).ok()?;
    Some(stored.address)
}

fn node_id_from_address(addr: &str) -> String {
    let stripped = addr.trim_start_matches("0x");
    let id: String = stripped
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(24)
        .collect();
    if id.is_empty() {
        "default-node".to_string()
    } else {
        id
    }
}

fn parse_utc_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        })
}

fn empty_earnings() -> EarningsData {
    EarningsData {
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
    }
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
    let expected_node_id = node_id_from_address(&wallet_addr);
    let relayer_url = state.relayer_url.lock().map_err(|e| e.to_string())?.clone();
    let provider_error = if is_running {
        None
    } else {
        recent_provider_error_summary()
    };

    // Compute uptime from process start time (tracked locally, no relayer needed)
    let uptime_secs: u64 = {
        if let Ok(guard) = state.start_time.lock() {
            guard.as_ref().map(|t| t.elapsed().as_secs()).unwrap_or(0)
        } else {
            0
        }
    };

    // Default offline status
    let mut status = NodeStatus {
        is_online: false, // Only set true when relayer confirms
        node_id: expected_node_id.clone(),
        uptime: uptime_secs,
        last_heartbeat: Utc::now(),
        tasks_active: 0,
        tasks_completed_today: 0,
        earnings_today: 0,
        cpu_usage: 0.0,
        memory_usage: 0.0,
        gpu_usage: None,
        network_status: if is_running {
            "connecting".to_string()
        } else if provider_error.is_some() {
            "provider_failed".to_string()
        } else {
            "disconnected".to_string()
        },
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
        let relayer_healthy =
            matches!(client.get(&health_url).send().await, Ok(r) if r.status().is_success());

        if relayer_healthy {
            // Correct relayer endpoint: /api/v1/nodes/{node_id}
            let url = format!("{}/api/v1/nodes/{}", relayer_url, expected_node_id);

            match client.get(&url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        status.is_online = true;
                        status.relayer_connected = true;
                        status.network_status = "connected".to_string();

                        if let Ok(node_info) = resp.json::<serde_json::Value>().await {
                            status.node_id = node_info
                                .get("node_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&expected_node_id)
                                .to_string();
                            status.tasks_active = node_info
                                .get("current_tasks")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            status.node_tier = node_info
                                .get("calculated_tier")
                                .and_then(|v| v.as_str())
                                .unwrap_or("standard")
                                .to_string();
                            if let Some(last_heartbeat) = node_info
                                .get("last_heartbeat")
                                .and_then(|v| v.as_str())
                                .and_then(parse_utc_datetime)
                            {
                                status.last_heartbeat = last_heartbeat;
                            }
                        }

                        let tasks_url = format!("{}/api/v1/nodes/{}/tasks?limit=200", relayer_url, expected_node_id);
                        if let Ok(tasks_resp) = client.get(&tasks_url).send().await {
                            if let Ok(tasks) = tasks_resp.json::<Vec<TaskHistoryEntry>>().await {
                                let today = Utc::now().date_naive();
                                status.tasks_completed_today = tasks
                                    .iter()
                                    .filter(|task| {
                                        task.status == "completed"
                                            && task
                                                .completed_at
                                                .map(|completed| completed.date_naive() == today)
                                                .unwrap_or(false)
                                    })
                                    .count() as u32;
                            }
                        }

                        let earnings_url = format!("{}/api/v1/nodes/{}/earnings", relayer_url, expected_node_id);
                        if let Ok(earnings_resp) = client.get(&earnings_url).send().await {
                            if let Ok(earnings) = earnings_resp.json::<EarningsData>().await {
                                status.earnings_today = earnings.today_earnings;
                            }
                        }
                    } else if resp.status().as_u16() == 404 {
                        status.is_online = true;
                        status.relayer_connected = false;
                        status.network_status = "registration_failed".to_string();
                        status.node_tier = "unregistered".to_string();
                    } else {
                        status.is_online = true;
                        status.relayer_connected = false;
                        status.network_status = format!("relayer_error_{}", resp.status().as_u16());
                    }
                }
                Err(_) => {
                    // Process running, relayer healthy, but node lookup failed
                    status.is_online = true; // Process is running
                    status.relayer_connected = false;
                    // Fix 5: Better timeout error message
                    status.network_status =
                        "Provider running — unable to connect to relayer (check network)"
                            .to_string();
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
pub async fn get_earnings(state: State<'_, ProviderState>) -> Result<EarningsData, String> {
    let wallet_addr = match get_wallet_address_local() {
        Some(address) => address,
        None => return Ok(empty_earnings()),
    };
    let relayer_url = state.relayer_url.lock().map_err(|e| e.to_string())?.clone();
    let node_id = node_id_from_address(&wallet_addr);
    let url = format!("{}/api/v1/nodes/{}/earnings", relayer_url, node_id);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => resp
            .json::<EarningsData>()
            .await
            .map_err(|e| e.to_string()),
        _ => Ok(empty_earnings()),
    }
}

#[command]
pub async fn get_task_history(
    state: State<'_, ProviderState>,
    limit: Option<u32>,
) -> Result<Vec<TaskHistoryEntry>, String> {
    let wallet_addr = match get_wallet_address_local() {
        Some(address) => address,
        None => return Ok(Vec::new()),
    };
    let relayer_url = state.relayer_url.lock().map_err(|e| e.to_string())?.clone();
    let node_id = node_id_from_address(&wallet_addr);
    let bounded_limit = limit.unwrap_or(50).clamp(1, 200);
    let url = format!("{}/api/v1/nodes/{}/tasks?limit={}", relayer_url, node_id, bounded_limit);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => resp
            .json::<Vec<TaskHistoryEntry>>()
            .await
            .map_err(|e| e.to_string()),
        _ => Ok(Vec::new()),
    }
}
