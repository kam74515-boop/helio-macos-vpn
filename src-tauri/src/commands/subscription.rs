use base64::prelude::*;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

#[derive(Debug, Serialize, Clone)]
pub struct ImportResult {
    pub success: bool,
    pub message: String,
    pub imported_nodes: usize,
}

#[tauri::command]
pub async fn import_subscription(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> Result<ImportResult, String> {
    let raw = if looks_like_subscription_body(&url) {
        url
    } else {
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("订阅请求失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("订阅请求失败: HTTP {}", resp.status()));
        }

        resp.text()
            .await
            .map_err(|e| format!("订阅读取失败: {}", e))?
    };

    if looks_like_yaml(&raw) {
        return Err("当前导入器暂不支持 Clash/Mihomo YAML 订阅，请使用 URI/base64 节点订阅。".to_string());
    }

    let decoded = decode_subscription_body(&raw);
    let outbounds = parse_subscription_lines(&decoded)?;
    if outbounds.is_empty() {
        return Err("没有解析到可用节点。当前支持 vless、vmess、trojan、hysteria2/hy2、tuic、anytls、ss URI。".to_string());
    }

    let new_config = build_singbox_config(&outbounds);
    let config_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let config_path = config_dir.join("config.json");
    fs::write(&config_path, &new_config).map_err(|e| e.to_string())?;

    crate::commands::singbox::start_engine(app.clone(), state, Some(new_config)).await?;

    Ok(ImportResult {
        success: true,
        message: "订阅导入成功，已生成 sing-box 配置并重启内核".to_string(),
        imported_nodes: outbounds.len(),
    })
}

fn looks_like_subscription_body(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.contains('\n') || supported_scheme(trimmed).is_some()
}

fn looks_like_yaml(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.contains("proxies:") || lower.contains("proxy-groups:") || lower.contains("rules:")
}

fn decode_subscription_body(input: &str) -> String {
    let trimmed = input.trim();
    if supported_scheme(trimmed).is_some() || trimmed.contains('\n') {
        return trimmed.to_string();
    }

    decode_base64_text(trimmed).unwrap_or_else(|| trimmed.to_string())
}

fn parse_subscription_lines(input: &str) -> Result<Vec<Value>, String> {
    let mut outbounds = Vec::new();
    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(outbound) = parse_proxy_uri(line) {
            outbounds.push(outbound);
        }
    }
    Ok(outbounds)
}

fn build_singbox_config(nodes: &[Value]) -> String {
    let node_tags: Vec<String> = nodes
        .iter()
        .filter_map(|node| node.get("tag").and_then(Value::as_str).map(ToString::to_string))
        .collect();
    let default_tag = node_tags.first().cloned().unwrap_or_else(|| "direct".to_string());

    let mut outbounds = Vec::new();
    outbounds.extend(nodes.iter().cloned());
    outbounds.push(json!({
        "type": "selector",
        "tag": "Proxy",
        "outbounds": if node_tags.is_empty() { vec!["direct".to_string()] } else { node_tags },
        "default": default_tag
    }));
    outbounds.push(json!({"type": "direct", "tag": "direct"}));

    serde_json::to_string_pretty(&json!({
        "_helio": {"name": "Imported", "version": "0.1.0"},
        "log": {"level": "info", "timestamp": true},
        "inbounds": [{
            "type": "mixed",
            "tag": "mixed-in",
            "listen": "127.0.0.1",
            "listen_port": 6152
        }],
        "outbounds": outbounds,
        "route": {
            "auto_detect_interface": true,
            "final": "Proxy",
            "rules": [
                {"inbound": "mixed-in", "action": "sniff"},
                {"outbound": "direct", "domain_suffix": ["apple.com", "icloud.com"]}
            ]
        }
    }))
    .unwrap_or_else(|_| crate::commands::singbox::default_singbox_config())
}

fn parse_proxy_uri(line: &str) -> Option<Value> {
    match supported_scheme(line)? {
        "vless" => parse_standard_proxy(line, "vless"),
        "trojan" => parse_standard_proxy(line, "trojan"),
        "hysteria2" | "hy2" => parse_standard_proxy(line, "hysteria2"),
        "tuic" => parse_standard_proxy(line, "tuic"),
        "anytls" => parse_standard_proxy(line, "anytls"),
        "vmess" => parse_vmess(line),
        "ss" => parse_shadowsocks(line),
        _ => None,
    }
}

fn supported_scheme(line: &str) -> Option<&'static str> {
    let lower = line.trim().to_lowercase();
    [
        "vless://",
        "vmess://",
        "trojan://",
        "hysteria2://",
        "hy2://",
        "tuic://",
        "anytls://",
        "ss://",
    ]
    .iter()
    .find_map(|scheme| lower.starts_with(scheme).then_some(scheme.trim_end_matches("://")))
}

fn parse_standard_proxy(line: &str, outbound_type: &str) -> Option<Value> {
    let uri = split_uri(line)?;
    let tag = clean_tag(uri.tag.as_deref(), &uri.host, outbound_type);
    let mut obj = Map::new();
    obj.insert("type".to_string(), json!(outbound_type));
    obj.insert("tag".to_string(), json!(tag));
    obj.insert("server".to_string(), json!(uri.host));
    obj.insert("server_port".to_string(), json!(uri.port));

    let userinfo = percent_decode(&uri.userinfo);
    match outbound_type {
        "vless" => {
            if userinfo.is_empty() {
                return None;
            }
            obj.insert("uuid".to_string(), json!(userinfo));
            obj.insert("packet_encoding".to_string(), json!("xudp"));
            if let Some(flow) = uri.query.get("flow").filter(|value| !value.is_empty()) {
                obj.insert("flow".to_string(), json!(flow));
            }
        }
        "trojan" | "hysteria2" | "anytls" => {
            if userinfo.is_empty() {
                return None;
            }
            obj.insert("password".to_string(), json!(userinfo));
        }
        "tuic" => {
            let (uuid, password) = userinfo.split_once(':')?;
            obj.insert("uuid".to_string(), json!(uuid));
            obj.insert("password".to_string(), json!(password));
            obj.insert(
                "congestion_control".to_string(),
                json!(uri.query.get("congestion_control").map(String::as_str).unwrap_or("bbr")),
            );
        }
        _ => {}
    }

    if needs_tls(outbound_type, &uri.query) {
        obj.insert("tls".to_string(), build_tls(&uri.query, outbound_type == "vless"));
    }
    if let Some(transport) = build_transport(&uri.query) {
        obj.insert("transport".to_string(), transport);
    }

    Some(Value::Object(obj))
}

fn parse_vmess(line: &str) -> Option<Value> {
    let payload = line.trim().strip_prefix("vmess://")?;
    let decoded = decode_base64_text(payload)?;
    let val: Value = serde_json::from_str(&decoded).ok()?;
    let host = val.get("add")?.as_str()?.to_string();
    let port = val
        .get("port")
        .and_then(|value| value.as_str().and_then(|s| s.parse::<u16>().ok()).or_else(|| value.as_u64().map(|n| n as u16)))
        .unwrap_or(443);
    let tag = clean_tag(val.get("ps").and_then(Value::as_str), &host, "vmess");
    let uuid = val.get("id")?.as_str()?;

    let mut obj = Map::new();
    obj.insert("type".to_string(), json!("vmess"));
    obj.insert("tag".to_string(), json!(tag));
    obj.insert("server".to_string(), json!(host));
    obj.insert("server_port".to_string(), json!(port));
    obj.insert("uuid".to_string(), json!(uuid));
    obj.insert("security".to_string(), json!(val.get("scy").and_then(Value::as_str).unwrap_or("auto")));
    if let Some(aid) = val.get("aid").and_then(|value| value.as_str().and_then(|s| s.parse::<u16>().ok()).or_else(|| value.as_u64().map(|n| n as u16))) {
        obj.insert("alter_id".to_string(), json!(aid));
    }

    if val.get("tls").and_then(Value::as_str).is_some_and(|tls| tls == "tls") {
        let mut tls = Map::new();
        tls.insert("enabled".to_string(), json!(true));
        if let Some(sni) = val.get("sni").and_then(Value::as_str).or_else(|| val.get("host").and_then(Value::as_str)) {
            if !sni.is_empty() {
                tls.insert("server_name".to_string(), json!(sni));
            }
        }
        obj.insert("tls".to_string(), Value::Object(tls));
    }

    if let Some(net) = val.get("net").and_then(Value::as_str) {
        let mut query = BTreeMap::new();
        query.insert("type".to_string(), net.to_string());
        if let Some(path) = val.get("path").and_then(Value::as_str) {
            query.insert("path".to_string(), path.to_string());
        }
        if let Some(host) = val.get("host").and_then(Value::as_str) {
            query.insert("host".to_string(), host.to_string());
        }
        if let Some(transport) = build_transport(&query) {
            obj.insert("transport".to_string(), transport);
        }
    }

    Some(Value::Object(obj))
}

fn parse_shadowsocks(line: &str) -> Option<Value> {
    let without_scheme = line.trim().strip_prefix("ss://")?;
    let (body, tag) = split_fragment(without_scheme);
    let decoded_body = if body.contains('@') {
        body.to_string()
    } else {
        decode_base64_text(&body)?
    };
    let (userinfo, hostport) = decoded_body.rsplit_once('@')?;
    let userinfo = decode_base64_text(userinfo).unwrap_or_else(|| percent_decode(userinfo));
    let (method, password) = userinfo.split_once(':')?;
    let (host, port) = parse_host_port(hostport)?;

    Some(json!({
        "type": "shadowsocks",
        "tag": clean_tag(tag.as_deref(), &host, "ss"),
        "server": host,
        "server_port": port,
        "method": method,
        "password": password
    }))
}

struct ParsedUri {
    userinfo: String,
    host: String,
    port: u16,
    query: BTreeMap<String, String>,
    tag: Option<String>,
}

fn split_uri(line: &str) -> Option<ParsedUri> {
    let after_scheme = line.split_once("://")?.1;
    let (without_fragment, tag) = split_fragment(after_scheme);
    let (authority, query_str) = without_fragment.split_once('?').unwrap_or((&without_fragment, ""));
    let (userinfo, hostport) = authority.rsplit_once('@').unwrap_or(("", authority));
    let (host, port) = parse_host_port(hostport)?;
    Some(ParsedUri {
        userinfo: userinfo.to_string(),
        host,
        port,
        query: parse_query(query_str),
        tag,
    })
}

fn split_fragment(input: &str) -> (String, Option<String>) {
    if let Some((body, tag)) = input.split_once('#') {
        (body.to_string(), Some(percent_decode(tag)))
    } else {
        (input.to_string(), None)
    }
}

fn parse_host_port(input: &str) -> Option<(String, u16)> {
    if let Some(rest) = input.strip_prefix('[') {
        let (host, after_host) = rest.split_once(']')?;
        let port = after_host.strip_prefix(':')?.parse::<u16>().ok()?;
        return Some((host.to_string(), port));
    }
    let (host, port) = input.rsplit_once(':')?;
    Some((host.to_string(), port.parse::<u16>().ok()?))
}

fn parse_query(input: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for pair in input.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        map.insert(percent_decode(key), percent_decode(value));
    }
    map
}

fn needs_tls(outbound_type: &str, query: &BTreeMap<String, String>) -> bool {
    matches!(outbound_type, "trojan" | "hysteria2" | "tuic" | "anytls")
        || query.get("security").is_some_and(|value| value == "tls" || value == "reality")
        || query.contains_key("sni")
        || query.contains_key("serverName")
}

fn build_tls(query: &BTreeMap<String, String>, reality_capable: bool) -> Value {
    let mut tls = Map::new();
    tls.insert("enabled".to_string(), json!(true));
    if let Some(server_name) = query
        .get("sni")
        .or_else(|| query.get("serverName"))
        .or_else(|| query.get("peer"))
        .filter(|value| !value.is_empty())
    {
        tls.insert("server_name".to_string(), json!(server_name));
    }
    if query
        .get("allowInsecure")
        .or_else(|| query.get("insecure"))
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
    {
        tls.insert("insecure".to_string(), json!(true));
    }
    if reality_capable && query.get("security").is_some_and(|value| value == "reality") {
        let mut reality = Map::new();
        reality.insert("enabled".to_string(), json!(true));
        if let Some(public_key) = query.get("pbk").filter(|value| !value.is_empty()) {
            reality.insert("public_key".to_string(), json!(public_key));
        }
        if let Some(short_id) = query.get("sid").filter(|value| !value.is_empty()) {
            reality.insert("short_id".to_string(), json!(short_id));
        }
        tls.insert("reality".to_string(), Value::Object(reality));
    }
    Value::Object(tls)
}

fn build_transport(query: &BTreeMap<String, String>) -> Option<Value> {
    match query.get("type").map(String::as_str) {
        Some("ws") => {
            let mut transport = Map::new();
            transport.insert("type".to_string(), json!("ws"));
            if let Some(path) = query.get("path").filter(|value| !value.is_empty()) {
                transport.insert("path".to_string(), json!(path));
            }
            if let Some(host) = query.get("host").filter(|value| !value.is_empty()) {
                transport.insert("headers".to_string(), json!({"Host": host}));
            }
            Some(Value::Object(transport))
        }
        Some("grpc") => {
            let mut transport = Map::new();
            transport.insert("type".to_string(), json!("grpc"));
            if let Some(service_name) = query.get("serviceName").filter(|value| !value.is_empty()) {
                transport.insert("service_name".to_string(), json!(service_name));
            }
            Some(Value::Object(transport))
        }
        _ => None,
    }
}

fn clean_tag(tag: Option<&str>, host: &str, fallback: &str) -> String {
    let candidate = tag.unwrap_or("").trim();
    if candidate.is_empty() {
        format!("{}-{}", fallback, host)
    } else {
        candidate.to_string()
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&input[index + 1..index + 3], 16) {
                output.push(hex);
                index += 3;
                continue;
            }
        }
        output.push(if bytes[index] == b'+' { b' ' } else { bytes[index] });
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn decode_base64_text(input: &str) -> Option<String> {
    let compact = input.trim().replace(['\r', '\n'], "");
    let padded = match compact.len() % 4 {
        0 => compact,
        missing => format!("{}{}", compact, "=".repeat(4 - missing)),
    };
    BASE64_STANDARD
        .decode(padded.as_bytes())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vless_reality_uri() {
        let outbound = parse_proxy_uri(
            "vless://123e4567-e89b-12d3-a456-426614174000@example.com:443?security=reality&sni=www.apple.com&pbk=abc&sid=12&type=grpc&serviceName=edge#Reality%20Node",
        )
        .expect("valid vless");

        assert_eq!(outbound["type"], "vless");
        assert_eq!(outbound["tag"], "Reality Node");
        assert_eq!(outbound["server"], "example.com");
        assert_eq!(outbound["server_port"], 443);
        assert_eq!(outbound["uuid"], "123e4567-e89b-12d3-a456-426614174000");
        assert_eq!(outbound["tls"]["server_name"], "www.apple.com");
        assert_eq!(outbound["tls"]["reality"]["enabled"], true);
        assert_eq!(outbound["transport"]["type"], "grpc");
    }

    #[test]
    fn parses_vmess_json_uri() {
        let payload = BASE64_STANDARD.encode(
            r#"{"v":"2","ps":"VMess Node","add":"vm.example.com","port":"8443","id":"123e4567-e89b-12d3-a456-426614174000","aid":"0","net":"ws","type":"none","host":"cdn.example.com","path":"/ws","tls":"tls","sni":"vm.example.com"}"#,
        );
        let outbound = parse_proxy_uri(&format!("vmess://{}", payload)).expect("valid vmess");

        assert_eq!(outbound["type"], "vmess");
        assert_eq!(outbound["tag"], "VMess Node");
        assert_eq!(outbound["server"], "vm.example.com");
        assert_eq!(outbound["server_port"], 8443);
        assert_eq!(outbound["transport"]["type"], "ws");
        assert_eq!(outbound["tls"]["enabled"], true);
    }

    #[test]
    fn parses_shadowsocks_uri() {
        let userinfo = BASE64_STANDARD.encode("2022-blake3-aes-128-gcm:password");
        let outbound = parse_proxy_uri(&format!("ss://{}@ss.example.com:8388#SS%20Node", userinfo))
            .expect("valid shadowsocks");

        assert_eq!(outbound["type"], "shadowsocks");
        assert_eq!(outbound["tag"], "SS Node");
        assert_eq!(outbound["server"], "ss.example.com");
        assert_eq!(outbound["server_port"], 8388);
        assert_eq!(outbound["method"], "2022-blake3-aes-128-gcm");
    }
}
