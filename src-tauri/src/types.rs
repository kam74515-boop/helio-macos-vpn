use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub connections: u32,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub icon_key: String,
    pub icon_base64: Option<String>,
    pub policy: String,
    pub last_address: String,
    pub dns_resolver: String,
    pub traffic_history: Vec<f64>,
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
    pub raw: serde_json::Value,
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
    pub config_name: String,
    pub mode: String,
    pub outbounds: Vec<SingboxOutbound>,
    pub rules: Vec<SingboxRule>,
    pub policy_groups: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManualOutboundInput {
    pub tag: String,
    pub outbound_type: String,
    pub server: Option<String>,
    pub server_port: Option<u16>,
    pub uuid: Option<String>,
    pub password: Option<String>,
    pub method: Option<String>,
    pub security: Option<String>,
    pub sni: Option<String>,
    pub raw_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectorGroupInput {
    pub tag: String,
    pub members: Vec<String>,
    pub default: Option<String>,
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
