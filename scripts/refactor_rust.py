import os

with open('src-tauri/src/lib.rs', 'r') as f:
    lines = [line.rstrip() for line in f]

def get_lines(start, end):
    # 1-indexed to 0-indexed, end inclusive
    return '\n'.join(lines[start-1:end]) + '\n'

os.makedirs('src-tauri/src/commands', exist_ok=True)

# state.rs
state_rs = '''use tauri_plugin_shell::process::CommandChild;
use std::sync::Mutex;

pub struct AppState {
    pub engine_process: Mutex<Option<CommandChild>>,
    pub traffic_snapshot: Mutex<TrafficSnapshot>,
    pub monitoring: Mutex<bool>,
}

#[derive(Debug, Clone)]
pub struct TrafficSnapshot {
    pub prev_rx: u64,
    pub prev_tx: u64,
    pub prev_time: std::time::Instant,
    pub history_rx: Vec<f64>,
    pub history_tx: Vec<f64>,
}

impl Default for TrafficSnapshot {
    fn default() -> Self {
        Self {
            prev_rx: 0,
            prev_tx: 0,
            prev_time: std::time::Instant::now(),
            history_rx: Vec::new(),
            history_tx: Vec::new(),
        }
    }
}
'''
with open('src-tauri/src/state.rs', 'w') as f: f.write(state_rs)

# types.rs
types_rs = 'use serde::{Deserialize, Serialize};\n\n' + get_lines(42, 132)
with open('src-tauri/src/types.rs', 'w') as f: f.write(types_rs)

# utils.rs
utils_rs = 'use std::process::Command;\n\n'
utils_rs += get_lines(136, 155).replace('fn run_cmd', 'pub fn run_cmd').replace('fn run_cmd_stderr', 'pub fn run_cmd_stderr') + '\n'
utils_rs += get_lines(159, 191).replace('fn guess_icon', 'pub fn guess_icon')
with open('src-tauri/src/utils.rs', 'w') as f: f.write(utils_rs)

# commands/process.rs
proc_rs = '''use crate::types::{ProcessInfo, ConnectionInfo};
use crate::utils::{run_cmd, guess_icon};
use std::collections::HashMap;

'''
proc_rs += get_lines(230, 299).replace('async fn get_processes_impl', 'pub async fn get_processes_impl') + '\n'
proc_rs += get_lines(303, 374).replace('async fn get_connections_impl', 'pub async fn get_connections_impl').replace('fn chrono_local', 'pub fn chrono_local')
with open('src-tauri/src/commands/process.rs', 'w') as f: f.write(proc_rs)

# commands/traffic.rs
traffic_rs = '''use crate::state::AppState;
use tauri::{AppHandle, State};
use serde::Serialize;
use crate::utils::run_cmd_stderr;

#[derive(Debug, Serialize, Clone)]
pub struct TrafficStats {
    pub upload_kbps: f64,
    pub download_kbps: f64,
    pub total_upload_mb: f64,
    pub total_download_mb: f64,
    pub history: Vec<f64>,
}

'''
traffic_rs += get_lines(387, 508).replace('async fn get_traffic_stats_impl', 'pub async fn get_traffic_stats_impl').replace('fn extract_bytes', 'pub fn extract_bytes')
with open('src-tauri/src/commands/traffic.rs', 'w') as f: f.write(traffic_rs)

# commands/network.rs
net_rs = '''use crate::types::{SpeedTestResult, SystemSnapshot};
use crate::utils::{run_cmd, run_cmd_stderr};
use serde::Serialize;
use tauri::{AppHandle, State};
use crate::state::AppState;
use crate::commands::process::{get_processes_impl, get_connections_impl};
use crate::commands::traffic::get_traffic_stats_impl;
use crate::commands::proxy::get_proxy_state_impl;

#[derive(Debug, Serialize, Clone)]
pub struct LatencyResult {
    pub internet_ms: Option<f64>,
    pub dns_ms: Option<f64>,
    pub router_ms: Option<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct NetworkInfo {
    pub ssid: String,
    pub local_ip: String,
    pub external_ip: String,
    pub interface: String,
    pub config_name: String,
}

'''
net_rs += get_lines(195, 226).replace('async fn get_system_snapshot', 'pub async fn get_system_snapshot') + '\n'
net_rs += get_lines(519, 606).replace('async fn get_latency_impl', 'pub async fn get_latency_impl').replace('fn parse_ping_avg', 'pub fn parse_ping_avg').replace('fn parse_dig_time', 'pub fn parse_dig_time').replace('fn get_default_gateway', 'pub fn get_default_gateway') + '\n'
net_rs += get_lines(657, 740).replace('async fn get_network_info_impl', 'pub async fn get_network_info_impl').replace('fn get_ssid', 'pub fn get_ssid').replace('fn get_local_ip', 'pub fn get_local_ip').replace('async fn get_external_ip', 'pub async fn get_external_ip').replace('fn get_active_iface', 'pub fn get_active_iface') + '\n'
net_rs += get_lines(893, 926)
with open('src-tauri/src/commands/network.rs', 'w') as f: f.write(net_rs)

# commands/proxy.rs
proxy_rs = '''use crate::types::ProxyState;
use crate::utils::run_cmd_stderr;

'''
proxy_rs += get_lines(610, 644).replace('async fn get_proxy_state_impl', 'pub async fn get_proxy_state_impl').replace('fn extract_proxy_field', 'pub fn extract_proxy_field') + '\n'
proxy_rs += get_lines(981, 1001)
with open('src-tauri/src/commands/proxy.rs', 'w') as f: f.write(proxy_rs)

# commands/singbox.rs
singbox_rs = '''use tauri::{AppHandle, State, Manager};
use tauri_plugin_shell::ShellExt;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use crate::state::AppState;
use crate::types::{SingboxConfig, SingboxOutbound, SingboxRule};

'''
singbox_rs += get_lines(744, 841).replace('fn default_singbox_config', 'pub fn default_singbox_config') + '\n'
singbox_rs += get_lines(843, 889) + '\n'
singbox_rs += get_lines(930, 979) + '\n'
singbox_rs += get_lines(1005, 1038)
with open('src-tauri/src/commands/singbox.rs', 'w') as f: f.write(singbox_rs)

# commands/mod.rs
mod_rs = '''pub mod process;
pub mod traffic;
pub mod network;
pub mod proxy;
pub mod singbox;
'''
with open('src-tauri/src/commands/mod.rs', 'w') as f: f.write(mod_rs)

# lib.rs (refactored)
lib_rs = '''pub mod state;
pub mod types;
pub mod utils;
pub mod commands;

use std::sync::Mutex;
use tauri::Manager;
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
            commands::singbox::update_singbox_config,
            // Speed test
            commands::network::run_speed_test,
            commands::network::run_speed_test_all,
            // Monitoring
            commands::singbox::start_monitoring,
            commands::singbox::stop_monitoring,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
'''
with open('src-tauri/src/lib.rs', 'w') as f: f.write(lib_rs)

print("Rust refactor done.")
