use serde::{Deserialize, Serialize};
use tauri::command;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::models::{NodeStatus, EarningsData, TaskHistoryEntry};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceHistory {
    pub timestamps: Vec<DateTime<Utc>>,
    pub cpu_usage: Vec<f64>,
    pub memory_usage: Vec<f64>,
    pub gpu_usage: Vec<Option<f64>>,
    pub earnings: Vec<u64>,
}

#[command]
pub async fn get_node_status() -> Result<NodeStatus, String> {
    tracing::info!("Getting node status...");
    
    // Mock node status for development
    Ok(NodeStatus {
        is_online: true,
        node_id: "node_12345".to_string(),
        uptime: 7200, // 2 hours
        last_heartbeat: Utc::now(),
        tasks_active: 3,
        tasks_completed_today: 15,
        earnings_today: 250, // $2.50 in cents
        cpu_usage: 35.7,
        memory_usage: 52.3,
        gpu_usage: Some(82.1),
        network_status: "healthy".to_string(),
        relayer_connected: true,
        provider_version: "0.1.0".to_string(),
        node_tier: "standard".to_string(),
    })
}

#[command]
pub async fn get_earnings() -> Result<EarningsData, String> {
    tracing::info!("Getting earnings data...");
    
    // Mock earnings data for development
    let mut daily_earnings = HashMap::new();
    let mut monthly_earnings = HashMap::new();
    
    // Generate mock daily earnings for the past 30 days
    for i in 0..30 {
        let date = (Utc::now() - chrono::Duration::days(i)).format("%Y-%m-%d").to_string();
        let earnings = (50 + (i * 5) + rand::random::<u64>() % 100) as u64; // Mock earnings
        daily_earnings.insert(date, earnings);
    }
    
    // Generate mock monthly earnings for the past 12 months
    for i in 0..12 {
        let date = (Utc::now() - chrono::Duration::days(i * 30)).format("%Y-%m").to_string();
        let earnings = (1500 + (i * 200) + rand::random::<u64>() % 500) as u64; // Mock earnings
        monthly_earnings.insert(date, earnings);
    }
    
    Ok(EarningsData {
        total_earnings: 12750, // $127.50 in cents
        today_earnings: 250,   // $2.50 in cents
        yesterday_earnings: 180, // $1.80 in cents
        this_week_earnings: 1450, // $14.50 in cents
        this_month_earnings: 4200, // $42.00 in cents
        daily_earnings,
        monthly_earnings,
        pending_rewards: 50,   // $0.50 in cents
        last_payout: Some(Utc::now() - chrono::Duration::days(1)),
        next_payout: Some(Utc::now() + chrono::Duration::days(6)),
    })
}

#[command]
pub async fn get_task_history(limit: Option<u32>) -> Result<Vec<TaskHistoryEntry>, String> {
    tracing::info!("Getting task history...");
    
    let limit = limit.unwrap_or(50);
    let mut tasks = Vec::new();
    
    // Generate mock task history
    for i in 0..limit {
        let task = TaskHistoryEntry {
            task_id: format!("task_{:06}", i),
            task_type: match i % 4 {
                0 => "image_generation".to_string(),
                1 => "text_processing".to_string(),
                2 => "model_training".to_string(),
                _ => "data_analysis".to_string(),
            },
            status: if i < 3 {
                "running".to_string()
            } else if i % 10 == 0 {
                "failed".to_string()
            } else {
                "completed".to_string()
            },
            submitted_at: Utc::now() - chrono::Duration::minutes(i as i64 * 15),
            started_at: Some(Utc::now() - chrono::Duration::minutes(i as i64 * 15 - 2)),
            completed_at: if i >= 3 && i % 10 != 0 {
                Some(Utc::now() - chrono::Duration::minutes(i as i64 * 15 - 10))
            } else {
                None
            },
            duration: if i >= 3 && i % 10 != 0 {
                Some(8 * 60) // 8 minutes in seconds
            } else {
                None
            },
            reward: if i >= 3 && i % 10 != 0 {
                Some(15 + (i % 20) as u64) // $0.15-0.35 range
            } else {
                None
            },
            client_id: format!("client_{}", i % 5),
            gpu_used: i % 3 != 0,
            error_message: if i % 10 == 0 {
                Some("Task timeout exceeded".to_string())
            } else {
                None
            },
        };
        tasks.push(task);
    }
    
    Ok(tasks)
}

#[command]
pub async fn get_node_metrics() -> Result<NodeMetrics, String> {
    tracing::info!("Getting node metrics...");
    
    // Mock real-time metrics
    Ok(NodeMetrics {
        cpu_usage: 45.2 + (rand::random::<f64>() - 0.5) * 10.0,
        memory_usage: 62.8 + (rand::random::<f64>() - 0.5) * 5.0,
        gpu_usage: Some(78.5 + (rand::random::<f64>() - 0.5) * 15.0),
        network_latency: Some(25.0 + (rand::random::<f64>() - 0.5) * 10.0),
        uptime: 14400, // 4 hours
        tasks_completed: 47,
        tasks_failed: 2,
        earnings_total: 12750, // $127.50
        last_updated: Utc::now(),
    })
}

#[command]
pub async fn get_performance_history(hours: Option<u32>) -> Result<PerformanceHistory, String> {
    tracing::info!("Getting performance history for {} hours", hours.unwrap_or(24));
    
    let hours = hours.unwrap_or(24);
    let mut history = PerformanceHistory {
        timestamps: Vec::new(),
        cpu_usage: Vec::new(),
        memory_usage: Vec::new(),
        gpu_usage: Vec::new(),
        earnings: Vec::new(),
    };
    
    // Generate mock historical data points (every 10 minutes)
    let points = (hours * 6) as i64;
    for i in 0..points {
        let timestamp = Utc::now() - chrono::Duration::minutes(i * 10);
        history.timestamps.push(timestamp);
        
        // Mock trending data with some randomness
        let base_cpu = 40.0 + (i as f64 / points as f64) * 20.0;
        let base_memory = 55.0 + (i as f64 / points as f64) * 10.0;
        let base_gpu = 70.0 + (i as f64 / points as f64) * 15.0;
        
        history.cpu_usage.push(base_cpu + (rand::random::<f64>() - 0.5) * 10.0);
        history.memory_usage.push(base_memory + (rand::random::<f64>() - 0.5) * 5.0);
        history.gpu_usage.push(Some(base_gpu + (rand::random::<f64>() - 0.5) * 15.0));
        
        // Cumulative earnings (monotonically increasing)
        let earnings = (i as u64 * 3) + rand::random::<u64>() % 5;
        history.earnings.push(earnings);
    }
    
    // Reverse to get chronological order
    history.timestamps.reverse();
    history.cpu_usage.reverse();
    history.memory_usage.reverse();
    history.gpu_usage.reverse();
    history.earnings.reverse();
    
    Ok(history)
}

#[command]
pub async fn refresh_node_data() -> Result<bool, String> {
    tracing::info!("Refreshing node data...");
    
    // Simulate data refresh delay
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // In real implementation, this would:
    // 1. Query the provider daemon for fresh stats
    // 2. Connect to relayer for updated task information
    // 3. Refresh earnings data from blockchain/relayer
    // 4. Update local cache
    
    Ok(true)
}