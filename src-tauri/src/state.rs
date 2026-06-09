use tauri_plugin_shell::process::CommandChild;
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
