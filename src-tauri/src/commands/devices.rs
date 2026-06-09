use serde::Serialize;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct LanDevice {
    pub ip: String,
    pub mac: Option<String>,
    pub name: Option<String>,
    pub interface: Option<String>,
}

#[tauri::command]
pub async fn get_lan_devices() -> Result<Vec<LanDevice>, String> {
    let output = match Command::new("arp").arg("-a").output() {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse macOS arp -a format:
        // ? (192.168.1.1) at ab:cd:ef:12:34:56 on en0 ifscope [ethernet]
        // hostname (192.168.1.1) at ab:cd:ef:12:34:56 on en0 [ethernet]
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        // Find the index of "(ip)"
        let mut name = None;
        let mut ip = None;
        let mut mac = None;
        let mut interface = None;

        for (i, part) in parts.iter().enumerate() {
            if part.starts_with('(') && part.ends_with(')') {
                // The part before this is the hostname (if not "?")
                if i > 0 && parts[i - 1] != "?" {
                    name = Some(parts[i - 1].to_string());
                }
                ip = Some(part.trim_start_matches('(').trim_end_matches(')').to_string());
            }
            if *part == "at" && i + 1 < parts.len() {
                let mac_candidate = parts[i + 1];
                if mac_candidate.contains(':') {
                    mac = Some(mac_candidate.to_string());
                }
            }
            if *part == "on" && i + 1 < parts.len() {
                interface = Some(parts[i + 1].to_string());
            }
        }

        if let Some(ip) = ip {
            devices.push(LanDevice {
                ip,
                mac,
                name,
                interface,
            });
        }
    }

    Ok(devices)
}
