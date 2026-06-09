use crate::types::{ProcessInfo, ConnectionInfo};
use crate::utils::{run_cmd, guess_icon};
use std::collections::HashMap;

pub async fn get_processes_impl() -> Result<Vec<ProcessInfo>, String> {
    // Use lsof to find processes with active TCP connections
    let lsof_out = run_cmd(&["lsof", "-iTCP", "-sTCP:ESTABLISHED", "-n", "-P", "-F", "pc"])
        .unwrap_or_default();

    let mut pid_names: HashMap<u32, String> = HashMap::new();
    let mut pid_connections: HashMap<u32, u32> = HashMap::new();
    let mut current_pid: Option<u32> = None;

    for line in lsof_out.lines() {
        if let Some(pid_str) = line.strip_prefix('p') {
            if let Ok(pid) = pid_str.parse::<u32>() {
                current_pid = Some(pid);
                *pid_connections.entry(pid).or_insert(0) += 1;
            }
        } else if let Some(name) = line.strip_prefix('c') {
            if let Some(pid) = current_pid {
                let short_name = name.split('/').last().unwrap_or(name);
                pid_names.entry(pid).or_insert_with(|| short_name.to_string());
            }
            current_pid = None;
        }
    }

    // If lsof returns nothing, fall back to ps for process list
    if pid_names.is_empty() {
        let ps_out = run_cmd(&["ps", "aux"]).unwrap_or_default();
        for line in ps_out.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                if let Ok(pid) = parts[1].parse::<u32>() {
                    let name = parts.last().unwrap_or(&"unknown");
                    let short = name.split('/').last().unwrap_or(name);
                    pid_names.entry(pid).or_insert_with(|| short.to_string());
                }
            }
        }
    }

    // Use sysinfo for process memory info (bytes as proxy for total traffic)
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let mut result: Vec<ProcessInfo> = pid_names.iter().map(|(pid, name)| {
        let conns = pid_connections.get(pid).copied().unwrap_or(0);
        let (rx, tx) = sys
            .process(sysinfo::Pid::from(*pid as usize))
            .map(|p| (p.disk_usage().read_bytes / 1024, p.disk_usage().written_bytes / 1024))
            .unwrap_or((0, 0));

        ProcessInfo {
            name: name.clone(),
            pid: *pid,
            connections: conns,
            upload_bytes: tx,
            download_bytes: rx,
            icon_key: guess_icon(name).to_string(),
        }
    })
    .collect();

    result.sort_by(|a, b| b.connections.cmp(&a.connections));
    if result.len() > 50 { result.truncate(50); }
    Ok(result)
}

#[tauri::command]
pub async fn get_processes() -> Result<Vec<ProcessInfo>, String> {
    get_processes_impl().await
}

pub async fn get_connections_impl() -> Result<Vec<ConnectionInfo>, String> {
    let now = chrono_local();
    let mut result = Vec::new();

    // Use lsof -iTCP to get connections
    let lsof_out = run_cmd(&["lsof", "-iTCP", "-n", "-P"]).unwrap_or_default();

    let mut id: u32 = 0;
    for line in lsof_out.lines() {
        if line.starts_with("COMMAND") { continue; }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }

        // lsof format: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
        if parts.len() >= 9 {
            let process = parts[0].to_string();
            let name_field = parts[8..].join("");

            // Parse "host:port->host:port" or "host:port (ESTABLISHED)"
            let (remote, status, method) = if name_field.contains("(ESTABLISHED)") {
                let addr = name_field.replace("(ESTABLISHED)", "").trim().to_string();
                let resolved = if addr.contains("->") {
                    addr.split("->").last().unwrap_or(&addr).to_string()
                } else { addr };
                (resolved, "活跃".to_string(), "TCP".to_string())
            } else if name_field.contains("(LISTEN)") {
                (name_field.replace("(LISTEN)", "").trim().to_string(), "监听".to_string(), "TCP".to_string())
            } else {
                let resolved = if name_field.contains("->") {
                    name_field.split("->").last().unwrap_or(&name_field).to_string()
                } else { name_field.to_string() };
                (resolved, "活跃".to_string(), "TCP".to_string())
            };

            id += 1;

            result.push(ConnectionInfo {
                id: id.to_string(),
                timestamp: now.clone(),
                process,
                status,
                proxy: "direct".to_string(),
                upload: "0 B".to_string(),
                download: "0 B".to_string(),
                duration: "-".to_string(),
                method,
                remote,
            });
        }
    }

    // Cap at 100
    if result.len() > 100 { result.truncate(100); }
    Ok(result)
}

pub fn chrono_local() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let hours = ((secs % 86400) / 3600 + 8) % 24; // UTC+8
    let mins = (secs % 3600) / 60;
    let secs_r = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs_r)
}

#[tauri::command]
pub async fn get_connections() -> Result<Vec<ConnectionInfo>, String> {
    get_connections_impl().await
}
