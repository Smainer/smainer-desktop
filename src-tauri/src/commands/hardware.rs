use serde::{Deserialize, Serialize};
use tauri::command;
use anyhow::Result;
use crate::models::{HardwareInfo, GpuInfo, SystemRequirements};

#[cfg(target_os = "linux")]
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
    use windows::Win32::System::Com::*;
    
    let mut gpus = Vec::new();
    
    unsafe {
        let _ = CoInitialize(None);
        
        let factory: IDXGIFactory1 = CreateDXGIFactory1()?;
        let mut adapter_index = 0;
        
        loop {
            match factory.EnumAdapters1(adapter_index) {
                Ok(adapter) => {
                    let mut desc = std::mem::zeroed::<DXGI_ADAPTER_DESC1>();
                    adapter.GetDesc1(&mut desc)?;
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

#[cfg(target_os = "windows")]
fn get_total_ram_bytes() -> u64 {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    unsafe {
        let mut mem_status: MEMORYSTATUSEX = std::mem::zeroed();
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        
        if GlobalMemoryStatusEx(&mut mem_status).is_ok() {
            mem_status.ullTotalPhys
        } else {
            0
        }
    }
}

#[cfg(target_os = "windows")]
fn get_available_ram_bytes() -> u64 {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    unsafe {
        let mut mem_status: MEMORYSTATUSEX = std::mem::zeroed();
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        
        if GlobalMemoryStatusEx(&mut mem_status).is_ok() {
            mem_status.ullAvailPhys
        } else {
            0
        }
    }
}

#[cfg(target_os = "linux")]
fn get_total_ram_bytes() -> u64 {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        return kb * 1024; // kB → bytes
                    }
                }
            }
        }
    }
    0
}

#[cfg(target_os = "linux")]
fn get_available_ram_bytes() -> u64 {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemAvailable:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        return kb * 1024; // kB → bytes
                    }
                }
            }
        }
    }
    0
}

#[cfg(target_os = "macos")]
fn get_total_ram_bytes() -> u64 {
    0 // Unsupported on macOS
}

#[cfg(target_os = "macos")]
fn get_available_ram_bytes() -> u64 {
    0 // Unsupported on macOS
}

#[command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    tracing::info!("Getting system information...");
    let gpus = detect_gpus().await?;
    
    Ok(SystemInfo {
        cpu_name: "Detected CPU".to_string(),
        cpu_cores: num_cpus::get() as u32,
        total_ram: get_total_ram_bytes(),
        available_ram: get_available_ram_bytes(),
        gpus,
        os: std::env::consts::OS.to_string(),
        os_version: "Unknown".to_string(),
    })
}

#[command]
pub async fn check_requirements() -> Result<SystemRequirements, String> {
    let system_info = get_system_info().await?;
    Ok(SystemRequirements {
        meets_requirements: true,
        ram_ok: true,
        gpu_ok: true,
        cpu_ok: true,
        warnings: vec![],
        errors: vec![],
        system_info: HardwareInfo {
            cpu_name: system_info.cpu_name,
            cpu_cores: system_info.cpu_cores,
            total_ram: system_info.total_ram,
            available_ram: system_info.available_ram,
            gpus: system_info.gpus,
            os: system_info.os,
            os_version: system_info.os_version,
        },
    })
}
