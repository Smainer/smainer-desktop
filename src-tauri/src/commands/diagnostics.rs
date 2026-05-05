use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{command, State};
use tokio::time::{timeout, Duration};
use url::Url;

use crate::commands::provider::{load_ai_config, ProviderState};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsBundle {
    pub bundle_path: String,
    pub created_at: DateTime<Utc>,
    pub items_collected: Vec<String>,
    pub summary: DiagnosticsSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsSummary {
    pub provider_running: bool,
    pub relayer_health_ok: bool,
    pub ollama_api_ok: bool,
    pub ai_enabled: bool,
    pub node_id: String,
    pub relayer_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiagnosticProbe {
    success: bool,
    result: String,
    error_type: Option<String>,
    duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SidecarResolution {
    exe_dir: String,
    candidates: Vec<String>,
    found_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiagnosticsManifest {
    generated_at: DateTime<Utc>,
    app: Value,
    node: Value,
    relayer: Value,
    provider: Value,
    ollama: Value,
    capability_env_summary: Value,
    config_summary: Value,
    recent_errors: Vec<String>,
    files_included: Vec<String>,
}

#[derive(Debug, Clone)]
struct DiagnosticsInputPaths {
    diagnostics_dir: PathBuf,
    provider_log_path: Option<PathBuf>,
    provider_startup_log_path: Option<PathBuf>,
    wallet_path: Option<PathBuf>,
    ai_config_path: Option<PathBuf>,
}

fn candidate_data_dirs() -> Vec<PathBuf> {
    if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default());
        let mut dirs = vec![appdata.join("smainer")];
        dirs.push(dirs::home_dir().unwrap_or_default().join(".smainer"));
        dirs
    } else {
        vec![dirs::home_dir().unwrap_or_default().join(".smainer")]
    }
}

fn get_primary_data_dir() -> PathBuf {
    let dirs = candidate_data_dirs();
    if let Some(existing) = dirs.iter().find(|dir| dir.exists()) {
        return existing.clone();
    }
    dirs.first().cloned().unwrap_or_else(|| PathBuf::from("."))
}

fn ensure_path_within(base: &Path, candidate: &Path) -> Result<()> {
    if candidate.starts_with(base) {
        return Ok(());
    }
    anyhow::bail!(
        "Resolved path {} escapes base {}",
        candidate.display(),
        base.display()
    );
}

fn get_safe_diagnostics_output_dir() -> Result<PathBuf> {
    let base = get_primary_data_dir();
    fs::create_dir_all(&base).with_context(|| format!("create {}", base.display()))?;
    let base_canonical =
        fs::canonicalize(&base).with_context(|| format!("canonicalize {}", base.display()))?;

    let diagnostics_dir = base_canonical.join("diagnostics");
    fs::create_dir_all(&diagnostics_dir)
        .with_context(|| format!("create {}", diagnostics_dir.display()))?;
    let diagnostics_canonical = fs::canonicalize(&diagnostics_dir)
        .with_context(|| format!("canonicalize {}", diagnostics_dir.display()))?;
    ensure_path_within(&base_canonical, &diagnostics_canonical)?;

    Ok(diagnostics_canonical)
}

fn get_provider_log_path() -> Option<PathBuf> {
    candidate_data_dirs()
        .into_iter()
        .map(|d| d.join("provider.log"))
        .find(|p| p.exists())
}

fn get_provider_startup_log_path() -> Option<PathBuf> {
    candidate_data_dirs()
        .into_iter()
        .map(|d| d.join("provider-startup.log"))
        .find(|p| p.exists())
}

fn get_wallet_path() -> Option<PathBuf> {
    candidate_data_dirs()
        .into_iter()
        .map(|d| d.join("wallet.json"))
        .find(|p| p.exists())
}

fn get_ai_config_path() -> Option<PathBuf> {
    candidate_data_dirs()
        .into_iter()
        .map(|d| d.join("ai_config.json"))
        .find(|p| p.exists())
}

fn resolve_default_diagnostics_inputs() -> Result<DiagnosticsInputPaths> {
    Ok(DiagnosticsInputPaths {
        diagnostics_dir: get_safe_diagnostics_output_dir()?,
        provider_log_path: get_provider_log_path(),
        provider_startup_log_path: get_provider_startup_log_path(),
        wallet_path: get_wallet_path(),
        ai_config_path: get_ai_config_path(),
    })
}

fn redact_by_key_name(key: &str, value: &Value) -> Value {
    let lower = key.to_lowercase();
    if lower.contains("private")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("bearer")
        || lower.contains("auth")
        || lower.contains("rpc")
    {
        return Value::String("<REDACTED>".to_string());
    }

    if lower.contains("address") {
        if let Some(s) = value.as_str() {
            return Value::String(mask_wallet_address(s));
        }
    }

    value.clone()
}

fn redact_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if v.is_object() || v.is_array() {
                    out.insert(k.clone(), redact_json_value(v));
                } else {
                    let by_key = redact_by_key_name(k, v);
                    if let Value::String(s) = by_key {
                        out.insert(k.clone(), Value::String(redact_sensitive_text(&s)));
                    } else {
                        out.insert(k.clone(), by_key);
                    }
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_json_value).collect()),
        Value::String(s) => Value::String(redact_sensitive_text(s)),
        _ => value.clone(),
    }
}

fn mask_wallet_address(address: &str) -> String {
    if address.len() <= 14 {
        return "<REDACTED>".to_string();
    }
    format!("{}...{}", &address[..8], &address[address.len() - 6..])
}

fn redact_hex_values(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 2 < chars.len() && chars[i] == '0' && chars[i + 1] == 'x' {
            let start = i + 2;
            let mut end = start;
            while end < chars.len() && chars[end].is_ascii_hexdigit() {
                end += 1;
            }
            let hex_len = end.saturating_sub(start);
            if hex_len >= 16 {
                let hex: String = chars[start..end].iter().collect();
                out.push_str("0x");
                out.push_str(&format!("{}...{}", &hex[..6], &hex[hex.len() - 4..]));
                i = end;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn redact_long_token_word(word: &str) -> String {
    let clean = word.trim_matches(|c: char| c == '"' || c == '\'' || c == ';' || c == ',');
    let looks_opaque = clean.len() >= 24
        && clean
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');
    if looks_opaque {
        "<REDACTED_TOKEN>".to_string()
    } else {
        word.to_string()
    }
}

fn redact_sensitive_assignments(line: &str) -> String {
    let lower = line.to_lowercase();
    for key in [
        "private_key",
        "apikey",
        "api_key",
        "token",
        "secret",
        "password",
        "bearer",
        "authorization",
        "rpc",
        "rpc_url",
    ] {
        if let Some(pos) = lower.find(key) {
            let prefix = &line[..pos];
            let rest = &line[pos..];
            if let Some(eq) = rest.find('=') {
                return format!("{}{}=<REDACTED>", prefix, &rest[..eq]);
            }
            if let Some(colon) = rest.find(':') {
                return format!("{}{}:<REDACTED>", prefix, &rest[..colon]);
            }
        }
    }

    if lower.contains("bearer ") {
        let parts: Vec<String> = line
            .split_whitespace()
            .map(|w| {
                if w.to_lowercase().starts_with("bearer") {
                    "Bearer <REDACTED>".to_string()
                } else {
                    redact_long_token_word(w)
                }
            })
            .collect();
        return parts.join(" ");
    }

    line.split_whitespace()
        .map(redact_long_token_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_url(url: &str) -> String {
    let mut parsed = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return redact_sensitive_assignments(url),
    };

    if !parsed.username().is_empty() {
        let _ = parsed.set_username("redacted");
    }
    if parsed.password().is_some() {
        let _ = parsed.set_password(Some("redacted"));
    }

    let mut sanitized_pairs = Vec::new();
    for (k, v) in parsed.query_pairs() {
        let key_lower = k.to_lowercase();
        let value = if key_lower.contains("token")
            || key_lower.contains("secret")
            || key_lower.contains("key")
            || key_lower.contains("auth")
            || key_lower.contains("password")
            || key_lower.contains("bearer")
        {
            "redacted".to_string()
        } else {
            redact_sensitive_assignments(&v)
        };
        sanitized_pairs.push((k.to_string(), value));
    }

    if !sanitized_pairs.is_empty() {
        {
            let mut qp = parsed.query_pairs_mut();
            qp.clear();
            for (k, v) in sanitized_pairs {
                qp.append_pair(&k, &v);
            }
        }
    }

    parsed.to_string()
}

fn redact_url_credentials(input: &str) -> String {
    input
        .split_whitespace()
        .map(|token| {
            let trimmed = token.trim_matches(|c: char| {
                c == '"' || c == '\'' || c == ',' || c == ';' || c == '(' || c == ')' || c == '[' || c == ']'
            });
            if trimmed.contains("://") {
                token.replace(trimmed, &sanitize_url(trimmed))
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_sensitive_text(input: &str) -> String {
    let with_hex = redact_hex_values(input);
    let with_urls = redact_url_credentials(&with_hex);
    redact_sensitive_assignments(&with_urls)
}

fn redact_log_text(content: &str) -> String {
    content
        .lines()
        .map(redact_sensitive_text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_redacted_log_tail(path: &Path, lines: usize) -> String {
    let content = fs::read_to_string(path).unwrap_or_default();
    let line_vec: Vec<&str> = content.lines().collect();
    let start = line_vec.len().saturating_sub(lines);
    redact_log_text(&line_vec[start..].join("\n"))
}

fn collect_recent_errors(redacted_log: &str, max_lines: usize) -> Vec<String> {
    redacted_log
        .lines()
        .filter(|line| {
            let l = line.to_lowercase();
            l.contains("error") || l.contains("failed") || l.contains("panic") || l.contains("traceback")
        })
        .rev()
        .take(max_lines)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn resolve_sidecar_paths() -> SidecarResolution {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    let candidates = if cfg!(target_os = "windows") {
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

    let found_path = candidates
        .iter()
        .find(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string());

    SidecarResolution {
        exe_dir: exe_dir.to_string_lossy().to_string(),
        candidates: candidates
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        found_path,
    }
}

fn node_id_from_address(addr: &str) -> String {
    let stripped = addr.trim_start_matches("0x");
    let id: String = stripped
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(24)
        .collect();
    if id.is_empty() {
        "default-node".to_string()
    } else {
        id
    }
}

fn relayer_health_url(base: &str) -> String {
    format!("{}/api/v1/health", base.trim_end_matches('/'))
}

fn http_to_ws_url(url: &str) -> String {
    if url.starts_with("https://") {
        format!("wss://{}", &url[8..])
    } else if url.starts_with("http://") {
        format!("ws://{}", &url[7..])
    } else {
        url.to_string()
    }
}

async fn probe_relayer_health(relayer_url: &str) -> DiagnosticProbe {
    let start = std::time::Instant::now();
    let health_url = relayer_health_url(relayer_url);
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(8)).build() {
        Ok(c) => c,
        Err(e) => {
            return DiagnosticProbe {
                success: false,
                result: format!("Failed to build HTTP client: {}", e),
                error_type: Some("client_error".to_string()),
                duration_ms: None,
            }
        }
    };

    match timeout(Duration::from_secs(10), client.get(&health_url).send()).await {
        Ok(Ok(resp)) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            DiagnosticProbe {
                success: status.is_success(),
                result: format!("{} {}", status, redact_sensitive_text(&body)),
                error_type: if status.is_success() {
                    None
                } else {
                    Some("http_status".to_string())
                },
                duration_ms: Some(start.elapsed().as_millis() as u64),
            }
        }
        Ok(Err(e)) => DiagnosticProbe {
            success: false,
            result: format!("Request failed: {}", redact_sensitive_text(&e.to_string())),
            error_type: Some("request_error".to_string()),
            duration_ms: Some(start.elapsed().as_millis() as u64),
        },
        Err(_) => DiagnosticProbe {
            success: false,
            result: "Health probe timed out".to_string(),
            error_type: Some("timeout".to_string()),
            duration_ms: Some(10_000),
        },
    }
}

async fn probe_ollama_api(endpoint: &str) -> DiagnosticProbe {
    let start = std::time::Instant::now();
    let url = format!("{}/api/version", endpoint.trim_end_matches('/'));
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(5)).build() {
        Ok(c) => c,
        Err(e) => {
            return DiagnosticProbe {
                success: false,
                result: format!("Failed to build HTTP client: {}", e),
                error_type: Some("client_error".to_string()),
                duration_ms: None,
            }
        }
    };

    match timeout(Duration::from_secs(6), client.get(&url).send()).await {
        Ok(Ok(resp)) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            DiagnosticProbe {
                success: status.is_success(),
                result: format!("{} {}", status, redact_sensitive_text(&body)),
                error_type: if status.is_success() {
                    None
                } else {
                    Some("http_status".to_string())
                },
                duration_ms: Some(start.elapsed().as_millis() as u64),
            }
        }
        Ok(Err(e)) => DiagnosticProbe {
            success: false,
            result: format!("Request failed: {}", redact_sensitive_text(&e.to_string())),
            error_type: Some("request_error".to_string()),
            duration_ms: Some(start.elapsed().as_millis() as u64),
        },
        Err(_) => DiagnosticProbe {
            success: false,
            result: "Ollama probe timed out".to_string(),
            error_type: Some("timeout".to_string()),
            duration_ms: Some(6_000),
        },
    }
}

fn detect_ollama_executable() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        if std::process::Command::new("where")
            .arg("ollama")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some("ollama".to_string());
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    {
        if std::process::Command::new("which")
            .arg("ollama")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            Some("ollama".to_string())
        } else {
            None
        }
    }
}

fn get_provider_process_status(state: &ProviderState) -> (bool, Option<u32>) {
    if let Ok(mut guard) = state.process.lock() {
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(None) => (true, Some(child.id())),
                Ok(Some(_)) => {
                    *guard = None;
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

fn write_redacted_copy(src: &Path, dst: &Path) -> Result<()> {
    let content = fs::read_to_string(src).with_context(|| format!("read {}", src.display()))?;
    fs::write(dst, redact_log_text(&content)).with_context(|| format!("write {}", dst.display()))?;
    Ok(())
}

#[command]
pub async fn export_diagnostics_bundle(state: State<'_, ProviderState>) -> Result<DiagnosticsBundle, String> {
    let inputs = resolve_default_diagnostics_inputs()
        .map_err(|e| format!("Failed to resolve diagnostics input paths: {}", e))?;
    export_diagnostics_bundle_with_inputs(&state, inputs, true).await
}

async fn export_diagnostics_bundle_with_inputs(
    state: &ProviderState,
    inputs: DiagnosticsInputPaths,
    archive_bundle: bool,
) -> Result<DiagnosticsBundle, String> {
    let timestamp = Utc::now();
    let diagnostics_dir = inputs.diagnostics_dir;

    let bundle_name = format!("smainer-diagnostics-{}", timestamp.format("%Y%m%d-%H%M%S"));
    let bundle_dir = diagnostics_dir.join(&bundle_name);
    fs::create_dir_all(&bundle_dir).map_err(|e| format!("Failed to create bundle dir: {}", e))?;
    let bundle_dir_canonical = fs::canonicalize(&bundle_dir)
        .map_err(|e| format!("Failed to resolve bundle dir path: {}", e))?;
    ensure_path_within(&diagnostics_dir, &bundle_dir_canonical)
        .map_err(|e| format!("Unsafe bundle output path: {}", e))?;

    let relayer_url = state.relayer_url.lock().map_err(|e| e.to_string())?.clone();
    let relayer_url_sanitized = sanitize_url(&relayer_url);
    let (provider_running, provider_pid) = get_provider_process_status(state);
    let sidecar = resolve_sidecar_paths();
    let relayer_health = probe_relayer_health(&relayer_url).await;

    let ai_config = load_ai_config().await.unwrap_or_default();
    let ai_enabled = ai_config.ai_serving_enabled;
    let ollama_endpoint = ai_config
        .ollama_config
        .as_ref()
        .map(|c| c.api_endpoint.clone())
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let ollama_endpoint_sanitized = sanitize_url(&ollama_endpoint);
    let ollama_api = probe_ollama_api(&ollama_endpoint).await;
    let ollama_executable = detect_ollama_executable();

    let wallet_json = match &inputs.wallet_path {
        Some(path) => match fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        {
            Some(v) => redact_json_value(&v),
            None => json!({"error": "wallet.json unreadable"}),
        },
        None => json!({"error": "wallet.json not found"}),
    };

    let wallet_address = wallet_json
        .get("address")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let node_id = node_id_from_address(&wallet_address);

    let provider_log_tail = inputs
        .provider_log_path
        .as_ref()
        .map(|p| read_redacted_log_tail(p, 140))
        .unwrap_or_default();
    let startup_log_tail = inputs
        .provider_startup_log_path
        .as_ref()
        .map(|p| read_redacted_log_tail(p, 120))
        .unwrap_or_default();

    let combined_for_errors = format!("{}\n{}", provider_log_tail, startup_log_tail);
    let recent_errors = collect_recent_errors(&combined_for_errors, 40);

    let mut files_included = Vec::new();
    let mut items_collected = vec![
        "Desktop runtime summary".to_string(),
        "Relayer health probe".to_string(),
        "Provider sidecar/path resolution".to_string(),
        "Ollama executable and API probe".to_string(),
        "Capability and environment summary".to_string(),
        "Recent errors and log tails".to_string(),
    ];

    if let Some(log_path) = &inputs.provider_log_path {
        let out = bundle_dir.join("provider.log.redacted");
        if write_redacted_copy(log_path, &out).is_ok() {
            files_included.push("provider.log.redacted".to_string());
        }
    }

    if let Some(startup_path) = &inputs.provider_startup_log_path {
        let out = bundle_dir.join("provider-startup.log.redacted");
        if write_redacted_copy(startup_path, &out).is_ok() {
            files_included.push("provider-startup.log.redacted".to_string());
        }
    }

    if let Some(ai_path) = &inputs.ai_config_path {
        let out = bundle_dir.join("ai_config.redacted.json");
        let ai_json = fs::read_to_string(&ai_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .map(|v| redact_json_value(&v))
            .unwrap_or_else(|| json!({"error": "ai_config unreadable"}));
        if fs::write(&out, serde_json::to_string_pretty(&ai_json).unwrap_or_else(|_| "{}".to_string())).is_ok() {
            files_included.push("ai_config.redacted.json".to_string());
        }
    }

    let wallet_file = bundle_dir.join("wallet.redacted.json");
    if fs::write(
        &wallet_file,
        serde_json::to_string_pretty(&wallet_json).unwrap_or_else(|_| "{}".to_string()),
    )
    .is_ok()
    {
        files_included.push("wallet.redacted.json".to_string());
    }

    let config_summary = json!({
        "wallet_present": wallet_json.get("error").is_none(),
        "wallet_address": mask_wallet_address(&wallet_address),
        "node_id": node_id,
        "relayer_url": relayer_url_sanitized,
        "relayer_ws_url": sanitize_url(&http_to_ws_url(&relayer_url)),
        "provider_log_path": inputs.provider_log_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "provider_startup_log_path": inputs.provider_startup_log_path.as_ref().map(|p| p.to_string_lossy().to_string()),
    });

    let capability_env_summary = json!({
        "CAPABILITY_AI_ENABLED": ai_enabled,
        "CAPABILITY_OLLAMA_ENABLED": ollama_api.success,
        "CAPABILITY_SUPPORTED_MODELS": ai_config
            .model_preferences
            .iter()
            .filter(|m| m.enabled)
            .map(|m| m.name.clone())
            .collect::<Vec<_>>(),
        "CAPABILITY_PRIVACY_MODE": format!("{:?}", ai_config.privacy_mode),
        "CAPABILITY_CONTRACT_VERSION": "1.0.0",
    });

    let manifest = DiagnosticsManifest {
        generated_at: timestamp,
        app: json!({
            "product_name": "Smainer",
            "desktop_version": env!("CARGO_PKG_VERSION"),
            "platform": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "target_family": std::env::consts::FAMILY,
        }),
        node: json!({
            "node_id": node_id,
            "wallet_address": mask_wallet_address(&wallet_address),
            "provider_running": provider_running,
            "provider_pid": provider_pid,
        }),
        relayer: json!({
            "configured_url": relayer_url_sanitized,
            "health_url": sanitize_url(&relayer_health_url(&relayer_url)),
            "health_probe": relayer_health,
        }),
        provider: json!({
            "sidecar_resolution": sidecar,
            "provider_log_tail": provider_log_tail,
            "startup_log_tail": startup_log_tail,
        }),
        ollama: json!({
            "executable_detected": ollama_executable.is_some(),
            "executable": ollama_executable,
            "api_endpoint": ollama_endpoint_sanitized,
            "api_probe": ollama_api,
        }),
        capability_env_summary,
        config_summary,
        recent_errors,
        files_included: files_included.clone(),
    };

    let manifest_value = serde_json::to_value(&manifest).unwrap_or_else(|_| json!({}));
    let manifest_redacted = redact_json_value(&manifest_value);

    let manifest_path = bundle_dir.join("diagnostics-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest_redacted).unwrap_or_else(|_| "{}".to_string()),
    )
    .map_err(|e| format!("Failed to write diagnostics manifest: {}", e))?;
    files_included.push("diagnostics-manifest.json".to_string());

    let notes_path = bundle_dir.join("README.txt");
    let mut notes = fs::File::create(&notes_path)
        .map_err(|e| format!("Failed to create diagnostics README: {}", e))?;
    let _ = writeln!(notes, "Smainer Diagnostics Bundle");
    let _ = writeln!(notes, "Generated at: {}", timestamp.to_rfc3339());
    let _ = writeln!(notes, "This bundle has redacted sensitive tokens and key material.");
    let _ = writeln!(notes, "Included files:");
    for f in &files_included {
        let _ = writeln!(notes, "- {}", f);
    }
    files_included.push("README.txt".to_string());

    let bundle_path = if archive_bundle {
        let archive_path = diagnostics_dir.join(format!("{}.tar.gz", bundle_name));
        let tar_result = std::process::Command::new("tar")
            .arg("-czf")
            .arg(&archive_path)
            .arg("-C")
            .arg(&diagnostics_dir)
            .arg(&bundle_name)
            .output();

        match tar_result {
            Ok(output) if output.status.success() => {
                let _ = fs::remove_dir_all(&bundle_dir);
                archive_path.to_string_lossy().to_string()
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                items_collected.push(format!("Archive creation failed: {}", stderr.trim()));
                bundle_dir.to_string_lossy().to_string()
            }
            Err(_) => {
                items_collected.push("Archive command unavailable; directory bundle returned".to_string());
                bundle_dir.to_string_lossy().to_string()
            }
        }
    } else {
        items_collected.push("Archive step skipped for deterministic test validation".to_string());
        bundle_dir.to_string_lossy().to_string()
    };

    Ok(DiagnosticsBundle {
        bundle_path,
        created_at: timestamp,
        items_collected,
        summary: DiagnosticsSummary {
            provider_running,
            relayer_health_ok: manifest_redacted
                .get("relayer")
                .and_then(|v| v.get("health_probe"))
                .and_then(|v| v.get("success"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            ollama_api_ok: manifest_redacted
                .get("ollama")
                .and_then(|v| v.get("api_probe"))
                .and_then(|v| v.get("success"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            ai_enabled,
            node_id: manifest_redacted
                .get("node")
                .and_then(|v| v.get("node_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            relayer_url: manifest_redacted
                .get("relayer")
                .and_then(|v| v.get("configured_url"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        },
    })
}

#[command]
pub async fn get_last_diagnostics_bundle() -> Result<Option<String>, String> {
    let diagnostics_dir = get_safe_diagnostics_output_dir()
        .map_err(|e| format!("Failed to resolve diagnostics dir: {}", e))?;
    let mut bundles: Vec<(String, std::time::SystemTime)> = Vec::new();

    if let Ok(entries) = fs::read_dir(&diagnostics_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("smainer-diagnostics-") && name.ends_with(".tar.gz") {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            bundles.push((name.to_string(), modified));
                        }
                    }
                }
            }
        }
    }

    bundles.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(bundles
        .first()
        .map(|(name, _)| diagnostics_dir.join(name).to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn redacts_private_key_and_bearer_tokens() {
        let line = "Authorization: Bearer abcdefghijklmnopqrstuvwxyz0123456789";
        let redacted = redact_sensitive_text(line);
        assert!(!redacted.contains("abcdefghijklmnopqrstuvwxyz"));
        assert!(redacted.contains("<REDACTED>"));
    }

    #[test]
    fn redacts_hex_values() {
        let line = "private_key=0x1234567890abcdef1234567890abcdef12345678";
        let redacted = redact_sensitive_text(line);
        assert!(redacted.contains("<REDACTED>"));
        assert!(!redacted.contains("1234567890abcdef1234567890abcdef12345678"));
    }

    #[test]
    fn redacts_sensitive_json_fields() {
        let value = json!({
            "private_key": "0xabcdef1234567890",
            "rpc_url": "https://user:pass@example.com",
            "address": "0x0123456789abcdef0123456789abcdef",
            "nested": {
                "token": "ABCDEFGHIJKLMNOPQRSTUVWX123456"
            }
        });
        let redacted = redact_json_value(&value);
        assert_eq!(redacted["private_key"], "<REDACTED>");
        assert_eq!(redacted["rpc_url"], "<REDACTED>");
        assert_eq!(redacted["nested"]["token"], "<REDACTED>");
        assert!(redacted["address"].as_str().unwrap_or("<missing>").contains("..."));
    }

    #[test]
    fn collects_error_lines() {
        let log = "info ok\nERROR relayer offline\nwarn\nfailed provider launch\n";
        let errors = collect_recent_errors(log, 10);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR"));
        assert!(errors[1].contains("failed"));
    }

    #[test]
    fn redacts_urls_with_inline_credentials_and_secret_query_values() {
        let line = "https://user:pass@example.com/api?token=abcd1234abcd1234abcd1234&mode=ok";
        let redacted = redact_sensitive_text(line);
        assert!(!redacted.contains("user:pass"));
        assert!(!redacted.contains("abcd1234abcd1234abcd1234"));
        assert!(redacted.contains("redacted"));
    }

    #[test]
    fn redacts_rpc_assignment_values() {
        let line = "rpc=https://user:pass@example.com";
        let redacted = redact_sensitive_text(line);
        assert_eq!(redacted, "rpc=<REDACTED>");
    }

    #[test]
    fn sanitizes_summary_urls() {
        let sanitized =
            sanitize_url("https://alice:s3cr3t@example.com/path?api_key=xyzxyzxyzxyzxyzxyzxyzxyz");
        assert!(!sanitized.contains("alice"));
        assert!(!sanitized.contains("s3cr3t"));
        assert!(!sanitized.contains("xyzxyzxyzxyzxyzxyzxyzxyz"));
        assert!(sanitized.contains("redacted"));
    }

    #[tokio::test]
    async fn exports_bundle_and_redacts_sensitive_artifacts() {
        let temp = tempdir().expect("tempdir");
        let diagnostics_dir = temp.path().join("diagnostics");
        fs::create_dir_all(&diagnostics_dir).expect("create diagnostics dir");

        let provider_log = temp.path().join("provider.log");
        fs::write(
            &provider_log,
            "INFO startup\nAuthorization: Bearer abcdefghijklmnopqrstuvwxyz0123456789\n",
        )
        .expect("write provider log");

        let startup_log = temp.path().join("provider-startup.log");
        fs::write(
            &startup_log,
            "rpc=https://alice:secret@example.com\nprivate_key=0xabcdef1234567890abcdef1234567890abcdef12\n",
        )
        .expect("write startup log");

        let wallet_path = temp.path().join("wallet.json");
        fs::write(
            &wallet_path,
            r#"{"address":"0x0123456789abcdef0123456789abcdef","private_key":"0xfeedfacecafebeef"}"#,
        )
        .expect("write wallet");

        let ai_path = temp.path().join("ai_config.json");
        fs::write(
            &ai_path,
            r#"{"ollama_config":{"api_endpoint":"https://user:pw@host.local/api?token=toktoktoktoktoktoktoktok"},"model_preferences":[],"secret":"SENSITIVEVALUE123456789012345"}"#,
        )
        .expect("write ai config");

        let state = ProviderState::default();
        {
            let mut relayer_url = state.relayer_url.lock().expect("lock relayer_url");
            *relayer_url =
                "https://bob:password@relayer.example.com?auth=supersecrettoken1234567890".to_string();
        }

        let inputs = DiagnosticsInputPaths {
            diagnostics_dir: diagnostics_dir.clone(),
            provider_log_path: Some(provider_log.clone()),
            provider_startup_log_path: Some(startup_log.clone()),
            wallet_path: Some(wallet_path.clone()),
            ai_config_path: Some(ai_path.clone()),
        };

        let bundle = export_diagnostics_bundle_with_inputs(&state, inputs, false)
            .await
            .expect("export diagnostics bundle");

        let bundle_dir = PathBuf::from(&bundle.bundle_path);
        assert!(bundle_dir.is_dir());

        let manifest = fs::read_to_string(bundle_dir.join("diagnostics-manifest.json"))
            .expect("read manifest");
        let redacted_provider_log = fs::read_to_string(bundle_dir.join("provider.log.redacted"))
            .expect("read provider redacted log");
        let redacted_startup_log =
            fs::read_to_string(bundle_dir.join("provider-startup.log.redacted"))
                .expect("read startup redacted log");
        let redacted_wallet = fs::read_to_string(bundle_dir.join("wallet.redacted.json"))
            .expect("read wallet redacted");
        let redacted_ai = fs::read_to_string(bundle_dir.join("ai_config.redacted.json"))
            .expect("read ai redacted");

        let artifacts = vec![
            ("manifest", manifest),
            ("provider.log.redacted", redacted_provider_log),
            ("provider-startup.log.redacted", redacted_startup_log),
            ("wallet.redacted.json", redacted_wallet),
            ("ai_config.redacted.json", redacted_ai),
        ];

        let raw_secrets = [
            "abcdefghijklmnopqrstuvwxyz0123456789",
            "alice:secret",
            "0xabcdef1234567890abcdef1234567890abcdef12",
            "0xfeedfacecafebeef",
            "SENSITIVEVALUE123456789012345",
            "supersecrettoken1234567890",
            "bob:password",
            "toktoktoktoktoktoktoktok",
        ];

        for (artifact_name, artifact_content) in &artifacts {
            for (secret_index, raw_secret) in raw_secrets.iter().enumerate() {
                assert!(
                    !artifact_content.contains(raw_secret),
                    "raw secret leaked into artifact {} at secret index {}",
                    artifact_name,
                    secret_index
                );
            }
        }

        let combined = artifacts
            .iter()
            .map(|(_, content)| content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(combined.contains("<REDACTED>") || combined.contains("redacted"));
    }
}
