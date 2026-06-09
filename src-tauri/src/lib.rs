use std::sync::Mutex;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandChild;
use std::io::Write;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// ── State ────────────────────────────────────────────

struct AppState {
    engine_process: Mutex<Option<CommandChild>>,
    traffic_snapshot: Mutex<TrafficSnapshot>,
    monitoring: Mutex<bool>,
}

#[derive(Debug, Clone)]
struct TrafficSnapshot {
    prev_rx: u64,
    prev_tx: u64,
    prev_time: std::time::Instant,
    history_rx: Vec<f64>,
    history_tx: Vec<f64>,
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

// ── Shared types ─────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub connections: u32,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub icon_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionInfo {
    pub id: String,
    pub timestamp: String,
    pub process: String,
    pub status: String,
    pub proxy: String,
    pub upload: String,
    pub download: String,
    pub duration: String,
    pub method: String,
    pub remote: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSnapshot {
    pub connections_total: u32,
    pub processes_with_connections: u32,
    pub upload_kbps: f64,
    pub download_kbps: f64,
    pub total_upload_mb: f64,
    pub total_download_mb: f64,
    pub external_ip: String,
    pub ssid: String,
    pub local_ip: String,
    pub internet_latency_ms: Option<f64>,
    pub dns_latency_ms: Option<f64>,
    pub router_latency_ms: Option<f64>,
    pub system_proxy_enabled: bool,
    pub traffic_history: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingboxOutbound {
    pub tag: String,
    pub outbound_type: String,
    pub server: String,
    pub server_port: u16,
    pub ping: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingboxRule {
    pub id: String,
    pub rule_type: String,
    pub value: String,
    pub action: String,
    pub hits: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingboxConfig {
    pub mode: String,
    pub outbounds: Vec<SingboxOutbound>,
    pub rules: Vec<SingboxRule>,
    pub policy_groups: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpeedTestResult {
    pub node_name: String,
    pub latency_ms: f64,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyState {
    pub system_proxy_enabled: bool,
    pub enhanced_mode: bool,
    pub http_host: String,
    pub http_port: String,
    pub socks_host: String,
    pub socks_port: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppIconMap {
    pub name: String,
    pub icon_key: String,
}

// ── Helper: run shell command ────────────────────────

fn run_cmd(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(args[0])
        .args(&args[1..])
        .output()
        .map_err(|e| format!("{}: {}", args.join(" "), e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_cmd_stderr(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(args[0])
        .args(&args[1..])
        .output()
        .map_err(|e| format!("{}: {}", args.join(" "), e))?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(combined)
}

// ── Icon mapping from process name ───────────────────

fn guess_icon(name: &str) -> &str {
    let lower = name.to_lowercase();
    if lower.contains("chrome") || lower.contains("google chrome") { "language" }
    else if lower.contains("safari") { "explore" }
    else if lower.contains("firefox") { "language" }
    else if lower.contains("terminal") || lower.contains("iterm") { "terminal" }
    else if lower.contains("xray") || lower.contains("v2ray") { "alt_route" }
    else if lower.contains("clash") || lower.contains("mihomo") { "alt_route" }
    else if lower.contains("sing-box") { "alt_route" }
    else if lower.contains("cursor") || lower.contains("code") { "deployed_code" }
    else if lower.contains("trae") { "memory" }
    else if lower.contains("wechat") || lower.contains("微信") { "chat" }
    else if lower.contains("feishu") || lower.contains("lark") || lower.contains("飞书") { "send" }
    else if lower.contains("dingtalk") || lower.contains("钉钉") { "send" }
    else if lower.contains("mail") || lower.contains("邮件") { "mail" }
    else if lower.contains("music") || lower.contains("音乐") || lower.contains("spotify") { "music_note" }
    else if lower.contains("slack") { "chat" }
    else if lower.contains("discord") { "chat" }
    else if lower.contains("zoom") { "videocam" }
    else if lower.contains("telegram") { "send" }
    else if lower.contains("quark") || lower.contains("夸克") { "cloud" }
    else if lower.contains("notion") { "description" }
    else if lower.contains("figma") { "palette" }
    else if lower.contains("docker") { "deployed_code" }
    else if lower.contains("node") || lower.contains("npm") { "deployed_code" }
    else if lower.contains("python") { "deployed_code" }
    else if lower.contains("antigravity") { "explore" }
    else if lower.contains("codex") { "deployed_code" }
    else if lower.starts_with("com.apple") { "build" }
    else if lower.contains("kernel") || lower.contains("system") || lower.contains("sys") { "build" }
    else if lower.contains("launchd") || lower.contains("core") { "build" }
    else { "memory" }
}

// ── Commands ─────────────────────────────────────────

#[tauri::command]
async fn get_system_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SystemSnapshot, String> {
    let (procs, conns) = tokio::try_join!(
        get_processes_impl(),
        get_connections_impl(),
    )?;

    let traffic = get_traffic_stats_impl(app, state).await?;
    let net_info = get_network_info_impl().await?;
    let proxy = get_proxy_state_impl().await?;
    let latency = get_latency_impl("8.8.8.8").await;

    Ok(SystemSnapshot {
        connections_total: conns.len() as u32,
        processes_with_connections: procs.len() as u32,
        upload_kbps: traffic.upload_kbps,
        download_kbps: traffic.download_kbps,
        total_upload_mb: traffic.total_upload_mb,
        total_download_mb: traffic.total_download_mb,
        external_ip: net_info.external_ip,
        ssid: net_info.ssid,
        local_ip: net_info.local_ip,
        internet_latency_ms: latency.internet_ms,
        dns_latency_ms: latency.dns_ms,
        router_latency_ms: latency.router_ms,
        system_proxy_enabled: proxy.system_proxy_enabled,
        traffic_history: traffic.history,
    })
}

// ── Processes ────────────────────────────────────────

async fn get_processes_impl() -> Result<Vec<ProcessInfo>, String> {
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
async fn get_processes() -> Result<Vec<ProcessInfo>, String> {
    get_processes_impl().await
}

// ── Connections ──────────────────────────────────────

async fn get_connections_impl() -> Result<Vec<ConnectionInfo>, String> {
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

fn chrono_local() -> String {
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
async fn get_connections() -> Result<Vec<ConnectionInfo>, String> {
    get_connections_impl().await
}

// ── Traffic Stats ────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
struct TrafficStats {
    upload_kbps: f64,
    download_kbps: f64,
    total_upload_mb: f64,
    total_download_mb: f64,
    history: Vec<f64>,
}

async fn get_traffic_stats_impl(
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TrafficStats, String> {
    // Read bytes from en0 (Wi-Fi) interface via netstat
    let ib_out = run_cmd_stderr(&["netstat", "-ib", "-n"]).unwrap_or_default();

    let mut rx_bytes: u64 = 0;
    let mut tx_bytes: u64 = 0;
    let mut found_iface = false;

    for line in ib_out.lines() {
        let lower = line.to_lowercase();
        // Look for active Wi-Fi or primary interface
        if lower.contains("en0") || lower.contains("en1") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                // netstat -ib columns: Name Mtu Network Address Ipkts Ierrs Opkts Oerrs Coll
                if let Ok(ipkts) = parts.get(4).unwrap_or(&"0").parse::<u64>() {
                    rx_bytes = ipkts * 1500; // rough estimate: packets * MTU
                }
                if let Ok(opkts) = parts.get(6).unwrap_or(&"0").parse::<u64>() {
                    tx_bytes = opkts * 1500;
                }
                found_iface = true;
            }
        }
    }

    // Fallback: use ifconfig
    if !found_iface {
        let ifc_out = run_cmd_stderr(&["ifconfig", "en0"]).unwrap_or_default();
        for line in ifc_out.lines() {
            let lower = line.to_lowercase();
            if lower.contains("bytes") {
                // Format: "RX packets 12345 bytes 67890 (66.2 KiB)"
                // Extract byte count
                if lower.contains("rx") {
                    if let Some(b) = extract_bytes(line) { rx_bytes = b; }
                }
                if lower.contains("tx") {
                    if let Some(b) = extract_bytes(line) { tx_bytes = b; }
                }
            }
        }
    }

    let total_rx = rx_bytes;
    let total_tx = tx_bytes;

    // Compute speed using previous snapshot
    let mut snap = state.traffic_snapshot.lock().unwrap();
    let now = std::time::Instant::now();
    let elapsed = snap.prev_time.elapsed().as_secs_f64();
    if elapsed < 0.1 { elapsed as f64; }

    let upload_kbps = if snap.prev_tx > 0 && elapsed > 0.0 {
        ((total_tx.saturating_sub(snap.prev_tx)) as f64 / 1024.0 / elapsed).max(0.0)
    } else { 0.0 };

    let download_kbps = if snap.prev_rx > 0 && elapsed > 0.0 {
        ((total_rx.saturating_sub(snap.prev_rx)) as f64 / 1024.0 / elapsed).max(0.0)
    } else { 0.0 };

    // History: max bar maps to ~100 KB/s, bars represent last 24 samples
    let max_bar = 100.0;
    let normalized = (download_kbps / max_bar * 100.0).min(100.0).max(4.0);

    snap.history_rx.push(normalized);
    snap.history_tx.push((upload_kbps / max_bar * 100.0).min(100.0).max(4.0));
    if snap.history_rx.len() > 24 { snap.history_rx.remove(0); }
    if snap.history_tx.len() > 24 { snap.history_tx.remove(0); }

    let history: Vec<f64> = snap.history_rx.clone();

    snap.prev_rx = total_rx;
    snap.prev_tx = total_tx;
    snap.prev_time = now;

    Ok(TrafficStats {
        upload_kbps: (upload_kbps * 10.0).round() / 10.0,
        download_kbps: (download_kbps * 10.0).round() / 10.0,
        total_upload_mb: (total_tx as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0,
        total_download_mb: (total_rx as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0,
        history,
    })
}

fn extract_bytes(line: &str) -> Option<u64> {
    // Parse "bytes 12345678" pattern from ifconfig
    for chunk in line.split_whitespace() {
        if let Ok(n) = chunk.parse::<u64>() {
            if n > 1000 {
                // Check if preceded by "bytes"
                let idx = line.find(&n.to_string())?;
                let before = &line[..idx].trim();
                if before.ends_with("bytes") {
                    return Some(n);
                }
            }
        }
    }
    // Also try "(NNN.N KiB)" format
    for chunk in line.split_whitespace() {
        let clean: String = chunk.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
        if !clean.is_empty() && clean.len() < 10 {
            if let Ok(n) = clean.parse::<f64>() {
                if n > 1.0 && n < 1_000_000_000.0 {
                    if chunk.contains("GiB") { return Some((n * 1024.0 * 1024.0 * 1024.0) as u64); }
                    if chunk.contains("MiB") { return Some((n * 1024.0 * 1024.0) as u64); }
                    if chunk.contains("KiB") { return Some((n * 1024.0) as u64); }
                }
            }
        }
    }
    None
}

#[tauri::command]
async fn get_traffic_stats(app: AppHandle, state: State<'_, AppState>) -> Result<TrafficStats, String> {
    get_traffic_stats_impl(app, state).await
}

// ── Latency ──────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
struct LatencyResult {
    internet_ms: Option<f64>,
    dns_ms: Option<f64>,
    router_ms: Option<f64>,
}

async fn get_latency_impl(host: &str) -> LatencyResult {
    let internet = tokio::task::spawn_blocking({
        let host = host.to_string();
        move || {
            let out = run_cmd(&["ping", "-c", "3", "-t", "2", &host]).unwrap_or_default();
            parse_ping_avg(&out)
        }
    });

    let dns = tokio::task::spawn_blocking({
        let host = host.to_string();
        move || {
            // Use dig for DNS timing
            let out = run_cmd(&["dig", "+time=2", "+tries=1", &host]).unwrap_or_default();
            parse_dig_time(&out)
        }
    });

    let router = tokio::task::spawn_blocking(|| {
        // Ping default gateway
        let gw = get_default_gateway().unwrap_or_else(|| "192.168.1.1".to_string());
        let out = run_cmd(&["ping", "-c", "1", "-t", "1", &gw]).unwrap_or_default();
        parse_ping_avg(&out)
    });

    let (i, d, r) = tokio::try_join!(internet, dns, router).unwrap_or_default();
    LatencyResult { internet_ms: i, dns_ms: d, router_ms: r }
}

fn parse_ping_avg(output: &str) -> Option<f64> {
    // Parse "round-trip min/avg/max/stddev = 4.567/8.901/15.234/3.456 ms"
    for line in output.lines() {
        if line.contains("min/avg/max") || line.contains("avg") {
            if let Some(avg) = line.split('/').nth(1) {
                return avg.trim().parse::<f64>().ok();
            }
        }
        // Alternative: "time=12.3 ms"
        if line.contains("time=") {
            let parts: Vec<&str> = line.split("time=").collect();
            if parts.len() >= 2 {
                let ms = parts[1].split_whitespace().next().unwrap_or("0");
                return ms.parse::<f64>().ok();
            }
        }
    }
    None
}

fn parse_dig_time(output: &str) -> Option<f64> {
    // ";; Query time: 36 msec"
    for line in output.lines() {
        if line.contains("Query time:") {
            let parts: Vec<&str> = line.split("Query time:").collect();
            if parts.len() >= 2 {
                let ms = parts[1].split_whitespace().next().unwrap_or("0");
                return ms.parse::<f64>().ok();
            }
        }
    }
    None
}

fn get_default_gateway() -> Option<String> {
    let out = run_cmd_stderr(&["netstat", "-rn", "-f", "inet"]).unwrap_or_default();
    for line in out.lines() {
        if line.contains("default") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 { return Some(parts[1].to_string()); }
        }
    }
    // Fallback
    let route_out = run_cmd_stderr(&["route", "-n", "get", "default"]).unwrap_or_default();
    for line in route_out.lines() {
        if line.contains("gateway:") {
            let parts: Vec<&str> = line.split("gateway:").collect();
            if parts.len() >= 2 {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    None
}

#[tauri::command]
async fn get_latency(host: String) -> Result<LatencyResult, String> {
    Ok(get_latency_impl(&host).await)
}

// ── Proxy State ──────────────────────────────────────

async fn get_proxy_state_impl() -> Result<ProxyState, String> {
    let http = run_cmd_stderr(&["networksetup", "-getwebproxy", "Wi-Fi"]).unwrap_or_default();
    let socks = run_cmd_stderr(&["networksetup", "-getsocksfirewallproxy", "Wi-Fi"]).unwrap_or_default();

    let http_enabled = http.contains("Enabled: Yes");
    let socks_enabled = socks.contains("Enabled: Yes");

    let http_host = extract_proxy_field(&http, "Server:");
    let http_port = extract_proxy_field(&http, "Port:");
    let socks_host = extract_proxy_field(&socks, "Server:");
    let socks_port = extract_proxy_field(&socks, "Port:");

    Ok(ProxyState {
        system_proxy_enabled: http_enabled,
        enhanced_mode: socks_enabled,
        http_host,
        http_port: if http_port.is_empty() { "80".to_string() } else { http_port },
        socks_host,
        socks_port: if socks_port.is_empty() { "1080".to_string() } else { socks_port },
    })
}

fn extract_proxy_field(output: &str, field: &str) -> String {
    for line in output.lines() {
        if line.contains(field) {
            return line.split(':').last().unwrap_or("").trim().to_string();
        }
    }
    String::new()
}

#[tauri::command]
async fn get_proxy_state() -> Result<ProxyState, String> {
    get_proxy_state_impl().await
}

// ── Network Info ─────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
struct NetworkInfo {
    ssid: String,
    local_ip: String,
    external_ip: String,
    interface: String,
    config_name: String,
}

async fn get_network_info_impl() -> Result<NetworkInfo, String> {
    let ssid = get_ssid();
    let local_ip = get_local_ip();
    let external_ip = get_external_ip().await;
    let iface = get_active_iface();

    Ok(NetworkInfo {
        ssid,
        local_ip,
        external_ip,
        interface: iface.clone(),
        config_name: iface,
    })
}

fn get_ssid() -> String {
    let out = run_cmd_stderr(&["networksetup", "-getairportnetwork", "en0"]).unwrap_or_default();
    for line in out.lines() {
        if let Some(ssid) = line.strip_prefix("Current Wi-Fi Network: ") {
            return ssid.trim().to_string();
        }
    }
    // Fallback: use airport command
    let alt = run_cmd(&[
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport",
        "-I",
    ]).unwrap_or_default();
    for line in alt.lines() {
        if line.trim().starts_with("SSID:") {
            return line.split(':').last().unwrap_or("").trim().to_string();
        }
    }
    "未连接".to_string()
}

fn get_local_ip() -> String {
    let out = run_cmd(&["ifconfig", "en0"]).unwrap_or_default();
    for line in out.lines() {
        if line.contains("inet ") && !line.contains("127.0.0.1") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, p) in parts.iter().enumerate() {
                if *p == "inet" && i + 1 < parts.len() {
                    let ip = parts[i + 1];
                    // Remove netmask if attached
                    return ip.split('/').next().unwrap_or(ip).to_string();
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

async fn get_external_ip() -> String {
    // Try curl to ifconfig.me with short timeout
    let out = run_cmd(&["curl", "-s", "--connect-timeout", "3", "--max-time", "5", "https://ifconfig.me"])
        .unwrap_or_default();
    let ip = out.trim();
    if ip.chars().filter(|c| *c == '.').count() == 3 && ip.len() <= 15 {
        return ip.to_string();
    }
    // Fallback
    let out2 = run_cmd(&["curl", "-s", "--connect-timeout", "3", "--max-time", "5", "https://api.ipify.org"])
        .unwrap_or_default();
    let ip2 = out2.trim();
    if ip2.chars().filter(|c| *c == '.').count() == 3 && ip2.len() <= 15 {
        return ip2.to_string();
    }
    "未知".to_string()
}

fn get_active_iface() -> String {
    let out = run_cmd_stderr(&["route", "-n", "get", "default"]).unwrap_or_default();
    for line in out.lines() {
        if line.contains("interface:") {
            return line.split(':').last().unwrap_or("en0").trim().to_string();
        }
    }
    "en0".to_string()
}

#[tauri::command]
async fn get_network_info() -> Result<NetworkInfo, String> {
    get_network_info_impl().await
}

// ── sing-box Config ──────────────────────────────────

#[tauri::command]
async fn get_singbox_config(app: AppHandle) -> Result<SingboxConfig, String> {
    let config_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = config_dir.join("config.json");

    let content = fs::read_to_string(&config_path)
        .unwrap_or_else(|_| default_singbox_config());

    let val: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    let mut outbounds = Vec::new();
    if let Some(obs) = val["outbounds"].as_array() {
        for ob in obs {
            let tag = ob["tag"].as_str().unwrap_or("unknown").to_string();
            let ob_type = ob["type"].as_str().unwrap_or("direct").to_string();
            let server = ob["server"].as_str().unwrap_or("-").to_string();
            let port = ob["server_port"].as_u64().unwrap_or(0) as u16;

            outbounds.push(SingboxOutbound {
                tag,
                outbound_type: ob_type,
                server,
                server_port: port,
                ping: "-".to_string(),
                state: "ok".to_string(),
            });
        }
    }

    let mut rules = Vec::new();
    if let Some(rls) = val["route"]["rules"].as_array() {
        for (i, r) in rls.iter().enumerate() {
            let rule_type = if r["domain"].is_array() { "DOMAIN" }
                else if r["domain_suffix"].is_array() { "DOMAIN-SUFFIX" }
                else if r["domain_keyword"].is_array() { "DOMAIN-KEYWORD" }
                else if r["geosite"].is_string() { "GEOSITE" }
                else if r["geoip"].is_string() { "GEOIP" }
                else if r["ip_cidr"].is_array() { "IP-CIDR" }
                else if r["protocol"].is_string() { "PROTOCOL" }
                else { "RULE" };

            let value = if let Some(d) = r["domain"].as_array() {
                d.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")
            } else if let Some(d) = r["domain_suffix"].as_array() {
                d.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")
            } else if let Some(k) = r["domain_keyword"].as_array() {
                k.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")
            } else if let Some(g) = r["geosite"].as_str() {
                g.to_string()
            } else if let Some(g) = r["geoip"].as_str() {
                g.to_string()
            } else if let Some(i) = r["ip_cidr"].as_array() {
                i.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")
            } else { "-".to_string() };

            let action = r["outbound"].as_str().unwrap_or("direct").to_string();

            rules.push(SingboxRule {
                id: i.to_string(),
                rule_type: rule_type.to_string(),
                value,
                action,
                hits: "0".to_string(),
            });
        }
    }

    // If no rules parsed, add FINAL
    if rules.is_empty() {
        rules.push(SingboxRule {
            id: "0".to_string(),
            rule_type: "FINAL".to_string(),
            value: "-".to_string(),
            action: "direct".to_string(),
            hits: "0".to_string(),
        });
    }

    let mode = if val["route"]["auto_detect_interface"].as_bool().unwrap_or(false) {
        "规则判定"
    } else {
        "全局代理"
    };

    let policy_groups: Vec<serde_json::Value> = Vec::new();

    Ok(SingboxConfig { mode: mode.to_string(), outbounds, rules, policy_groups })
}

fn default_singbox_config() -> String {
    r#"{
  "log": {"level": "info"},
  "inbounds": [{"type": "mixed", "tag": "mixed-in", "listen": "127.0.0.1", "listen_port": 6152}],
  "outbounds": [{"type": "direct", "tag": "direct"}, {"type": "block", "tag": "block"}],
  "route": {"rules": [{"outbound": "direct", "domain_suffix": ["apple.com", "icloud.com"]}]}
}"#.to_string()
}

#[tauri::command]
async fn update_singbox_config(app: AppHandle, state: State<'_, AppState>, config: String) -> Result<(), String> {
    let config_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&config_dir).unwrap_or_default();
    let config_path = config_dir.join("config.json");

    // Validate JSON
    let _: serde_json::Value = serde_json::from_str(&config)
        .map_err(|e| format!("无效的配置 JSON: {}", e))?;

    fs::write(&config_path, &config).map_err(|e| e.to_string())?;

    // Restart engine with new config
    let mut current = state.engine_process.lock().unwrap();
    if let Some(child) = current.take() {
        let _ = child.kill();
    }

    // Relaunch
    drop(current);
    let sidecar = app.shell().sidecar("sing-box").map_err(|e| e.to_string())?;
    let (rx, child) = sidecar
        .args(["run", "-c", config_path.to_str().unwrap()])
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut current = state.engine_process.lock().unwrap();
    *current = Some(child);

    // Log relay
    tauri::async_runtime::spawn(async move {
        let mut rx = rx;
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Stdout(line) => {
                    log::info!("sing-box: {:?}", String::from_utf8_lossy(&line));
                }
                tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                    log::error!("sing-box: {:?}", String::from_utf8_lossy(&line));
                }
                _ => {}
            }
        }
    });

    Ok(())
}

// ── Speed Test ───────────────────────────────────────

#[tauri::command]
async fn run_speed_test(node_name: String) -> Result<SpeedTestResult, String> {
    // Ping the node to measure latency
    let out = run_cmd_stderr(&["ping", "-c", "2", "-t", "3", &node_name]).unwrap_or_default();
    let latency = parse_ping_avg(&out).unwrap_or(999.0);
    Ok(SpeedTestResult {
        node_name: node_name.clone(),
        latency_ms: latency,
        success: latency < 500.0,
    })
}

#[tauri::command]
async fn run_speed_test_all() -> Result<Vec<SpeedTestResult>, String> {
    // Test connectivity to common targets
    let targets = vec![
        ("谷歌 DNS", "8.8.8.8"),
        ("Cloudflare DNS", "1.1.1.1"),
        ("百度", "www.baidu.com"),
        ("GitHub", "github.com"),
    ];

    let mut results = Vec::new();
    for (name, host) in &targets {
        let out = run_cmd_stderr(&["ping", "-c", "2", "-t", "2", host]).unwrap_or_default();
        let latency = parse_ping_avg(&out).unwrap_or(999.0);
        results.push(SpeedTestResult {
            node_name: name.to_string(),
            latency_ms: latency,
            success: latency < 500.0,
        });
    }
    Ok(results)
}

// ── Start / Stop Engine (existing, enhanced) ─────────

#[tauri::command]
async fn start_engine(app: AppHandle, state: State<'_, AppState>, config: String) -> Result<(), String> {
    let config_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&config_dir).unwrap_or_default();
    let config_path = config_dir.join("config.json");

    if let Ok(mut file) = fs::File::create(&config_path) {
        let _ = file.write_all(config.as_bytes());
    } else {
        return Err("无法写入 config.json".into());
    }

    let mut current = state.engine_process.lock().unwrap();
    if let Some(child) = current.take() {
        let _ = child.kill();
    }

    let sidecar = app.shell().sidecar("sing-box").map_err(|e| e.to_string())?;
    let (mut rx, child) = sidecar
        .args(["run", "-c", config_path.to_str().unwrap()])
        .spawn()
        .map_err(|e| e.to_string())?;

    *current = Some(child);

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Stdout(line) => {
                    log::info!("sing-box: {:?}", String::from_utf8_lossy(&line));
                }
                tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                    log::error!("sing-box: {:?}", String::from_utf8_lossy(&line));
                }
                _ => {}
            }
        }
    });

    Ok(())
}

#[tauri::command]
async fn stop_engine(state: State<'_, AppState>) -> Result<(), String> {
    let mut current = state.engine_process.lock().unwrap();
    if let Some(child) = current.take() {
        let _ = child.kill();
    }
    Ok(())
}

#[tauri::command]
async fn set_system_proxy(enable: bool) -> Result<(), String> {
    let state_str = if enable { "on" } else { "off" };

    std::process::Command::new("networksetup")
        .args(["-setsocksfirewallproxy", "Wi-Fi", "127.0.0.1", "6153", state_str])
        .output()
        .map_err(|e| e.to_string())?;

    std::process::Command::new("networksetup")
        .args(["-setwebproxy", "Wi-Fi", "127.0.0.1", "6152", state_str])
        .output()
        .map_err(|e| e.to_string())?;

    std::process::Command::new("networksetup")
        .args(["-setsecurewebproxy", "Wi-Fi", "127.0.0.1", "6152", state_str])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ── Monitoring (Tauri events) ────────────────────────

#[tauri::command]
async fn start_monitoring(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut is_monitoring = state.monitoring.lock().unwrap();
    if *is_monitoring {
        return Ok(());
    }
    *is_monitoring = true;
    drop(is_monitoring);

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let traffic = {
                let state = app_handle.state::<AppState>();
                get_traffic_stats_impl(app_handle.clone(), state).await
            };

            if let Ok(traffic) = traffic {
                let _ = app_handle.emit("traffic-update", &traffic);
            }
        }
    });

    Ok(())
}

#[tauri::command]
async fn stop_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    let mut is_monitoring = state.monitoring.lock().unwrap();
    *is_monitoring = false;
    Ok(())
}

// ── App Entry ────────────────────────────────────────

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
            start_engine,
            stop_engine,
            set_system_proxy,
            // System data
            get_system_snapshot,
            get_processes,
            get_connections,
            get_traffic_stats,
            get_latency,
            get_proxy_state,
            get_network_info,
            // sing-box config
            get_singbox_config,
            update_singbox_config,
            // Speed test
            run_speed_test,
            run_speed_test_all,
            // Monitoring
            start_monitoring,
            stop_monitoring,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
