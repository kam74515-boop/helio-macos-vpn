use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use std::fs;
use crate::config_store::{ConfigStore, Profile, Rule};

fn generate_rule_id() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}", timestamp)
}

fn parse_value(value: &str) -> Vec<String> {
    value
        .split(&[',', '\n', '\r'][..])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

async fn validate_with_singbox(app: &AppHandle, profile: &Profile) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    let runtime = store.get_runtime_config(profile);

    // Write to a temp path for validation
    let temp_path = store
        .get_runtime_config_path()
        .with_extension("check.json");
    fs::write(&temp_path, &runtime).map_err(|e| e.to_string())?;

    let sidecar = app
        .shell()
        .sidecar("sing-box")
        .map_err(|e| e.to_string())?;
    let (mut rx, _child) = sidecar
        .args(["check", "-c", temp_path.to_str().unwrap()])
        .spawn()
        .map_err(|e| format!("sing-box check 启动失败: {}", e))?;

    let mut output = String::new();
    let mut exit_code = None;

    while let Some(event) = rx.recv().await {
        match event {
            tauri_plugin_shell::process::CommandEvent::Stdout(line) => {
                output.push_str(&String::from_utf8_lossy(&line));
            }
            tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                output.push_str(&String::from_utf8_lossy(&line));
            }
            tauri_plugin_shell::process::CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
                break;
            }
            _ => {}
        }
    }

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    match exit_code {
        Some(0) => Ok(()),
        Some(code) => Err(format!(
            "sing-box check 失败 (exit {}): {}",
            code,
            output.trim()
        )),
        None => Err(format!(
            "sing-box check 异常终止: {}",
            output.trim()
        )),
    }
}

#[tauri::command]
pub async fn get_rules(app: AppHandle) -> Result<Vec<Rule>, String> {
    let store = app.state::<ConfigStore>();
    let profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;
    Ok(profile.rules)
}

#[tauri::command]
pub async fn add_rule(
    app: AppHandle,
    rule_type: String,
    value: String,
    action: String,
) -> Result<Rule, String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    // Keep original for rollback
    let original_profile = profile.clone();

    let rule = Rule {
        id: generate_rule_id(),
        rule_type,
        value: parse_value(&value),
        action,
        hits: "0".to_string(),
    };

    profile.rules.push(rule.clone());

    // Save and validate
    store.save_profile(&profile)?;
    if let Err(e) = validate_with_singbox(&app, &profile).await {
        // Rollback
        let _ = store.save_profile(&original_profile);
        return Err(e);
    }

    // Validation passed - write runtime config and restart engine
    let runtime = store.get_runtime_config(&profile);
    store.write_runtime_config(&runtime)?;

    // Restart engine so new rules take effect immediately
    let state = app.state::<crate::state::AppState>();
    let _ = crate::commands::singbox::start_engine(app.clone(), state, Some(runtime)).await;

    Ok(rule)
}

#[tauri::command]
pub async fn edit_rule(
    app: AppHandle,
    id: String,
    rule_type: String,
    value: String,
    action: String,
) -> Result<Rule, String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let original_profile = profile.clone();

    let rule = profile
        .rules
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("规则 {} 不存在", id))?;

    rule.rule_type = rule_type;
    rule.value = parse_value(&value);
    rule.action = action;

    let updated_rule = rule.clone();

    store.save_profile(&profile)?;
    if let Err(e) = validate_with_singbox(&app, &profile).await {
        let _ = store.save_profile(&original_profile);
        return Err(e);
    }

    let runtime = store.get_runtime_config(&profile);
    store.write_runtime_config(&runtime)?;

    // Restart engine so edited rules take effect immediately
    let state = app.state::<crate::state::AppState>();
    let _ = crate::commands::singbox::start_engine(app.clone(), state, Some(runtime)).await;

    Ok(updated_rule)
}

#[tauri::command]
pub async fn delete_rule(app: AppHandle, id: String) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let original_profile = profile.clone();
    let before = profile.rules.len();
    profile.rules.retain(|r| r.id != id);

    if profile.rules.len() == before {
        return Err(format!("规则 {} 不存在", id));
    }

    store.save_profile(&profile)?;
    if let Err(e) = validate_with_singbox(&app, &profile).await {
        let _ = store.save_profile(&original_profile);
        return Err(e);
    }

    let runtime = store.get_runtime_config(&profile);
    store.write_runtime_config(&runtime)?;

    // Restart engine so deleted rules take effect immediately
    let state = app.state::<crate::state::AppState>();
    let _ = crate::commands::singbox::start_engine(app.clone(), state, Some(runtime)).await;

    Ok(())
}

#[tauri::command]
pub async fn reorder_rules(app: AppHandle, ids: Vec<String>) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    let original_profile = profile.clone();

    // Build new order based on ids
    let mut new_rules = Vec::new();
    for id in &ids {
        if let Some(rule) = profile.rules.iter().find(|r| r.id == *id) {
            new_rules.push(rule.clone());
        }
    }

    // Add any rules not in ids (preserve them at the end)
    for rule in &profile.rules {
        if !ids.contains(&rule.id) {
            new_rules.push(rule.clone());
        }
    }

    profile.rules = new_rules;

    store.save_profile(&profile)?;
    if let Err(e) = validate_with_singbox(&app, &profile).await {
        let _ = store.save_profile(&original_profile);
        return Err(e);
    }

    let runtime = store.get_runtime_config(&profile);
    store.write_runtime_config(&runtime)?;

    // Restart engine so reordered rules take effect immediately
    let state = app.state::<crate::state::AppState>();
    let _ = crate::commands::singbox::start_engine(app.clone(), state, Some(runtime)).await;

    Ok(())
}

#[tauri::command]
pub async fn reset_rule_counters(app: AppHandle) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store
        .get_active_profile()
        .ok_or_else(|| "没有活跃的 profile".to_string())?;

    for rule in &mut profile.rules {
        rule.hits = "0".to_string();
    }

    store.save_profile(&profile)?;
    Ok(())
}
