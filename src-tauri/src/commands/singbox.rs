use tauri::{AppHandle, State, Manager, Emitter};
use tauri_plugin_shell::ShellExt;
use std::path::PathBuf;
use crate::commands::traffic::get_traffic_stats_impl;
use std::fs;
use std::io::Write;
use crate::state::AppState;
use crate::types::{
    ManualOutboundInput, SelectorGroupInput, SingboxConfig, SingboxOutbound, SingboxRule,
};
use crate::config_store::{
    ConfigStore, Profile, refresh_proxy_selector_in_profile, update_profile_from_runtime,
    is_reserved_node_tag, build_runtime_config,
};
use serde_json::{json, Map, Value};

#[tauri::command]
pub async fn get_singbox_config_json(app: AppHandle) -> Result<serde_json::Value, String> {
    let store = app.state::<ConfigStore>();
    let profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;
    let runtime = store.get_runtime_config(&profile);
    let val: serde_json::Value = serde_json::from_str(&runtime)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    Ok(val)
}

#[tauri::command]
pub async fn get_singbox_config(app: AppHandle) -> Result<SingboxConfig, String> {
    let store = app.state::<ConfigStore>();
    let profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;
    let runtime = store.get_runtime_config(&profile);
    let val: serde_json::Value = serde_json::from_str(&runtime)
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
                raw: ob.clone(),
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

    let mode = detect_mode(&val);
    let config_name = val.get("_helio")
        .and_then(|h| h.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Default")
        .to_string();

    let policy_groups = collect_policy_groups(&val);

    Ok(SingboxConfig { config_name, mode, outbounds, rules, policy_groups })
}

fn detect_mode(config: &serde_json::Value) -> String {
    let route = config.get("route").and_then(|v| v.as_object());
    let final_outbound = route
        .and_then(|r| r.get("final"))
        .and_then(|v| v.as_str())
        .unwrap_or("direct");
    let auto_detect = route
        .and_then(|r| r.get("auto_detect_interface"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if final_outbound == "direct" {
        "direct".to_string()
    } else if !auto_detect {
        "global".to_string()
    } else {
        "rule".to_string()
    }
}

#[tauri::command]
pub async fn set_proxy_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: String,
) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    match mode.as_str() {
        "direct" => profile.mode = "direct".to_string(),
        "global" => profile.mode = "global".to_string(),
        "rule" => profile.mode = "rule".to_string(),
        _ => return Err(format!("未知模式: {}", mode)),
    }

    store.save_profile(&profile)?;

    let runtime = store.get_runtime_config(&profile);
    start_engine(app, state, Some(runtime)).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_proxy_mode(app: AppHandle) -> Result<String, String> {
    let store = app.state::<ConfigStore>();
    let profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;
    Ok(profile.mode.clone())
}

#[tauri::command]
pub async fn save_outbound(
    app: AppHandle,
    state: State<'_, AppState>,
    outbound: ManualOutboundInput,
) -> Result<SingboxConfig, String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let outbound_value = build_manual_outbound(outbound)?;
    let tag = outbound_value
        .get("tag")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "节点 tag 不能为空".to_string())?
        .to_string();

    if is_reserved_node_tag(&tag) {
        return Err(format!("{} 是系统保留 tag，不能作为代理节点", tag));
    }

    // Upsert into profile.outbounds
    profile.outbounds.retain(|item| item.get("tag").and_then(Value::as_str) != Some(&tag));
    profile.outbounds.push(outbound_value);

    refresh_proxy_selector_in_profile(&mut profile)?;
    store.save_profile(&profile)?;

    let runtime = store.get_runtime_config(&profile);
    start_engine(app.clone(), state, Some(runtime)).await?;
    get_singbox_config(app).await
}

#[tauri::command]
pub async fn delete_outbound(
    app: AppHandle,
    state: State<'_, AppState>,
    tag: String,
) -> Result<SingboxConfig, String> {
    let tag = tag.trim();
    if tag.is_empty() {
        return Err("节点 tag 不能为空".to_string());
    }
    if is_reserved_node_tag(tag) {
        return Err(format!("{} 是系统保留 tag，不能删除", tag));
    }

    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let before = profile.outbounds.len();
    profile.outbounds.retain(|item| item.get("tag").and_then(Value::as_str) != Some(tag));
    if profile.outbounds.len() == before {
        return Err(format!("没有找到节点 {}", tag));
    }

    // Remove from selectors
    for selector in &mut profile.selectors {
        selector.outbounds.retain(|member| member != tag);
        if selector.outbounds.is_empty() {
            selector.outbounds.push("direct".to_string());
        }
        if !selector.outbounds.contains(&selector.default) {
            selector.default = selector.outbounds.first().cloned().unwrap_or_else(|| "direct".to_string());
        }
    }

    refresh_proxy_selector_in_profile(&mut profile)?;
    store.save_profile(&profile)?;

    let runtime = store.get_runtime_config(&profile);
    start_engine(app.clone(), state, Some(runtime)).await?;
    get_singbox_config(app).await
}

#[tauri::command]
pub async fn save_selector_group(
    app: AppHandle,
    state: State<'_, AppState>,
    group: SelectorGroupInput,
) -> Result<SingboxConfig, String> {
    let tag = group.tag.trim().to_string();
    if tag.is_empty() {
        return Err("策略组名称不能为空".to_string());
    }
    if matches!(tag.as_str(), "direct" | "block") {
        return Err(format!("{} 是系统保留 tag，不能作为策略组", tag));
    }

    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let available_nodes: Vec<String> = profile.outbounds.iter()
        .filter(|item| {
            let ob_type = item.get("type").and_then(Value::as_str).unwrap_or("");
            !matches!(ob_type, "selector" | "direct" | "block" | "dns")
        })
        .filter_map(|item| item.get("tag").and_then(Value::as_str).map(String::from))
        .collect();

    let mut members: Vec<String> = group
        .members
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty() && (item == "direct" || available_nodes.contains(item)))
        .collect();

    members.sort();
    members.dedup();
    if members.is_empty() {
        members = available_nodes;
    }
    if !members.iter().any(|item| item == "direct") {
        members.push("direct".to_string());
    }

    let default = group
        .default
        .filter(|item| members.contains(item))
        .unwrap_or_else(|| members.first().cloned().unwrap_or_else(|| "direct".to_string()));

    // Upsert selector
    if let Some(existing) = profile.selectors.iter_mut().find(|s| s.tag == tag) {
        existing.outbounds = members;
        existing.default = default;
    } else {
        profile.selectors.push(crate::config_store::SelectorGroup {
            tag,
            group_type: "selector".to_string(),
            outbounds: members,
            default,
            url: None,
            interval: None,
        });
    }

    refresh_proxy_selector_in_profile(&mut profile)?;
    store.save_profile(&profile)?;

    let runtime = store.get_runtime_config(&profile);
    start_engine(app.clone(), state, Some(runtime)).await?;
    get_singbox_config(app).await
}

#[tauri::command]
pub async fn switch_proxy(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    proxy: String,
) -> Result<SingboxConfig, String> {
    set_selector_default(app, state, group, proxy).await
}

#[tauri::command]
pub async fn get_proxy_config(app: AppHandle) -> Result<serde_json::Value, String> {
    let store = app.state::<ConfigStore>();
    let profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let mut http_port = None;
    let mut socks_port = None;
    let mut http_listen = "127.0.0.1".to_string();
    let mut socks_listen = "127.0.0.1".to_string();

    for inbound in &profile.inbounds {
        let t = inbound.get("type").and_then(Value::as_str).unwrap_or("");
        if t == "mixed" || t == "http" {
            http_port = inbound.get("listen_port").and_then(Value::as_u64).map(|n| n as u16);
            if http_port.is_none() {
                http_port = inbound.get("port").and_then(Value::as_u64).map(|n| n as u16);
            }
            if let Some(listen) = inbound.get("listen").and_then(Value::as_str) {
                http_listen = listen.to_string();
            }
        }
        if t == "socks" || t == "mixed" {
            socks_port = inbound.get("listen_port").and_then(Value::as_u64).map(|n| n as u16);
            if socks_port.is_none() {
                socks_port = inbound.get("port").and_then(Value::as_u64).map(|n| n as u16);
            }
            if let Some(listen) = inbound.get("listen").and_then(Value::as_str) {
                socks_listen = listen.to_string();
            }
        }
    }

    // Fallback to defaults if no inbounds configured
    if http_port.is_none() && socks_port.is_none() {
        http_port = Some(6152);
        socks_port = Some(6152);
    }

    Ok(serde_json::json!({
        "http_port": http_port,
        "socks_port": socks_port,
        "http_listen": http_listen,
        "socks_listen": socks_listen,
    }))
}

#[tauri::command]
pub async fn set_selector_default(
    app: AppHandle,
    state: State<'_, AppState>,
    group_tag: String,
    target_tag: String,
) -> Result<SingboxConfig, String> {
    let group_tag = group_tag.trim();
    let target_tag = target_tag.trim();
    if group_tag.is_empty() || target_tag.is_empty() {
        return Err("策略组和目标节点不能为空".to_string());
    }

    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let group = profile.selectors.iter_mut()
        .find(|s| s.tag == group_tag)
        .ok_or_else(|| format!("没有找到策略组 {}", group_tag))?;

    if !group.outbounds.contains(&target_tag.to_string()) {
        return Err(format!("{} 不在策略组 {} 中", target_tag, group_tag));
    }

    group.default = target_tag.to_string();
    store.save_profile(&profile)?;

    let runtime = store.get_runtime_config(&profile);
    start_engine(app.clone(), state, Some(runtime)).await?;
    get_singbox_config(app).await
}

fn config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.json")
}

fn read_config_value(app: &AppHandle) -> Result<Value, String> {
    let path = config_path(app);
    let content = fs::read_to_string(&path).unwrap_or_else(|_| default_singbox_config());
    let mut config: Value = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    normalize_config(&mut config);
    Ok(config)
}

async fn write_config_and_restart(
    app: &AppHandle,
    state: State<'_, AppState>,
    config: &Value,
) -> Result<(), String> {
    let config_path = config_path(app);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let updated = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&config_path, &updated).map_err(|e| e.to_string())?;
    start_engine(app.clone(), state, Some(updated)).await
}

fn normalize_config(config: &mut Value) {
    let default: Value = serde_json::from_str(&default_singbox_config()).unwrap_or_else(|_| json!({}));
    if !config.is_object() {
        *config = default;
        return;
    }
    if !config.get("inbounds").is_some_and(Value::is_array) {
        config["inbounds"] = default["inbounds"].clone();
    }
    if !config.get("outbounds").is_some_and(Value::is_array) {
        config["outbounds"] = default["outbounds"].clone();
    }
    if !config.get("route").is_some_and(Value::is_object) {
        config["route"] = default["route"].clone();
    }
    if config.get("_helio").is_none() {
        config["_helio"] = json!({"name": "Default", "version": "0.1.0"});
    }

    if let Some(outbounds) = config.get_mut("outbounds").and_then(Value::as_array_mut) {
        let has_direct = outbounds.iter().any(|item| {
            item.get("type").and_then(Value::as_str) == Some("direct")
                && item.get("tag").and_then(Value::as_str) == Some("direct")
        });
        if !has_direct {
            outbounds.push(json!({"type": "direct", "tag": "direct"}));
        }
        let has_proxy = outbounds.iter().any(|item| {
            item.get("type").and_then(Value::as_str) == Some("selector")
                && item.get("tag").and_then(Value::as_str) == Some("Proxy")
        });
        if !has_proxy {
            outbounds.insert(0, json!({
                "type": "selector",
                "tag": "Proxy",
                "outbounds": ["direct"],
                "default": "direct"
            }));
        }
    }

    if config["route"].get("rules").is_none() {
        config["route"]["rules"] = json!([
            {"inbound": "mixed-in", "action": "sniff"},
            {"outbound": "direct", "domain_suffix": ["apple.com", "icloud.com"]}
        ]);
    }
    if config["route"].get("final").is_none() {
        config["route"]["final"] = json!("Proxy");
    }
    if config["route"].get("auto_detect_interface").is_none() {
        config["route"]["auto_detect_interface"] = json!(true);
    }
}

fn outbounds_mut(config: &mut Value) -> Result<&mut Vec<Value>, String> {
    config
        .get_mut("outbounds")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "sing-box 配置缺少 outbounds 数组".to_string())
}

fn build_manual_outbound(input: ManualOutboundInput) -> Result<Value, String> {
    if let Some(raw) = input.raw_json.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        let mut value: Value = serde_json::from_str(raw)
            .map_err(|e| format!("高级 JSON 解析失败: {}", e))?;
        if !value.is_object() {
            return Err("高级 JSON 必须是单个 outbound 对象".to_string());
        }
        let tag = input.tag.trim();
        if !tag.is_empty() {
            value["tag"] = json!(tag);
        }
        if value.get("type").and_then(Value::as_str).is_none() {
            value["type"] = json!(input.outbound_type.trim());
        }
        return Ok(value);
    }

    let tag = input.tag.trim();
    if tag.is_empty() {
        return Err("节点名称不能为空".to_string());
    }
    let outbound_type = normalize_outbound_type(&input.outbound_type);
    if outbound_type.is_empty() {
        return Err("节点类型不能为空".to_string());
    }
    let server = required_text(input.server.as_deref(), "服务器地址")?;
    let server_port = input.server_port.ok_or_else(|| "服务器端口不能为空".to_string())?;

    let mut obj = Map::new();
    obj.insert("type".to_string(), json!(outbound_type));
    obj.insert("tag".to_string(), json!(tag));
    obj.insert("server".to_string(), json!(server));
    obj.insert("server_port".to_string(), json!(server_port));

    match outbound_type {
        "vless" | "vmess" => {
            obj.insert("uuid".to_string(), json!(required_text(input.uuid.as_deref(), "UUID")?));
            if outbound_type == "vless" {
                obj.insert("packet_encoding".to_string(), json!("xudp"));
            }
            if outbound_type == "vmess" {
                obj.insert(
                    "security".to_string(),
                    json!(input.security.as_deref().filter(|value| !value.trim().is_empty()).unwrap_or("auto")),
                );
            }
        }
        "trojan" | "hysteria2" | "anytls" => {
            obj.insert("password".to_string(), json!(required_text(input.password.as_deref(), "密码")?));
        }
        "tuic" => {
            obj.insert("uuid".to_string(), json!(required_text(input.uuid.as_deref(), "UUID")?));
            obj.insert("password".to_string(), json!(required_text(input.password.as_deref(), "密码")?));
            obj.insert("congestion_control".to_string(), json!("bbr"));
        }
        "shadowsocks" => {
            obj.insert(
                "method".to_string(),
                json!(input.method.as_deref().filter(|value| !value.trim().is_empty()).unwrap_or("2022-blake3-aes-128-gcm")),
            );
            obj.insert("password".to_string(), json!(required_text(input.password.as_deref(), "密码")?));
        }
        other => return Err(format!("暂不支持手动创建 {} 节点，可用高级 JSON 添加", other)),
    }

    let security = input.security.unwrap_or_default();
    let sni = input.sni.unwrap_or_default();
    if security == "tls" || security == "reality" || !sni.trim().is_empty() {
        let mut tls = Map::new();
        tls.insert("enabled".to_string(), json!(true));
        if !sni.trim().is_empty() {
            tls.insert("server_name".to_string(), json!(sni.trim()));
        }
        if security == "reality" {
            tls.insert("reality".to_string(), json!({"enabled": true}));
        }
        obj.insert("tls".to_string(), Value::Object(tls));
    }

    Ok(Value::Object(obj))
}

fn required_text(value: Option<&str>, label: &str) -> Result<String, String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("{}不能为空", label))
}

fn normalize_outbound_type(value: &str) -> &'static str {
    match value.trim().to_lowercase().as_str() {
        "ss" | "shadowsocks" => "shadowsocks",
        "hy2" | "hysteria2" => "hysteria2",
        "vless" => "vless",
        "vmess" => "vmess",
        "trojan" => "trojan",
        "tuic" => "tuic",
        "anytls" => "anytls",
        _ => "",
    }
}

fn upsert_outbound_value(config: &mut Value, outbound: Value) -> Result<(), String> {
    let tag = outbound
        .get("tag")
        .and_then(Value::as_str)
        .ok_or_else(|| "outbound 缺少 tag".to_string())?
        .to_string();

    let outbounds = outbounds_mut(config)?;
    outbounds.retain(|item| item.get("tag").and_then(Value::as_str) != Some(tag.as_str()));

    let insert_index = outbounds
        .iter()
        .position(|item| item.get("tag").and_then(Value::as_str) == Some("direct"))
        .unwrap_or(outbounds.len());
    outbounds.insert(insert_index, outbound);
    Ok(())
}

fn remove_member_from_selectors(config: &mut Value, tag: &str) -> Result<(), String> {
    let outbounds = outbounds_mut(config)?;
    for item in outbounds.iter_mut().filter(|item| item.get("type").and_then(Value::as_str) == Some("selector")) {
        let current_default = item.get("default").and_then(Value::as_str).map(ToString::to_string);
        if let Some(members) = item.get_mut("outbounds").and_then(Value::as_array_mut) {
            members.retain(|member| member.as_str() != Some(tag));
            if members.is_empty() {
                members.push(json!("direct"));
            }
            let default_missing = match current_default.as_deref() {
                Some(default) => !members.iter().any(|member| member.as_str() == Some(default)),
                None => true,
            };
            if default_missing {
                item["default"] = members.first().cloned().unwrap_or_else(|| json!("direct"));
            }
        }
    }
    Ok(())
}

fn refresh_proxy_selector(config: &mut Value) -> Result<(), String> {
    let mut members = collect_node_tags(config);
    members.extend(collect_selector_tags(config).into_iter().filter(|tag| tag != "Proxy"));
    if !members.iter().any(|tag| tag == "direct") {
        members.push("direct".to_string());
    }
    let mut deduped = Vec::new();
    for member in members {
        if !deduped.contains(&member) {
            deduped.push(member);
        }
    }
    let mut members = deduped;
    if members.is_empty() {
        members.push("direct".to_string());
    }

    let outbounds = outbounds_mut(config)?;
    let Some(proxy) = outbounds.iter_mut().find(|item| {
        item.get("type").and_then(Value::as_str) == Some("selector")
            && item.get("tag").and_then(Value::as_str) == Some("Proxy")
    }) else {
        return Err("缺少 Proxy 策略组".to_string());
    };

    let current_default = proxy.get("default").and_then(Value::as_str).map(ToString::to_string);
    let next_default = current_default
        .filter(|value| members.contains(value))
        .unwrap_or_else(|| members.first().cloned().unwrap_or_else(|| "direct".to_string()));
    proxy["outbounds"] = json!(members);
    proxy["default"] = json!(next_default);
    Ok(())
}

fn collect_node_tags(config: &Value) -> Vec<String> {
    config
        .get("outbounds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| {
            let outbound_type = item.get("type").and_then(Value::as_str).unwrap_or("");
            let tag = item.get("tag").and_then(Value::as_str).unwrap_or("");
            !matches!(outbound_type, "selector" | "direct" | "block" | "dns")
                && !is_reserved_node_tag(tag)
        })
        .filter_map(|item| item.get("tag").and_then(Value::as_str).map(ToString::to_string))
        .collect()
}

fn collect_selector_tags(config: &Value) -> Vec<String> {
    config
        .get("outbounds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("selector"))
        .filter_map(|item| item.get("tag").and_then(Value::as_str).map(ToString::to_string))
        .collect()
}

fn collect_policy_groups(config: &Value) -> Vec<Value> {
    config
        .get("outbounds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("selector"))
        .map(|item| {
            let members: Vec<String> = item
                .get("outbounds")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|member| member.as_str().map(ToString::to_string))
                .collect();
            json!({
                "name": item.get("tag").and_then(Value::as_str).unwrap_or("Proxy"),
                "mode": "手动选择策略组",
                "members": members.len(),
                "memberTags": members,
                "selected": item.get("default").and_then(Value::as_str).unwrap_or("direct")
            })
        })
        .collect()
}

pub fn default_singbox_config() -> String {
    r#"{
  "log": {"level": "info", "timestamp": true},
  "inbounds": [
    {
      "type": "mixed",
      "tag": "mixed-in",
      "listen": "127.0.0.1",
      "listen_port": 6152
    }
  ],
  "outbounds": [
    {"type": "selector", "tag": "Proxy", "outbounds": ["direct"], "default": "direct"},
    {"type": "direct", "tag": "direct"}
  ],
  "route": {
    "auto_detect_interface": true,
    "final": "Proxy",
    "rules": [
      {"inbound": "mixed-in", "action": "sniff"},
      {"outbound": "direct", "domain_suffix": ["apple.com", "icloud.com"]}
    ]
  },
  "experimental": {
    "clash_api": {
      "external_controller": "127.0.0.1:9090",
      "secret": ""
    }
  }
}"#.to_string()
}

#[tauri::command]
pub async fn update_singbox_config(app: AppHandle, state: State<'_, AppState>, config: String) -> Result<(), String> {
    let store = app.state::<ConfigStore>();

    // Validate JSON
    let config_val: serde_json::Value = serde_json::from_str(&config)
        .map_err(|e| format!("无效的配置 JSON: {}", e))?;

    // Try to update active profile from the runtime JSON
    let mut profile = store.get_active_profile()
        .unwrap_or_else(|| {
            let mut p = Profile::default();
            p.name = "Default".to_string();
            p
        });

    update_profile_from_runtime(&mut profile, &config_val);
    store.save_profile(&profile)?;

    // Write runtime config for sing-box sidecar
    store.write_runtime_config(&config)?;

    // Restart engine with new config
    let mut current = state.engine_process.lock().unwrap();
    if let Some(child) = current.take() {
        let _ = child.kill();
    }

    // Relaunch
    drop(current);
    let config_path = store.get_runtime_config_path();
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
                    log::info!("sing-box: {:?}", String::from_utf8_lossy(&line));
                }
                _ => {}
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn start_engine(app: AppHandle, state: State<'_, AppState>, config: Option<String>) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    let config_content = match config {
        Some(value) if !value.trim().is_empty() => value,
        _ => {
            if let Some(profile) = store.get_active_profile() {
                store.get_runtime_config(&profile)
            } else {
                default_singbox_config()
            }
        }
    };

    let _: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("无效的 sing-box 配置 JSON: {}", e))?;

    store.write_runtime_config(&config_content)?;
    let config_path = store.get_runtime_config_path();

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
                    log::info!("sing-box: {:?}", String::from_utf8_lossy(&line));
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn has_runtime_config_shape(content: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return false;
    };
    value.get("inbounds").is_some_and(|v| v.is_array())
        && value.get("outbounds").is_some_and(|v| v.is_array())
        && value.get("route").is_some_and(|v| v.is_object())
}

#[tauri::command]
pub async fn stop_engine(state: State<'_, AppState>) -> Result<(), String> {
    let mut current = state.engine_process.lock().unwrap();
    if let Some(child) = current.take() {
        let _ = child.kill();
    }
    Ok(())
}

#[tauri::command]
pub async fn start_monitoring(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
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
            let should_continue = {
                let state = app_handle.state::<AppState>();
                let monitoring = state.monitoring.lock().unwrap();
                *monitoring
            };
            if !should_continue {
                break;
            }

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
pub async fn stop_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    let mut is_monitoring = state.monitoring.lock().unwrap();
    *is_monitoring = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_input(tag: &str, outbound_type: &str) -> ManualOutboundInput {
        ManualOutboundInput {
            tag: tag.to_string(),
            outbound_type: outbound_type.to_string(),
            server: Some("example.com".to_string()),
            server_port: Some(443),
            uuid: Some("123e4567-e89b-12d3-a456-426614174000".to_string()),
            password: Some("password".to_string()),
            method: None,
            security: Some("tls".to_string()),
            sni: Some("www.apple.com".to_string()),
            raw_json: None,
        }
    }

    #[test]
    fn saving_node_adds_it_to_proxy_selector() {
        let mut config: Value = serde_json::from_str(&default_singbox_config()).unwrap();
        normalize_config(&mut config);
        let node = build_manual_outbound(empty_input("Node A", "vless")).unwrap();
        upsert_outbound_value(&mut config, node).unwrap();
        refresh_proxy_selector(&mut config).unwrap();

        let proxy = config["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["tag"] == "Proxy")
            .unwrap();
        let members: Vec<&str> = proxy["outbounds"].as_array().unwrap().iter().filter_map(Value::as_str).collect();
        assert!(members.contains(&"Node A"));
        assert!(members.contains(&"direct"));
    }

    #[test]
    fn selector_groups_are_reported_as_policy_groups() {
        let mut config: Value = serde_json::from_str(&default_singbox_config()).unwrap();
        normalize_config(&mut config);
        upsert_outbound_value(&mut config, build_manual_outbound(empty_input("Node A", "vless")).unwrap()).unwrap();
        upsert_outbound_value(&mut config, json!({
            "type": "selector",
            "tag": "Auto",
            "outbounds": ["Node A", "direct"],
            "default": "Node A"
        })).unwrap();

        let groups = collect_policy_groups(&config);
        assert!(groups.iter().any(|group| group["name"] == "Proxy"));
        assert!(groups.iter().any(|group| group["name"] == "Auto" && group["selected"] == "Node A"));
    }
}
