pub mod state;
pub mod types;
pub mod utils;
pub mod commands;
pub mod config_store;

use std::sync::Mutex;
use state::{AppState, TrafficSnapshot};
use tauri::Manager;

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
            // TUN / Enhanced mode
            commands::tun::get_tun_status,
            commands::tun::get_permission_status,
            commands::tun::toggle_enhanced_mode,
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
            commands::singbox::save_outbound,
            commands::singbox::delete_outbound,
            commands::singbox::save_selector_group,
            commands::singbox::set_selector_default,
            commands::singbox::switch_proxy,
            commands::singbox::get_proxy_config,
            // Speed test
            commands::network::run_speed_test,
            commands::network::run_speed_test_all,
            commands::network::test_node_latency,
            commands::network::run_network_diagnostics,
            // Monitoring
            commands::singbox::start_monitoring,
            commands::singbox::stop_monitoring,
            // Clash API
            commands::clash_api::get_clash_proxies,
            commands::clash_api::get_clash_connections,
            commands::clash_api::get_clash_traffic,
            commands::clash_api::test_proxy_latency,
            commands::clash_api::close_connection,
            commands::clash_api::clear_connections,
            commands::clash_api::toggle_capture,
            // Subscriptions
            commands::subscription::import_subscription,
            // Profile management
            commands::config_store::list_profiles,
            commands::config_store::get_active_profile,
            commands::config_store::switch_profile,
            commands::config_store::save_profile,
            // CA management
            commands::ca::get_ca_status,
            commands::ca::generate_ca,
            commands::ca::export_ca,
            commands::ca::install_ca,
            // Rules CRUD
            commands::rules::get_rules,
            commands::rules::add_rule,
            commands::rules::edit_rule,
            commands::rules::delete_rule,
            commands::rules::reorder_rules,
            commands::rules::reset_rule_counters,
            // MITM hostnames
            commands::mitm::get_mitm_hostnames,
            commands::mitm::add_mitm_hostname,
            commands::mitm::remove_mitm_hostname,
            commands::mitm::set_mitm_enabled,
            // LAN devices
            commands::devices::get_lan_devices,
        ])
        .setup(|app| {
            let store = config_store::ConfigStore::new(&app.handle())?;
            app.manage(store);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
