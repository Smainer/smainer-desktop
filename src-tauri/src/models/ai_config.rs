use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Schema version for compatibility tracking with provider daemon
pub const CAPABILITY_CONFIG_SCHEMA_VERSION: &str = "1.0.0";
pub const CONTRACT_VERSION: &str = "2024.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICapabilityConfig {
    pub schema_version: String,
    pub contract_version: String,
    pub ai_serving_enabled: bool,
    pub ollama_config: Option<OllamaConfig>,
    pub model_preferences: Vec<ModelConfig>,
    pub privacy_mode: PrivacyMode,
    pub resources: ResourceRequirements,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub install_requested: bool,
    pub installation_path: Option<String>,
    pub api_endpoint: String, // Default: http://localhost:11434
    pub models_to_install: Vec<String>,
    pub auto_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub enabled: bool,
    pub priority: u8, // 1-10, higher = more preferred
    pub requirements: ModelRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub min_vram_gb: u32,
    pub min_ram_gb: u32,
    pub min_disk_gb: u32,
    pub requires_gpu: bool,
    pub network_bandwidth_mbps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyMode {
    Standard,    // Normal operation
    Enhanced,    // No data logging, minimal telemetry  
    Maximum,     // Local processing only, no external calls
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub max_cpu_percent: u8,
    pub max_ram_gb: u32,
    pub max_vram_gb: Option<u32>,
    pub max_disk_io_mbps: Option<u32>,
    pub max_network_mbps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICapabilityReport {
    pub config: AICapabilityConfig,
    pub system_validation: SystemValidation,
    pub compatibility_status: CompatibilityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemValidation {
    pub meets_ai_requirements: bool,
    pub ollama_available: bool,
    pub models_validated: HashMap<String, ModelValidation>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelValidation {
    pub available: bool,
    pub vram_sufficient: bool,
    pub ram_sufficient: bool,
    pub disk_sufficient: bool,
    pub performance_tier: String, // "optimal", "acceptable", "limited"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityStatus {
    Optimal,      // All requirements met
    Acceptable,   // Minor limitations  
    Limited,      // Major limitations
    Incompatible, // Cannot run AI tasks
}

impl Default for AICapabilityConfig {
    fn default() -> Self {
        Self {
            schema_version: CAPABILITY_CONFIG_SCHEMA_VERSION.to_string(),
            contract_version: CONTRACT_VERSION.to_string(),
            ai_serving_enabled: false, // Explicit opt-in default
            ollama_config: None,
            model_preferences: vec![],
            privacy_mode: PrivacyMode::Standard,
            resources: ResourceRequirements {
                max_cpu_percent: 80,
                max_ram_gb: 8,
                max_vram_gb: None,
                max_disk_io_mbps: None,
                max_network_mbps: None,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            install_requested: false,
            installation_path: None,
            api_endpoint: "http://localhost:11434".to_string(),
            models_to_install: vec!["llama3.1:8b".to_string()],
            auto_update: false,
        }
    }
}

// Common model configurations with validated requirements
impl ModelConfig {
    pub fn llama3_1_8b() -> Self {
        Self {
            name: "llama3.1:8b".to_string(),
            enabled: false,
            priority: 8,
            requirements: ModelRequirements {
                min_vram_gb: 6,
                min_ram_gb: 8,
                min_disk_gb: 5,
                requires_gpu: true,
                network_bandwidth_mbps: Some(50),
            },
        }
    }

    pub fn llama3_1_70b() -> Self {
        Self {
            name: "llama3.1:70b".to_string(),
            enabled: false,
            priority: 10,
            requirements: ModelRequirements {
                min_vram_gb: 48,
                min_ram_gb: 64,
                min_disk_gb: 40,
                requires_gpu: true,
                network_bandwidth_mbps: Some(100),
            },
        }
    }

    pub fn mistral_7b() -> Self {
        Self {
            name: "mistral:7b".to_string(),
            enabled: false,
            priority: 7,
            requirements: ModelRequirements {
                min_vram_gb: 4,
                min_ram_gb: 8,
                min_disk_gb: 4,
                requires_gpu: true,
                network_bandwidth_mbps: Some(25),
            },
        }
    }

    pub fn phi3_mini() -> Self {
        Self {
            name: "phi3:mini".to_string(),
            enabled: false,
            priority: 5,
            requirements: ModelRequirements {
                min_vram_gb: 2,
                min_ram_gb: 4,
                min_disk_gb: 2,
                requires_gpu: false,
                network_bandwidth_mbps: Some(10),
            },
        }
    }
}