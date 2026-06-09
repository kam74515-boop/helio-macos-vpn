use crate::types::ProxyState;
use crate::utils::{run_cmd, run_cmd_stderr};

pub async fn get_proxy_state_impl() -> Result<ProxyState, String> {
    let service = primary_network_service();
    let http = run_cmd_stderr(&["networksetup", "-getwebproxy", &service]).unwrap_or_default();
    let socks = run_cmd_stderr(&["networksetup", "-getsocksfirewallproxy", &service]).unwrap_or_default();

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
        http_port: if http_port.is_empty() { "6152".to_string() } else { http_port },
        socks_host,
        socks_port: if socks_port.is_empty() { "6152".to_string() } else { socks_port },
    })
}

pub fn primary_network_service() -> String {
    let iface = run_cmd_stderr(&["route", "-n", "get", "default"])
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.trim().strip_prefix("interface:").map(|v| v.trim().to_string()))
        .unwrap_or_else(|| "en0".to_string());

    let ports = run_cmd(&["networksetup", "-listallhardwareports"]).unwrap_or_default();
    let mut current_service = String::new();
    for line in ports.lines() {
        if let Some(service) = line.strip_prefix("Hardware Port:") {
            current_service = service.trim().to_string();
        } else if let Some(device) = line.strip_prefix("Device:") {
            if device.trim() == iface && !current_service.is_empty() {
                return current_service;
            }
        }
    }

    "Wi-Fi".to_string()
}

pub fn extract_proxy_field(output: &str, field: &str) -> String {
    for line in output.lines() {
        if line.contains(field) {
            return line.split(':').last().unwrap_or("").trim().to_string();
        }
    }
    String::new()
}

#[tauri::command]
pub async fn get_proxy_state() -> Result<ProxyState, String> {
    get_proxy_state_impl().await
}

#[tauri::command]
pub async fn set_system_proxy(enable: bool) -> Result<(), String> {
    let service = primary_network_service();
    let state_str = if enable { "on" } else { "off" };
    let mut errs = Vec::new();

    if enable {
        if !run_networksetup(&["-setwebproxy", &service, "127.0.0.1", "6152"]) {
            errs.push("HTTP proxy config");
        }
        if !run_networksetup(&["-setsecurewebproxy", &service, "127.0.0.1", "6152"]) {
            errs.push("HTTPS proxy config");
        }
        if !run_networksetup(&["-setsocksfirewallproxy", &service, "127.0.0.1", "6152"]) {
            errs.push("SOCKS proxy config");
        }
    }

    if !run_networksetup(&["-setwebproxystate", &service, state_str]) {
        errs.push("HTTP proxy state");
    }
    if !run_networksetup(&["-setsecurewebproxystate", &service, state_str]) {
        errs.push("HTTPS proxy state");
    }
    if !run_networksetup(&["-setsocksfirewallproxystate", &service, state_str]) {
        errs.push("SOCKS proxy state");
    }

    if !errs.is_empty() {
        if enable {
            let _ = run_networksetup(&["-setwebproxystate", &service, "off"]);
            let _ = run_networksetup(&["-setsecurewebproxystate", &service, "off"]);
            let _ = run_networksetup(&["-setsocksfirewallproxystate", &service, "off"]);
        }
        return Err(format!("无法设置 {} 的以下代理项: {:?}", service, errs));
    }

    Ok(())
}

fn run_networksetup(args: &[&str]) -> bool {
    std::process::Command::new("networksetup")
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
