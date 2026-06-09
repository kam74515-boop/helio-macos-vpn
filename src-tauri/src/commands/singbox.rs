use tauri::{AppHandle, State, Manager, Emitter};
use tauri_plugin_shell::ShellExt;
use std::path::PathBuf;
use crate::commands::traffic::get_traffic_stats_impl;
use std::fs;
use std::io::Write;
use crate::state::AppState;
use crate::types::{SingboxConfig, SingboxOutbound, SingboxRule};

#[tauri::command]
pub async fn get_singbox_config_json(app: AppHandle) -> Result<serde_json::Value, String> {
    let config_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = config_dir.join("config.json");

    let content = fs::read_to_string(&config_path)
        .unwrap_or_else(|_| default_singbox_config());

    let val: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    Ok(val)
}

#[tauri::command]
pub async fn get_singbox_config(app: AppHandle) -> Result<SingboxConfig, String> {
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

    let mode = detect_mode(&val);
    let config_name = val.get("_helio")
        .and_then(|h| h.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Default")
        .to_string();

    let policy_groups: Vec<serde_json::Value> = Vec::new();

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
    let config_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = config_dir.join("config.json");

    let content = fs::read_to_string(&config_path)
        .unwrap_or_else(|_| default_singbox_config());

    let mut config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    // Ensure route object exists
    if config.get("route").is_none() {
        config["route"] = serde_json::json!({});
    }

    match mode.as_str() {
        "direct" => {
            config["route"]["final"] = serde_json::json!("direct");
            config["route"]["auto_detect_interface"] = serde_json::json!(true);
        }
        "global" => {
            config["route"]["final"] = serde_json::json!("Proxy");
            config["route"]["auto_detect_interface"] = serde_json::json!(false);
        }
        "rule" => {
            config["route"]["final"] = serde_json::json!("Proxy");
            config["route"]["auto_detect_interface"] = serde_json::json!(true);
        }
        _ => return Err(format!("未知模式: {}", mode)),
    }

    // Write updated config
    let updated = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&config_path, &updated).map_err(|e| e.to_string())?;

    // Restart engine with new config
    start_engine(app, state, Some(updated)).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_proxy_mode(app: AppHandle) -> Result<String, String> {
    let config = get_singbox_config(app).await?;
    Ok(config.mode)
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
  }
}"#.to_string()
}

#[tauri::command]
pub async fn update_singbox_config(app: AppHandle, state: State<'_, AppState>, config: String) -> Result<(), String> {
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
    let config_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&config_dir).unwrap_or_default();
    let config_path = config_dir.join("config.json");
    let config_content = match config {
        Some(value) if !value.trim().is_empty() => value,
        _ => fs::read_to_string(&config_path)
            .ok()
            .filter(|content| has_runtime_config_shape(content))
            .unwrap_or_else(default_singbox_config),
    };

    let _: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("无效的 sing-box 配置 JSON: {}", e))?;

    if let Ok(mut file) = fs::File::create(&config_path) {
        let _ = file.write_all(config_content.as_bytes());
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
