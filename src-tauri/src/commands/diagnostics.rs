use serde::{Deserialize, Serialize};
use tauri::{command, State};
use anyhow::{Result, Context};
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use chrono::{DateTime, Utc};
use tokio::time::{timeout, Duration};
use crate::commands::provider::ProviderState;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsBundle {
    pub bundle_path: String,
    pub created_at: DateTime<Utc>,
    pub items_collected: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiagnosticsManifest {
    pub generated_at: DateTime<Utc>,
    pub node_id: String,
    pub wallet_address: String,
    pub relayer_url: String,
    pub provider_process_running: bool,
    pub provider_pid: Option<u32>,
    pub dns_probe: DiagnosticProbe,
    pub health_probe: DiagnosticProbe,
    pub websocket_probe: DiagnosticProbe,
    pub files_included: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiagnosticProbe {
    pub success: bool,
    pub result: String,
    pub error_type: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Redact private key - show only first 6 + last 4 chars
fn redact_private_key(key: &str) -> String {
    if key.len() > 10 {
        format!("{}...{}", &key[..6], &key[key.len()-4..])
    } else if key.len() > 6 {
        format!("{}...", &key[..6])
    } else {
        "<REDACTED>".to_string()
    }
}

/// Get app data directory where diagnostics bundle will be stored
fn get_diagnostics_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default())
    } else {
        dirs::home_dir().unwrap_or_default()
    };
    base.join(".smainer")
}

/// Get provider log file path
fn get_provider_log_path() -> PathBuf {
    get_diagnostics_dir().join("provider.log")
}

/// Get provider startup log path  
fn get_provider_startup_log_path() -> PathBuf {
    get_diagnostics_dir().join("provider-startup.log")
}

/// Read wallet config with private key redaction
fn read_wallet_config_redacted() -> Result<serde_json::Value> {
    let wallet_path = get_diagnostics_dir().join("wallet.json");
    if !wallet_path.exists() {
        return Ok(serde_json::json!({"error": "wallet.json not found"}));
    }
    
    let content = fs::read_to_string(&wallet_path)?;
    let mut wallet: serde_json::Value = serde_json::from_str(&content)?;
    
    // Redact private key
    if let Some(private_key) = wallet.get("private_key").and_then(|v| v.as_str()) {
        wallet["private_key"] = serde_json::Value::String(redact_private_key(private_key));
    }
    
    Ok(wallet)
}

/// Perform DNS resolution probe
async fn probe_dns_resolution() -> DiagnosticProbe {
    let start = std::time::Instant::now();
    
    match timeout(Duration::from_secs(5), tokio::net::lookup_host("api.smainer.io:443")).await {
        Ok(Ok(addresses)) => {
            let addrs: Vec<String> = addresses.map(|addr| addr.to_string()).collect();
            DiagnosticProbe {
                success: true,
                result: format!("Resolved to: {:?}", addrs),
                error_type: None,
                duration_ms: Some(start.elapsed().as_millis() as u64),
            }
        },
        Ok(Err(e)) => DiagnosticProbe {
            success: false,
            result: format!("DNS resolution failed: {}", e),
            error_type: Some("dns_error".to_string()),
            duration_ms: Some(start.elapsed().as_millis() as u64),
        },
        Err(_) => DiagnosticProbe {
            success: false,
            result: "DNS resolution timed out".to_string(),
            error_type: Some("timeout".to_string()),
            duration_ms: Some(5000),
        },
    }
}

/// Perform HTTP health check
async fn probe_health_endpoint() -> DiagnosticProbe {
    let start = std::time::Instant::now();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");
    
    match client.get("https://api.smainer.io/api/v1/health").send().await {
        Ok(response) => {
            let status = response.status();
            match response.text().await {
                Ok(body) => DiagnosticProbe {
                    success: status.is_success(),
                    result: format!("HTTP {} - {}", status, body),
                    error_type: if status.is_success() { None } else { Some("http_error".to_string()) },
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                },
                Err(e) => DiagnosticProbe {
                    success: false,
                    result: format!("HTTP {} - Failed to read body: {}", status, e),
                    error_type: Some("read_error".to_string()),
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                },
            }
        },
        Err(e) => {
            let error_type = if e.is_timeout() {
                "timeout"
            } else if e.is_connect() {
                "connect_error"
            } else if e.is_request() {
                "request_error"
            } else {
                "unknown_error"
            };
            
            DiagnosticProbe {
                success: false,
                result: format!("Health check failed: {}", e),
                error_type: Some(error_type.to_string()),
                duration_ms: Some(start.elapsed().as_millis() as u64),
            }
        },
    }
}

/// Derive node_id from wallet address (matches provider.rs logic)
fn node_id_from_address(addr: &str) -> String {
    let stripped = addr.trim_start_matches("0x");
    let id: String = stripped.chars().filter(|c| c.is_alphanumeric()).take(24).collect();
    if id.is_empty() { "default-node".to_string() } else { id }
}

/// Perform WebSocket connection test
async fn probe_websocket_connection(node_id: &str) -> DiagnosticProbe {
    let start = std::time::Instant::now();
    let ws_url = format!("wss://api.smainer.io/ws/node/{}", node_id);
    
    // For now, just test the URL format and DNS resolution
    // Full WebSocket test would require auth signature which needs private key
    DiagnosticProbe {
        success: false, // Always mark as failed since we can't do full test without private key
        result: format!("WebSocket URL: {} (auth test skipped for security)", ws_url),
        error_type: Some("auth_required".to_string()),
        duration_ms: Some(start.elapsed().as_millis() as u64),
    }
}

/// Get current provider process status
fn get_provider_process_status(state: &State<'_, ProviderState>) -> (bool, Option<u32>) {
    if let Ok(mut guard) = state.process.lock() {
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(None) => (true, Some(child.id())), // Still running
                Ok(Some(_)) => {
                    *guard = None; // Process exited
                    (false, None)
                }
                Err(_) => (false, None),
            }
        } else {
            (false, None)
        }
    } else {
        (false, None)
    }
}

/// Read wallet address from local config
fn get_wallet_address() -> String {
    let wallet_path = get_diagnostics_dir().join("wallet.json");
    if let Ok(content) = fs::read_to_string(&wallet_path) {
        if let Ok(wallet) = serde_json::from_str::<serde_json::Value>(&content) {
            return wallet.get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
        }
    }
    "unknown".to_string()
}

/// Copy file with error handling, return true if successful
fn copy_file_if_exists(src: &PathBuf, dest_dir: &PathBuf, filename: &str) -> bool {
    if !src.exists() {
        return false;
    }
    
    if let Err(e) = fs::copy(src, dest_dir.join(filename)) {
        tracing::warn!("Failed to copy {}: {}", filename, e);
        false
    } else {
        true
    }
}

/// Create diagnostics bundle with all relevant files and probes
#[command]
pub async fn export_diagnostics_bundle(state: State<'_, ProviderState>) -> Result<DiagnosticsBundle, String> {
    let timestamp = Utc::now();
    let bundle_name = format!("smainer-diagnostics-{}", timestamp.format("%Y%m%d-%H%M%S"));
    let diagnostics_dir = get_diagnostics_dir();
    let bundle_dir = diagnostics_dir.join(&bundle_name);
    
    // Create bundle directory
    fs::create_dir_all(&bundle_dir).map_err(|e| format!("Failed to create bundle directory: {}", e))?;
    
    let wallet_address = get_wallet_address();
    let node_id = node_id_from_address(&wallet_address);
    let relayer_url = state.relayer_url.lock().map_err(|e| e.to_string())?.clone();
    let (provider_running, provider_pid) = get_provider_process_status(&state);
    
    // Perform network probes
    let dns_probe = probe_dns_resolution().await;
    let health_probe = probe_health_endpoint().await;
    let ws_probe = probe_websocket_connection(&node_id).await;
    
    // Collect files
    let mut files_included = Vec::new();
    let mut items_collected = Vec::new();
    
    // Copy provider logs
    let provider_log = get_provider_log_path();
    if copy_file_if_exists(&provider_log, &bundle_dir, "provider.log") {
        files_included.push("provider.log".to_string());
        items_collected.push("Provider daemon log".to_string());
    }
    
    let startup_log = get_provider_startup_log_path();
    if copy_file_if_exists(&startup_log, &bundle_dir, "provider-startup.log") {
        files_included.push("provider-startup.log".to_string());
        items_collected.push("Provider startup log".to_string());
    }
    
    // Create redacted wallet config
    match read_wallet_config_redacted() {
        Ok(redacted_wallet) => {
            let wallet_file = bundle_dir.join("wallet-redacted.json");
            if let Err(e) = fs::write(&wallet_file, serde_json::to_string_pretty(&redacted_wallet).unwrap()) {
                tracing::warn!("Failed to write redacted wallet config: {}", e);
            } else {
                files_included.push("wallet-redacted.json".to_string());
                items_collected.push("Wallet config (redacted)".to_string());
            }
        },
        Err(e) => {
            tracing::warn!("Failed to read wallet config: {}", e);
        }
    }
    
    // Create diagnostics manifest
    let manifest = DiagnosticsManifest {
        generated_at: timestamp,
        node_id: node_id.clone(),
        wallet_address: wallet_address.clone(),
        relayer_url: relayer_url.clone(),
        provider_process_running: provider_running,
        provider_pid,
        dns_probe,
        health_probe,
        websocket_probe: ws_probe,
        files_included: files_included.clone(),
    };
    
    let manifest_file = bundle_dir.join("diagnostics-manifest.json");
    fs::write(&manifest_file, serde_json::to_string_pretty(&manifest).unwrap())
        .map_err(|e| format!("Failed to write manifest: {}", e))?;
    files_included.push("diagnostics-manifest.json".to_string());
    items_collected.push("Network probes and system info".to_string());
    
    // Create simple tar.gz archive
    let bundle_archive = diagnostics_dir.join(format!("{}.tar.gz", bundle_name));
    let tar_result = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&bundle_archive)
        .arg("-C")
        .arg(&diagnostics_dir)
        .arg(&bundle_name)
        .output();
    
    match tar_result {
        Ok(output) if output.status.success() => {
            // Clean up temp directory
            let _ = fs::remove_dir_all(&bundle_dir);
            
            Ok(DiagnosticsBundle {
                bundle_path: bundle_archive.to_string_lossy().to_string(),
                created_at: timestamp,
                items_collected,
            })
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("tar command failed: {}", stderr))
        },
        Err(e) => {
            Err(format!("Failed to execute tar command: {}. Files available in: {}", e, bundle_dir.display()))
        }
    }
}

/// Get the last created diagnostics bundle path for testing
#[command]
pub async fn get_last_diagnostics_bundle() -> Result<Option<String>, String> {
    let diagnostics_dir = get_diagnostics_dir();
    
    let mut bundles = Vec::new();
    if let Ok(entries) = fs::read_dir(&diagnostics_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("smainer-diagnostics-") && name.ends_with(".tar.gz") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            bundles.push((name.to_string(), modified));
                        }
                    }
                }
            }
        }
    }
    
    // Sort by modification time, newest first
    bundles.sort_by(|a, b| b.1.cmp(&a.1));
    
    Ok(bundles.first().map(|(name, _)| diagnostics_dir.join(name).to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_redact_private_key() {
        // Test normal key redaction
        let key = "0x1234567890abcdef1234567890abcdef12345678";
        let redacted = redact_private_key(key);
        assert_eq!(redacted, "0x1234...5678");
        
        // Test short key
        let short_key = "0x123456";
        let redacted_short = redact_private_key(short_key);
        assert_eq!(redacted_short, "0x1234...");
        
        // Test very short key
        let very_short = "0x123";
        let redacted_very_short = redact_private_key(very_short);
        assert_eq!(redacted_very_short, "<REDACTED>");
    }
    
    #[test]
    fn test_node_id_from_address() {
        let address = "0x071cd50ddd9a2d0e1e95e6decd9f0a292b489dc6b9b13e68aac43b2295b626d6";
        let node_id = node_id_from_address(address);
        assert_eq!(node_id, "071cd50ddd9a2d0e1e95e6de");
        assert_eq!(node_id.len(), 24);
        
        // Test empty address
        let empty_id = node_id_from_address("");
        assert_eq!(empty_id, "default-node");
        
        // Test short address
        let short_id = node_id_from_address("0x123");
        assert_eq!(short_id, "123");
    }
    
    #[tokio::test]
    async fn test_probe_dns_resolution() {
        let probe = probe_dns_resolution().await;
        // DNS probe should either succeed or fail with specific error types
        assert!(probe.duration_ms.is_some());
        if !probe.success {
            assert!(probe.error_type.is_some());
            assert!(probe.error_type.as_ref().unwrap() == "dns_error" || 
                   probe.error_type.as_ref().unwrap() == "timeout");
        }
    }
    
    #[tokio::test]
    async fn test_probe_health_endpoint() {
        let probe = probe_health_endpoint().await;
        // Health probe should either succeed or fail with specific error types
        assert!(probe.duration_ms.is_some());
        if !probe.success {
            assert!(probe.error_type.is_some());
            let error_type = probe.error_type.as_ref().unwrap();
            assert!(["timeout", "connect_error", "request_error", "http_error", "read_error", "unknown_error"]
                   .contains(&error_type.as_str()));
        }
    }
    
    #[tokio::test]
    async fn test_probe_websocket_connection() {
        let node_id = "test-node-id-123456789012";
        let probe = probe_websocket_connection(node_id).await;
        
        // WebSocket probe should always report failure due to auth requirements
        assert!(!probe.success);
        assert_eq!(probe.error_type.as_ref().unwrap(), "auth_required");
        assert!(probe.result.contains("wss://api.smainer.io/ws/node/"));
        assert!(probe.result.contains(node_id));
    }
    
    #[test]
    fn test_diagnostics_manifest_serialization() {
        let manifest = DiagnosticsManifest {
            generated_at: Utc::now(),
            node_id: "test-node".to_string(),
            wallet_address: "0x123".to_string(),
            relayer_url: "https://api.smainer.io".to_string(),
            provider_process_running: true,
            provider_pid: Some(1234),
            dns_probe: DiagnosticProbe {
                success: true,
                result: "DNS resolved".to_string(),
                error_type: None,
                duration_ms: Some(100),
            },
            health_probe: DiagnosticProbe {
                success: false,
                result: "Connection timeout".to_string(),
                error_type: Some("timeout".to_string()),
                duration_ms: Some(5000),
            },
            websocket_probe: DiagnosticProbe {
                success: false,
                result: "Auth required".to_string(),
                error_type: Some("auth_required".to_string()),
                duration_ms: Some(10),
            },
            files_included: vec!["provider.log".to_string(), "wallet-redacted.json".to_string()],
        };
        
        // Test that manifest can be serialized to JSON
        let json_result = serde_json::to_string_pretty(&manifest);
        assert!(json_result.is_ok());
        
        let json_str = json_result.unwrap();
        assert!(json_str.contains("test-node"));
        assert!(json_str.contains("provider.log"));
        assert!(json_str.contains("auth_required"));
    }
}