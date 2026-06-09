use crate::types::{SpeedTestResult, SystemSnapshot};
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

#[tauri::command]
pub async fn get_system_snapshot(
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

pub async fn get_latency_impl(host: &str) -> LatencyResult {
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

pub fn parse_ping_avg(output: &str) -> Option<f64> {
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

pub fn parse_dig_time(output: &str) -> Option<f64> {
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

pub fn get_default_gateway() -> Option<String> {
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
pub async fn get_latency(host: String) -> Result<LatencyResult, String> {
    Ok(get_latency_impl(&host).await)
}

pub async fn get_network_info_impl() -> Result<NetworkInfo, String> {
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

pub fn get_ssid() -> String {
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

pub fn get_local_ip() -> String {
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

pub async fn get_external_ip() -> String {
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

pub fn get_active_iface() -> String {
    let out = run_cmd_stderr(&["route", "-n", "get", "default"]).unwrap_or_default();
    for line in out.lines() {
        if line.contains("interface:") {
            return line.split(':').last().unwrap_or("en0").trim().to_string();
        }
    }
    "en0".to_string()
}

#[tauri::command]
pub async fn get_network_info() -> Result<NetworkInfo, String> {
    get_network_info_impl().await
}

#[tauri::command]
pub async fn run_speed_test(node_name: String) -> Result<SpeedTestResult, String> {
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
pub async fn run_speed_test_all() -> Result<Vec<SpeedTestResult>, String> {
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
