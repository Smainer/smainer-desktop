use serde::{Deserialize, Serialize};
use tauri::command;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupResult {
    pub success: bool,
    pub files_deleted: Vec<String>,
    pub errors: Vec<String>,
    pub message: String,
}

/// Get the path to the Smainer app data directory
fn get_app_data_dir() -> Result<PathBuf, String> {
    let mut path = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?;
    path.push(".smainer");
    Ok(path)
}

/// Clean up all Smainer application data
/// This removes wallet.json, ai_config.json, and the entire .smainer directory
/// WARNING: This permanently deletes private keys!
#[command]
pub async fn cleanup_app_data() -> Result<CleanupResult, String> {
    tracing::warn!("cleanup_app_data command invoked - will delete wallet and config data");
    
    let app_data_dir = get_app_data_dir()?;
    let mut files_deleted = Vec::new();
    let mut errors = Vec::new();
    
    if !app_data_dir.exists() {
        return Ok(CleanupResult {
            success: true,
            files_deleted: vec![],
            errors: vec![],
            message: "No application data found to delete".to_string(),
        });
    }
    
    // Track specific files
    let wallet_json = app_data_dir.join("wallet.json");
    let ai_config_json = app_data_dir.join("ai_config.json");
    
    // Delete wallet.json
    if wallet_json.exists() {
        match fs::remove_file(&wallet_json) {
            Ok(_) => {
                files_deleted.push("wallet.json".to_string());
                tracing::info!("Deleted wallet.json");
            }
            Err(e) => {
                errors.push(format!("Failed to delete wallet.json: {}", e));
                tracing::error!("Failed to delete wallet.json: {}", e);
            }
        }
    }
    
    // Delete ai_config.json
    if ai_config_json.exists() {
        match fs::remove_file(&ai_config_json) {
            Ok(_) => {
                files_deleted.push("ai_config.json".to_string());
                tracing::info!("Deleted ai_config.json");
            }
            Err(e) => {
                errors.push(format!("Failed to delete ai_config.json: {}", e));
                tracing::error!("Failed to delete ai_config.json: {}", e);
            }
        }
    }
    
    // Remove the entire .smainer directory
    match fs::remove_dir_all(&app_data_dir) {
        Ok(_) => {
            files_deleted.push(".smainer directory".to_string());
            tracing::info!("Deleted .smainer directory");
            Ok(CleanupResult {
                success: true,
                files_deleted,
                errors,
                message: "All application data deleted successfully".to_string(),
            })
        }
        Err(e) => {
            errors.push(format!("Failed to remove .smainer directory: {}", e));
            tracing::error!("Failed to remove .smainer directory: {}", e);
            
            // Partial success if we deleted some files
            if !files_deleted.is_empty() {
                Ok(CleanupResult {
                    success: false,
                    files_deleted,
                    errors,
                    message: "Partial cleanup - some files could not be removed".to_string(),
                })
            } else {
                Err(format!("Failed to delete application data: {}", e))
            }
        }
    }
}

/// Check if application data exists
#[command]
pub async fn check_app_data_exists() -> Result<bool, String> {
    let app_data_dir = get_app_data_dir()?;
    
    if !app_data_dir.exists() {
        return Ok(false);
    }
    
    // Check if any known files exist
    let wallet_exists = app_data_dir.join("wallet.json").exists();
    let config_exists = app_data_dir.join("ai_config.json").exists();
    
    Ok(wallet_exists || config_exists || app_data_dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false))
}

/// Get information about application data
#[derive(Debug, Serialize)]
pub struct AppDataInfo {
    pub exists: bool,
    pub path: String,
    pub files: Vec<String>,
    pub total_size_bytes: u64,
}

#[command]
pub async fn get_app_data_info() -> Result<AppDataInfo, String> {
    let app_data_dir = get_app_data_dir()?;
    
    if !app_data_dir.exists() {
        return Ok(AppDataInfo {
            exists: false,
            path: app_data_dir.display().to_string(),
            files: vec![],
            total_size_bytes: 0,
        });
    }
    
    let mut files = Vec::new();
    let mut total_size = 0u64;
    
    if let Ok(entries) = fs::read_dir(&app_data_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    files.push(entry.file_name().to_string_lossy().to_string());
                    total_size += metadata.len();
                }
            }
        }
    }
    
    Ok(AppDataInfo {
        exists: true,
        path: app_data_dir.display().to_string(),
        files,
        total_size_bytes: total_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_data_dir_returns_path() {
        let result = get_app_data_dir();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".smainer"));
    }

    #[tokio::test]
    async fn test_check_app_data_exists_works() {
        let result = check_app_data_exists().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_app_data_info_returns_info() {
        let result = get_app_data_info().await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.path.contains(".smainer"));
    }

    #[test]
    fn test_cleanup_result_structure() {
        let result = CleanupResult {
            success: true,
            files_deleted: vec!["wallet.json".to_string()],
            errors: vec![],
            message: "Test complete".to_string(),
        };
        
        assert_eq!(result.success, true);
        assert_eq!(result.files_deleted.len(), 1);
        assert_eq!(result.errors.len(), 0);
    }
}

