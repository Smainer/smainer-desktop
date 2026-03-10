use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::models::HardwareInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub is_online: bool,
    pub node_id: String,
    pub uptime: u64, // seconds
    pub last_heartbeat: DateTime<Utc>,
    pub tasks_active: u32,
    pub tasks_completed_today: u32,
    pub earnings_today: u64, // in cents
    pub cpu_usage: f64, // percentage
    pub memory_usage: f64, // percentage  
    pub gpu_usage: Option<f64>, // percentage
    pub network_status: String,
    pub relayer_connected: bool,
    pub provider_version: String,
    pub node_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub is_running: bool,
    pub uptime: u64, // seconds
    pub tasks_completed: u32,
    pub tasks_active: u32,
    pub last_heartbeat: DateTime<Utc>,
    pub earnings_today: u64, // in cents
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub gpu_usage: Option<f64>,
    pub network_status: String,
    pub relayer_connected: bool,
    pub last_task_time: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsData {
    pub total_earnings: u64, // in cents
    pub today_earnings: u64,
    pub yesterday_earnings: u64,
    pub this_week_earnings: u64,
    pub this_month_earnings: u64,
    pub daily_earnings: HashMap<String, u64>, // date -> earnings in cents
    pub monthly_earnings: HashMap<String, u64>, // month -> earnings in cents
    pub pending_rewards: u64, // in cents
    pub last_payout: Option<DateTime<Utc>>,
    pub next_payout: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryEntry {
    pub task_id: String,
    pub task_type: String,
    pub status: String, // "pending", "running", "completed", "failed"
    pub submitted_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration: Option<u64>, // seconds
    pub reward: Option<u64>, // in cents
    pub client_id: String,
    pub gpu_used: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegistration {
    pub wallet_address: String,
    pub hardware_capabilities: HardwareInfo,
    pub stake_amount: Option<u64>,
    pub relayer_endpoint: Option<String>,
    pub node_name: Option<String>,
    pub contact_info: Option<String>,
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self {
            is_online: false,
            node_id: String::new(),
            uptime: 0,
            last_heartbeat: Utc::now(),
            tasks_active: 0,
            tasks_completed_today: 0,
            earnings_today: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            gpu_usage: None,
            network_status: "disconnected".to_string(),
            relayer_connected: false,
            provider_version: "0.1.0".to_string(),
            node_tier: "standard".to_string(),
        }
    }
}

impl EarningsData {
    pub fn total_earnings_dollars(&self) -> f64 {
        self.total_earnings as f64 / 100.0
    }
    
    pub fn today_earnings_dollars(&self) -> f64 {
        self.today_earnings as f64 / 100.0
    }
    
    pub fn pending_rewards_dollars(&self) -> f64 {
        self.pending_rewards as f64 / 100.0
    }
}

impl TaskHistoryEntry {
    pub fn is_completed(&self) -> bool {
        self.status == "completed"
    }
    
    pub fn is_running(&self) -> bool {
        self.status == "running"
    }
    
    pub fn is_failed(&self) -> bool {
        self.status == "failed"
    }
    
    pub fn reward_dollars(&self) -> Option<f64> {
        self.reward.map(|r| r as f64 / 100.0)
    }
    
    pub fn duration_minutes(&self) -> Option<f64> {
        self.duration.map(|d| d as f64 / 60.0)
    }
}