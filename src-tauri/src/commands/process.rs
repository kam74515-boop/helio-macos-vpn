use crate::types::{ProcessInfo, ConnectionInfo};
use crate::utils::{run_cmd, guess_icon, get_app_icon_base64};
use std::collections::HashMap;

pub async fn get_processes_impl() -> Result<Vec<ProcessInfo>, String> {
    tokio::task::spawn_blocking(|| {
        // 1. Get all running processes via sysinfo (comprehensive list)
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        // 2. Get network connection counts via lsof (all TCP sockets, not just ESTABLISHED)
        let lsof_out = run_cmd(&["lsof", "-iTCP", "-n", "-P", "-F", "pc"])
            .unwrap_or_default();

        let mut pid_connections: HashMap<u32, u32> = HashMap::new();
        for line in lsof_out.lines() {
            if let Some(pid_str) = line.strip_prefix('p') {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    *pid_connections.entry(pid).or_insert(0) += 1;
                }
            }
        }

        // 3. Bulk-get executable paths for all processes via ps (much faster than per-process lsof)
        let mut pid_to_exe: HashMap<u32, String> = HashMap::new();
        if let Ok(ps_out) = run_cmd(&["ps", "-axo", "pid,args"]) {
            for line in ps_out.lines().skip(1) {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(pid) = parts[0].parse::<u32>() {
                        // First arg is usually the executable path
                        let first_arg = parts[1];
                        if first_arg.starts_with('/') || first_arg.starts_with("./") {
                            pid_to_exe.insert(pid, first_arg.to_string());
                        }
                    }
                }
            }
        }

        // 4. Build comprehensive process list from sysinfo
        let mut result: Vec<ProcessInfo> = Vec::new();
        for (pid, process) in sys.processes() {
            let pid_u32 = pid.as_u32();
            let mut exe_path = process.exe().map(|p| p.to_string_lossy().to_string());

            // Fallback to ps args when sysinfo exe is unavailable
            if exe_path.is_none() {
                if let Some(ps_path) = pid_to_exe.get(&pid_u32) {
                    exe_path = Some(ps_path.clone());
                }
            }

            // Use exe path to derive real app name when sysinfo name may be truncated
            let name = if let Some(ref exe) = exe_path {
                let sysinfo_name = process.name().to_string_lossy().to_string();
                if sysinfo_name.len() <= 24 && exe.contains(".app/") {
                    exe.split(".app/").next()
                        .and_then(|p| std::path::Path::new(p).file_name())
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_string())
                        .unwrap_or(sysinfo_name)
                } else {
                    sysinfo_name
                }
            } else {
                process.name().to_string_lossy().to_string()
            };

            // Skip kernel and low-level system processes that clutter the list
            let skip_list = [
                "kernel_task", "launchd", "kernel", "syslogd", "notifyd",
                "distnoted", "cfprefsd", "cloudd", "biomed", "fileproviderd",
                "secinitd", "authd", "usermgrd", "logd", "fseventsd",
                "kextd", "configd", "powerd", "thermalmonitord", "appleeventsd",
                "wdhelper", "talagent", "secd", "akd", "apsd",
                "mdworker", "mds", "mds_stores", "spotlight", "coreaudiod",
                "bluetoothd", "wirelessproxd", "wifiaxyd", "locationd",
            ];
            if skip_list.contains(&name.as_str()) {
                continue;
            }

            let conns = pid_connections.get(&pid_u32).copied().unwrap_or(0);
            // Note: sysinfo disk_usage is disk I/O, not network traffic.
            // Real per-process network bytes require nettop or sampling lsof.
            // We show connection count as the primary metric and zero traffic
            // to avoid misleading disk I/O as network data.
            let (rx, tx) = (0u64, 0u64);

            let exe_path_ref = exe_path.as_deref();
            let icon_base64 = get_app_icon_base64(pid_u32, &name, exe_path_ref);

            result.push(ProcessInfo {
                name: name.clone(),
                pid: pid_u32,
                connections: conns,
                upload_bytes: tx,
                download_bytes: rx,
                icon_key: guess_icon(&name, exe_path_ref).to_string(),
                icon_base64,
                policy: "direct".to_string(),
                last_address: "-".to_string(),
                dns_resolver: "system".to_string(),
                traffic_history: Vec::new(),
            });
        }

        // Sort: processes with connections first, then by name
        result.sort_by(|a, b| {
            b.connections.cmp(&a.connections)
                .then_with(|| a.name.cmp(&b.name))
        });

        // Show top 100 processes
        if result.len() > 100 { result.truncate(100); }
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_processes() -> Result<Vec<ProcessInfo>, String> {
    get_processes_impl().await
}

pub async fn get_connections_impl() -> Result<Vec<ConnectionInfo>, String> {
    tokio::task::spawn_blocking(|| {
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
    })
    .await
    .map_err(|e| e.to_string())?
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
