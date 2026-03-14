use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub memory: u64, // Memory in MB
    pub compute_capability: String,
    pub driver_version: String,
    pub is_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub total_ram: u64, // RAM in bytes
    pub available_ram: u64, // Available RAM in bytes
    pub gpus: Vec<GpuInfo>,
    pub os: String,
    pub os_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    pub meets_requirements: bool,
    pub ram_ok: bool,
    pub gpu_ok: bool,
    pub cpu_ok: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub system_info: HardwareInfo,
}

impl Default for HardwareInfo {
    fn default() -> Self {
        Self {
            cpu_name: "Unknown CPU".to_string(),
            cpu_cores: 0,
            total_ram: 0,
            available_ram: 0,
            gpus: Vec::new(),
            os: "Unknown".to_string(),
            os_version: "Unknown".to_string(),
        }
    }
}

impl GpuInfo {
    pub fn is_nvidia(&self) -> bool {
        self.vendor.to_lowercase() == "nvidia"
    }
    
    pub fn is_amd(&self) -> bool {
        self.vendor.to_lowercase() == "amd"
    }
    
    pub fn memory_gb(&self) -> f64 {
        self.memory as f64 / 1024.0
    }
}

impl HardwareInfo {
    pub fn total_ram_gb(&self) -> f64 {
        self.total_ram as f64 / (1024.0 * 1024.0 * 1024.0)
    }
    
    pub fn available_ram_gb(&self) -> f64 {
        self.available_ram as f64 / (1024.0 * 1024.0 * 1024.0)
    }
    
    pub fn supported_gpus(&self) -> Vec<&GpuInfo> {
        self.gpus.iter().filter(|gpu| gpu.is_supported).collect()
    }
    
    pub fn best_gpu(&self) -> Option<&GpuInfo> {
        self.supported_gpus()
            .into_iter()
            .max_by_key(|gpu| gpu.memory)
    }
}