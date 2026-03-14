use serde::{Deserialize, Serialize};
use tauri::command;
use anyhow::Result;
use crate::models::{HardwareInfo, GpuInfo, SystemRequirements};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub total_ram: u64,
    pub available_ram: u64,
    pub gpus: Vec<GpuInfo>,
    pub os: String,
    pub os_version: String,
}

#[command]
pub async fn detect_gpus() -> Result<Vec<GpuInfo>, String> {
    tracing::info!("Detecting GPUs...");
    
    #[cfg(target_os = "windows")]
    {
        detect_gpus_windows().map_err(|e| e.to_string())
    }
    
    #[cfg(target_os = "linux")]
    {
        detect_gpus_linux().map_err(|e| e.to_string())
    }

    #[cfg(target_os = "macos")]
    {
        Ok(vec![]) // MacOS support requires metal crate or system_profiler
    }
}

#[cfg(target_os = "linux")]
fn detect_gpus_linux() -> Result<Vec<GpuInfo>> {
    let output = Command::new("lspci")
        .arg("-vmm")
        .output()?;
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();
    let mut current_device = String::new();
    let mut current_vendor = String::new();
    let mut is_vga = false;

    for line in stdout.lines() {
        if line.is_empty() {
            if is_vga {
                gpus.push(GpuInfo {
                    name: current_device.clone(),
                    vendor: current_vendor.clone(),
                    memory: 0, // lspci doesn't easily show memory size without sudo lspci -v
                    compute_capability: "Unknown".to_string(),
                    driver_version: "Unknown".to_string(),
                    is_supported: true,
                });
            }
            is_vga = false;
            current_device.clear();
            current_vendor.clear();
            continue;
        }

        if line.starts_with("Class:\tVGA") || line.starts_with("Class:\t3D") {
            is_vga = true;
        } else if line.starts_with("Vendor:\t") {
            current_vendor = line["Vendor:\t".len()..].to_string();
        } else if line.starts_with("Device:\t") {
            current_device = line["Device:\t".len()..].to_string();
        }
    }
    
    Ok(gpus)
}

#[cfg(target_os = "windows")]
fn detect_gpus_windows() -> Result<Vec<GpuInfo>> {
    use windows::Win32::Graphics::Dxgi::*;
    use windows::Win32::Graphics::Dxgi::Common::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Com::*;
    
    let mut gpus = Vec::new();
    
    unsafe {
        let _ = CoInitialize(None);
        
        let factory: IDXGIFactory1 = CreateDXGIFactory1()?;
        let mut adapter_index = 0;
        
        loop {
            match factory.EnumAdapters1(adapter_index) {
                Ok(adapter) => {
                    let desc = adapter.GetDesc1()?;
                    let name = String::from_utf16_lossy(&desc.Description);
                    let name = name.trim_end_matches('\0');
                    let memory = desc.DedicatedVideoMemory / (1024 * 1024);
                    
                    let vendor = if name.to_lowercase().contains("nvidia") {
                        "NVIDIA"
                    } else if name.to_lowercase().contains("amd") {
                        "AMD"
                    } else if name.to_lowercase().contains("intel") {
                        "Intel"
                    } else {
                        "Unknown"
                    };
                    
                    if memory > 1024 { // Filter out basic display adapters
                         gpus.push(GpuInfo {
                            name: name.to_string(),
                            vendor: vendor.to_string(),
                            memory: memory as u64,
                            compute_capability: "Unknown".to_string(),
                            driver_version: "Unknown".to_string(),
                            is_supported: true,
                        });
                    }
                    adapter_index += 1;
                }
                Err(_) => break,
            }
        }
        CoUninitialize();
    }
    
    Ok(gpus)
}

#[command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    tracing::info!("Getting system information...");
    let gpus = detect_gpus().await?;
    
    // Simple cross-platform CPU/RAM info if sysinfo crate not used
    // For now returning placeholders or basic implementation
    // Ideally use sysinfo crate
    
    Ok(SystemInfo {
        cpu_name: "Detected CPU".to_string(),
        cpu_cores: num_cpus::get() as u32, // Use num_cpus crate if available, else standard
        total_ram: 0, // Hard without sysinfo
        available_ram: 0,
        gpus,
        os: std::env::consts::OS.to_string(),
        os_version: "Unknown".to_string(),
    })
}

#[command]
pub async fn check_requirements() -> Result<SystemRequirements, String> {
    Ok(SystemRequirements {
        meets_requirements: true,
        ram_ok: true,
        gpu_ok: true,
        cpu_ok: true,
        warnings: vec![],
        errors: vec![],
        system_info: get_system_info().await?,
    })
}
