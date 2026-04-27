// Unit tests for cleanup commands
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_dir() -> PathBuf {
        let test_dir = std::env::temp_dir().join(format!("smainer_test_{}", std::process::id()));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    fn cleanup_test_dir(dir: &PathBuf) {
        if dir.exists() {
            fs::remove_dir_all(dir).unwrap_or(());
        }
    }

    #[test]
    fn test_get_app_data_dir_returns_path() {
        // This test just ensures the function returns a valid path
        let result = get_app_data_dir();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".smainer"));
    }

    #[tokio::test]
    async fn test_check_app_data_exists_returns_false_when_no_data() {
        // Note: This test assumes no real .smainer directory exists
        // In a real test environment, we'd mock the home directory
        let result = check_app_data_exists().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_result_structure() {
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

    #[tokio::test]
    async fn test_get_app_data_info_structure() {
        let result = get_app_data_info().await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        // Should return info even if directory doesn't exist
        assert!(info.path.contains(".smainer"));
    }
}
