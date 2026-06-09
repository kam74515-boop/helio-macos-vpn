pub mod state;
pub mod types;
pub mod utils;
pub mod commands;

use std::sync::Mutex;
use state::{AppState, TrafficSnapshot};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            engine_process: Mutex::new(None),
            traffic_snapshot: Mutex::new(TrafficSnapshot::default()),
            monitoring: Mutex::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            // Engine
            commands::singbox::start_engine,
            commands::singbox::stop_engine,
            commands::proxy::set_system_proxy,
            // System data
            commands::network::get_system_snapshot,
            commands::process::get_processes,
            commands::process::get_connections,
            commands::traffic::get_traffic_stats,
            commands::network::get_latency,
            commands::proxy::get_proxy_state,
            commands::network::get_network_info,
            // sing-box config
            commands::singbox::get_singbox_config,
            commands::singbox::get_singbox_config_json,
            commands::singbox::update_singbox_config,
            commands::singbox::set_proxy_mode,
            commands::singbox::get_proxy_mode,
            // Speed test
            commands::network::run_speed_test,
            commands::network::run_speed_test_all,
            commands::network::test_node_latency,
            // Monitoring
            commands::singbox::start_monitoring,
            commands::singbox::stop_monitoring,
            // Subscriptions
            commands::subscription::import_subscription,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
