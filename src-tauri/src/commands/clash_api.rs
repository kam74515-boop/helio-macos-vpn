use serde::{Deserialize, Serialize};
use serde_json::Value;

const CLASH_API_BASE: &str = "http://127.0.0.1:9090";

fn api_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default()
}

fn api_unavailable() -> String {
    "Clash API 不可用：sing-box 未启动或未启用 experimental.clash_api".to_string()
}

// ── Data structures ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClashProxy {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    #[serde(default)]
    pub history: Vec<ProxyDelay>,
    #[serde(default)]
    pub all: Vec<String>,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(default)]
    pub alive: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyDelay {
    pub time: u64,
    #[serde(default)]
    pub delay: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClashConnection {
    pub id: String,
    pub metadata: ConnectionMetadata,
    #[serde(default)]
    pub upload: u64,
    #[serde(default)]
    pub download: u64,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub chains: Vec<String>,
    #[serde(default)]
    pub rule: String,
    #[serde(default, rename = "rulePayload")]
    pub rule_payload: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConnectionMetadata {
    #[serde(default)]
    pub host: String,
    #[serde(default, rename = "destinationIP")]
    pub dst_ip: String,
    #[serde(default, rename = "destinationPort")]
    pub dst_port: String,
    #[serde(default)]
    pub network: String,
    #[serde(default, rename = "type")]
    pub conn_type: String,
    #[serde(default, rename = "sourceIP")]
    pub source_ip: String,
    #[serde(default, rename = "sourcePort")]
    pub source_port: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ClashTraffic {
    #[serde(default)]
    pub up: u64,
    #[serde(default)]
    pub down: u64,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_clash_proxies() -> Result<Vec<ClashProxy>, String> {
    let client = api_client();
    let url = format!("{}/proxies", CLASH_API_BASE);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{} (请求失败: {})", api_unavailable(), e))?;

    if !resp.status().is_success() {
        return Err(format!("Clash API 返回错误状态: {}", resp.status()));
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 proxies 响应失败: {}", e))?;

    let proxies_obj = body
        .get("proxies")
        .and_then(Value::as_object)
        .ok_or_else(|| "proxies 响应缺少 proxies 对象".to_string())?;

    let mut proxies = Vec::new();
    for (name, value) in proxies_obj {
        let mut proxy: ClashProxy = serde_json::from_value(value.clone())
            .map_err(|e| format!("解析 proxy {} 失败: {}", name, e))?;
        proxy.name = name.clone();
        // If alive is not present, infer from history or type
        if !value.get("alive").is_some() {
            proxy.alive = !proxy.history.is_empty() && proxy.history.iter().any(|h| h.delay > 0);
        }
        proxies.push(proxy);
    }

    Ok(proxies)
}

#[tauri::command]
pub async fn get_clash_connections() -> Result<Vec<ClashConnection>, String> {
    let client = api_client();
    let url = format!("{}/connections", CLASH_API_BASE);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{} (请求失败: {})", api_unavailable(), e))?;

    if !resp.status().is_success() {
        return Err(format!("Clash API 返回错误状态: {}", resp.status()));
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 connections 响应失败: {}", e))?;

    let connections = body
        .get("connections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut result = Vec::new();
    for item in connections {
        let conn: ClashConnection = serde_json::from_value(item)
            .map_err(|e| format!("解析 connection 失败: {}", e))?;
        result.push(conn);
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_clash_traffic() -> Result<ClashTraffic, String> {
    let client = api_client();
    let url = format!("{}/traffic", CLASH_API_BASE);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{} (请求失败: {})", api_unavailable(), e))?;

    if !resp.status().is_success() {
        return Err(format!("Clash API 返回错误状态: {}", resp.status()));
    }

    // /traffic is an SSE stream. Use a short timeout to avoid hanging forever.
    let text = match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        resp.text(),
    ).await {
        Ok(Ok(text)) => text,
        Ok(Err(e)) => return Err(format!("读取 traffic 流失败: {}", e)),
        Err(_) => return Ok(ClashTraffic::default()), // timeout: return zero
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("data:") {
            let payload = trimmed.strip_prefix("data:").unwrap_or(trimmed).trim();
            if payload.is_empty() || payload == "{}" {
                continue;
            }
            if let Ok(traffic) = serde_json::from_str::<ClashTraffic>(payload) {
                return Ok(traffic);
            }
        }
        if trimmed.starts_with('{') {
            if let Ok(traffic) = serde_json::from_str::<ClashTraffic>(trimmed) {
                return Ok(traffic);
            }
        }
    }

    Ok(ClashTraffic::default())
}

#[tauri::command]
pub async fn test_proxy_latency(proxy_name: String) -> Result<u64, String> {
    let client = api_client();
    let encoded = urlencoding::encode(&proxy_name);
    let url = format!(
        "{}/proxies/{}/delay?url=http://www.gstatic.com/generate_204&timeout=5000",
        CLASH_API_BASE, encoded
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{} (请求失败: {})", api_unavailable(), e))?;

    if !resp.status().is_success() {
        return Err(format!("Clash API 返回错误状态: {}", resp.status()));
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 delay 响应失败: {}", e))?;

    let delay = body
        .get("delay")
        .and_then(Value::as_u64)
        .ok_or_else(|| "delay 响应中缺少 delay 字段".to_string())?;

    Ok(delay)
}

#[tauri::command]
pub async fn close_connection(connection_id: String) -> Result<(), String> {
    let client = api_client();
    let url = format!("{}/connections/{}", CLASH_API_BASE, urlencoding::encode(&connection_id));

    let resp = client
        .delete(&url)
        .send()
        .await
        .map_err(|e| format!("{} (请求失败: {})", api_unavailable(), e))?;

    if !resp.status().is_success() {
        return Err(format!("关闭连接失败，状态: {}", resp.status()));
    }

    Ok(())
}

#[tauri::command]
pub async fn clear_connections() -> Result<(), String> {
    let client = api_client();
    let url = format!("{}/connections", CLASH_API_BASE);

    let resp = client
        .delete(&url)
        .send()
        .await
        .map_err(|e| format!("{} (请求失败: {})", api_unavailable(), e))?;

    if !resp.status().is_success() {
        return Err(format!("关闭所有连接失败，状态: {}", resp.status()));
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_capture(enabled: bool) -> Result<bool, String> {
    // Capture service is not yet implemented. Return the requested state
    // so the UI can toggle, but note that real capture requires MITM + proxy.
    if enabled {
        return Err("流量捕获需要启用系统代理或增强模式，并配置 MITM CA 证书。功能开发中。".to_string());
    }
    Ok(false)
}
