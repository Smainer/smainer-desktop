#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;
use tracing_subscriber;

mod commands;
mod models;
mod utils;

use commands::{hardware, provider, wallet, monitoring};

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
            app.tray_handle().set_tooltip("Smainer Node").unwrap();
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
            
            // Wallet commands
            wallet::generate_wallet,
            wallet::get_wallet_address,
            wallet::sign_message,
            
            // Monitoring commands
            monitoring::get_node_status,
            monitoring::get_earnings,
            monitoring::get_task_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}