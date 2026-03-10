use serde::{Deserialize, Serialize};
use tauri::command;
use anyhow::Result;
use crate::models::{HardwareInfo, GpuInfo, SystemRequirements};

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
    
    #[cfg(not(target_os = "windows"))]
    {
        // Mock data for non-Windows development
        Ok(vec![
            GpuInfo {
                name: "NVIDIA RTX 4090".to_string(),
                vendor: "NVIDIA".to_string(),
                memory: 24576, // 24GB
                compute_capability: "8.9".to_string(),
                driver_version: "551.76".to_string(),
                is_supported: true,
            },
            GpuInfo {
                name: "AMD RX 7900 XTX".to_string(),
                vendor: "AMD".to_string(),
                memory: 24576, // 24GB
                compute_capability: "RDNA3".to_string(),
                driver_version: "23.12.1".to_string(),
                is_supported: true,
            }
        ])
    }
}

#[cfg(target_os = "windows")]
fn detect_gpus_windows() -> Result<Vec<GpuInfo>> {
    use windows::Win32::Graphics::Dxgi::*;
    use windows::Win32::Graphics::Dxgi::Common::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Com::*;
    
    let mut gpus = Vec::new();
    
    unsafe {
        CoInitialize(None)?;
        
        let factory: IDXGIFactory1 = CreateDXGIFactory1()?;
        let mut adapter_index = 0;
        
        loop {
            match factory.EnumAdapters1(adapter_index) {
                Ok(adapter) => {
                    let desc = adapter.GetDesc1()?;
                    
                    let name = String::from_utf16_lossy(&desc.Description);
                    let name = name.trim_end_matches('\0');
                    
                    let memory = desc.DedicatedVideoMemory / (1024 * 1024); // Convert to MB
                    
                    let vendor = if name.to_lowercase().contains("nvidia") {
                        "NVIDIA"
                    } else if name.to_lowercase().contains("amd") || name.to_lowercase().contains("radeon") {
                        "AMD"
                    } else if name.to_lowercase().contains("intel") {
                        "Intel"
                    } else {
                        "Unknown"
                    };
                    
                    // Basic support check - exclude integrated graphics with < 4GB
                    let is_supported = memory > 4096 && vendor != "Intel";
                    
                    gpus.push(GpuInfo {
                        name: name.to_string(),
                        vendor: vendor.to_string(),
                        memory,
                        compute_capability: "Unknown".to_string(), // Would need more complex detection
                        driver_version: "Unknown".to_string(), // Would need registry lookup
                        is_supported,
                    });
                    
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
    
    #[cfg(target_os = "windows")]
    {
        get_system_info_windows(gpus).map_err(|e| e.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Mock data for non-Windows development
        Ok(SystemInfo {
            cpu_name: "Intel Core i9-13900K".to_string(),
            cpu_cores: 24,
            total_ram: 32 * 1024 * 1024 * 1024, // 32GB
            available_ram: 16 * 1024 * 1024 * 1024, // 16GB available
            gpus,
            os: "Windows".to_string(),
            os_version: "11".to_string(),
        })
    }
}

#[cfg(target_os = "windows")]
fn get_system_info_windows(gpus: Vec<GpuInfo>) -> Result<SystemInfo> {
    use windows::Win32::System::SystemInformation::*;
    use std::mem;
    
    unsafe {
        let mut sys_info: SYSTEM_INFO = mem::zeroed();
        GetSystemInfo(&mut sys_info);
        
        let mut mem_status: MEMORYSTATUSEX = mem::zeroed();
        mem_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;
        GlobalMemoryStatusEx(&mut mem_status)?;
        
        // Get CPU name from registry
        let cpu_name = get_cpu_name_from_registry()
            .unwrap_or_else(|_| "Unknown CPU".to_string());
        
        Ok(SystemInfo {
            cpu_name,
            cpu_cores: sys_info.dwNumberOfProcessors,
            total_ram: mem_status.ullTotalPhys,
            available_ram: mem_status.ullAvailPhys,
            gpus,
            os: "Windows".to_string(),
            os_version: get_windows_version().unwrap_or_else(|_| "Unknown".to_string()),
        })
    }
}

#[cfg(target_os = "windows")]
fn get_cpu_name_from_registry() -> Result<String> {
    use winreg::enums::*;
    use winreg::RegKey;
    
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0")?;
    let cpu_name: String = key.get_value("ProcessorNameString")?;
    Ok(cpu_name.trim().to_string())
}

#[cfg(target_os = "windows")]
fn get_windows_version() -> Result<String> {
    use windows::Win32::System::SystemInformation::*;
    use std::mem;
    
    unsafe {
        let mut version_info: OSVERSIONINFOEXW = mem::zeroed();
        version_info.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOEXW>() as u32;
        
        // Note: GetVersionEx is deprecated but still works for basic version info
        if GetVersionExW(&mut version_info.dwOSVersionInfoSize as *mut _ as *mut OSVERSIONINFOW).as_bool() {
            Ok(format!("{}.{}", version_info.dwMajorVersion, version_info.dwMinorVersion))
        } else {
            Ok("Unknown".to_string())
        }
    }
}

#[command]
pub async fn check_requirements() -> Result<SystemRequirements, String> {
    tracing::info!("Checking system requirements...");
    
    let system_info = get_system_info().await?;
    
    let min_ram = 8 * 1024 * 1024 * 1024; // 8GB minimum
    let min_gpu_memory = 4 * 1024; // 4GB minimum GPU memory
    
    let ram_ok = system_info.total_ram >= min_ram;
    let gpu_ok = system_info.gpus.iter().any(|gpu| gpu.memory >= min_gpu_memory && gpu.is_supported);
    let cpu_ok = system_info.cpu_cores >= 4;
    
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    
    if !ram_ok {
        errors.push(format!("Insufficient RAM: {}GB available, 8GB required", 
            system_info.total_ram / (1024 * 1024 * 1024)));
    }
    
    if !gpu_ok {
        errors.push("No supported GPU found. Requires NVIDIA/AMD GPU with 4GB+ VRAM".to_string());
    }
    
    if !cpu_ok {
        warnings.push(format!("Low CPU core count: {} cores, 4+ recommended", system_info.cpu_cores));
    }
    
    let meets_requirements = ram_ok && gpu_ok && cpu_ok;
    
    if system_info.available_ram < system_info.total_ram / 2 {
        warnings.push("High memory usage detected. Consider closing other applications".to_string());
    }
    
    Ok(SystemRequirements {
        meets_requirements,
        ram_ok,
        gpu_ok,
        cpu_ok,
        warnings,
        errors,
        system_info,
    })
}