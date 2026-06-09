use crate::state::AppState;
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

pub async fn get_traffic_stats_impl(
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
            if parts.len() >= 10 {
                // netstat -ib columns: Name Mtu Network Address Ipkts Ierrs Ibytes Opkts Oerrs Obytes Coll
                // Use actual byte counts (Ibytes index 6, Obytes index 9) instead of packet estimates
                if let Ok(ibytes) = parts.get(6).unwrap_or(&"0").parse::<u64>() {
                    rx_bytes = ibytes;
                }
                if let Ok(obytes) = parts.get(9).unwrap_or(&"0").parse::<u64>() {
                    tx_bytes = obytes;
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

pub fn extract_bytes(line: &str) -> Option<u64> {
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
pub async fn get_traffic_stats(app: AppHandle, state: State<'_, AppState>) -> Result<TrafficStats, String> {
    get_traffic_stats_impl(app, state).await
}
