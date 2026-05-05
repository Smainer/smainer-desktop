#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;
use tauri::tray::TrayIconBuilder;
use tracing_subscriber;

mod commands;
mod models;
mod utils;

use commands::{hardware, provider, wallet, monitoring, cleanup, diagnostics};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // Initialize app state
            app.manage(commands::provider::ProviderState::default());
            
            // Initialize system tray
            TrayIconBuilder::new()
                .tooltip("Smainer Node")
                .build(app.handle())
                .expect("failed to create tray icon");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Hardware detection commands
            hardware::detect_gpus,
            hardware::get_system_info,
            hardware::check_requirements,
            
            // Provider management commands
            provider::start_provider,
            provider::stop_provider,
            provider::get_provider_status,
            provider::register_node,
            provider::check_registration_status,
            provider::save_ai_config,
            provider::load_ai_config,
            provider::validate_ai_capabilities,
            provider::install_ollama,
            provider::check_ollama_installed,
            provider::get_provider_log_path_cmd,
            
            // Wallet commands
            wallet::generate_wallet,
            wallet::import_wallet,
            wallet::get_wallet_address,
            wallet::sign_message,
            
            // Monitoring commands
            monitoring::get_node_status,
            monitoring::get_earnings,
            monitoring::get_task_history,
            
            // Cleanup commands
            cleanup::cleanup_app_data,
            cleanup::check_app_data_exists,
            cleanup::get_app_data_info,
            
            // Diagnostics commands
            diagnostics::export_diagnostics_bundle,
            diagnostics::get_last_diagnostics_bundle
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}